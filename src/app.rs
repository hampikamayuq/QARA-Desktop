//! Parte GTK/WebKit do host: janelas layer-shell (uma por monitor),
//! WebView endurecida servindo a UI por `qara://`, bridge `qara`,
//! watcher de config e injeção de estado.
//!
//! Toda a lógica testável (config, bridge, launcher, prefs) vive nos
//! módulos irmãos, sem dependência de GTK.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use gtk::{gdk, gio, glib};
use gtk4 as gtk;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use webkit6::prelude::*;
use webkit6::{
    NavigationPolicyDecision, NetworkSession, PolicyDecisionType, Settings, URISchemeRequest,
    UserContentInjectedFrames, UserContentManager, UserScript, UserScriptInjectionTime, WebContext,
    WebView,
};

use crate::bridge::{self, Action, ErrorCode, Response};
use crate::config::{self, Config, ConfigSource};
use crate::launcher;
use crate::prefs::{self, Prefs};
use crate::{cli, paths};

/// Assets da UI embutidos no binário (servidos via `qara://ui/...`).
const UI_INDEX_HTML: &[u8] = include_bytes!("../ui/index.html");
const UI_STYLES_CSS: &[u8] = include_bytes!("../ui/styles.css");
const UI_APP_JS: &[u8] = include_bytes!("../ui/app.js");

const UI_ENTRY_URI: &str = "qara://ui/index.html";
/// Debounce do watcher de config.
const CONFIG_DEBOUNCE: Duration = Duration::from_millis(500);

struct WindowEntry {
    monitor: Option<gdk::Monitor>,
    window: gtk::ApplicationWindow,
    webview: WebView,
    ucm: UserContentManager,
}

struct Host {
    options: cli::Options,
    config: RefCell<Config>,
    config_source: RefCell<ConfigSource>,
    prefs: RefCell<Prefs>,
    windows: RefCell<Vec<WindowEntry>>,
    web_context: WebContext,
    network_session: NetworkSession,
    settings: Settings,
    /// Mantém vivos o monitor de arquivo e o modelo de monitores.
    config_monitor: RefCell<Option<gio::FileMonitor>>,
    monitors_model: RefCell<Option<gio::ListModel>>,
    debounce_source: RefCell<Option<glib::SourceId>>,
    exit_code: std::cell::Cell<i32>,
}

/// Sobe o app GTK. Retorna o código de saída do processo.
pub fn run(options: cli::Options) -> i32 {
    let app = gtk::Application::builder()
        .application_id("br.com.clinicaqara.desktop")
        .build();

    let loaded = config::load();
    let prefs = prefs::load(&paths::prefs_file());

    let web_context = WebContext::new();
    // Sessão EFÊMERA: sem cache nem cookies persistentes — a UI é local.
    let network_session = NetworkSession::new_ephemeral();
    let settings = build_settings(&options);

    let host = Rc::new(Host {
        options,
        config: RefCell::new(loaded.config),
        config_source: RefCell::new(loaded.source),
        prefs: RefCell::new(prefs),
        windows: RefCell::new(Vec::new()),
        web_context,
        network_session,
        settings,
        config_monitor: RefCell::new(None),
        monitors_model: RefCell::new(None),
        debounce_source: RefCell::new(None),
        exit_code: std::cell::Cell::new(0),
    });

    register_qara_scheme(&host);

    let host_activate = host.clone();
    app.connect_activate(move |app| activate(app, &host_activate));

    // GTK não deve interpretar nossas flags próprias.
    let status: i32 = app.run_with_args::<&str>(&[]).into();
    if status != 0 {
        status
    } else {
        host.exit_code.get()
    }
}

fn activate(app: &gtk::Application, host: &Rc<Host>) {
    if !host.windows.borrow().is_empty() {
        // Segunda ativação (instância única do GApplication): já estamos no ar.
        tracing::info!("ativação repetida ignorada; instância já em execução");
        return;
    }

    if host.options.windowed {
        tracing::info!("modo --windowed: janela normal única para depuração");
        create_window(app, host, None);
    } else if gtk4_layer_shell::is_supported() {
        tracing::info!("layer-shell suportado; criando uma janela por monitor");
        setup_monitor_windows(app, host);
    } else {
        tracing::error!(
            "gtk4-layer-shell não é suportado neste compositor; \
             rode com --windowed para depuração (sem fallback X11)"
        );
        eprintln!(
            "qara-desktop: layer-shell indisponível neste ambiente (Wayland/COSMIC esperado).\n\
             Use `qara-desktop --windowed` para depuração."
        );
        host.exit_code.set(1);
        app.quit();
        return;
    }

    watch_config(host);
}

// ---------------------------------------------------------------------------
// Janelas / multi-monitor
// ---------------------------------------------------------------------------

fn setup_monitor_windows(app: &gtk::Application, host: &Rc<Host>) {
    let Some(display) = gdk::Display::default() else {
        tracing::error!("nenhum display GDK disponível");
        host.exit_code.set(1);
        app.quit();
        return;
    };
    let monitors = display.monitors();

    sync_monitor_windows(app, host, &monitors);

    let app_weak = app.downgrade();
    let host_hotplug = host.clone();
    monitors.connect_items_changed(move |model, _, _, _| {
        if let Some(app) = app_weak.upgrade() {
            tracing::info!("mudança no conjunto de monitores; sincronizando janelas");
            sync_monitor_windows(&app, &host_hotplug, model);
        }
    });
    host.monitors_model.replace(Some(monitors));
}

/// Cria/destrói janelas para casar com o modelo atual de monitores (hotplug).
fn sync_monitor_windows(app: &gtk::Application, host: &Rc<Host>, monitors: &gio::ListModel) {
    let current: Vec<gdk::Monitor> = (0..monitors.n_items())
        .filter_map(|i| monitors.item(i).and_downcast::<gdk::Monitor>())
        .collect();

    // Remove janelas de monitores que sumiram ou ficaram inválidos.
    host.windows.borrow_mut().retain(|entry| {
        let keep = match &entry.monitor {
            Some(monitor) => monitor.is_valid() && current.contains(monitor),
            None => true,
        };
        if !keep {
            tracing::info!("monitor removido; destruindo janela correspondente");
            entry.window.destroy();
        }
        keep
    });

    // Cria janelas para monitores novos.
    for monitor in &current {
        let already = host
            .windows
            .borrow()
            .iter()
            .any(|e| e.monitor.as_ref() == Some(monitor));
        if !already {
            let connector = monitor.connector().unwrap_or_else(|| "?".into());
            tracing::info!(conector = %connector, "criando janela layer-shell para monitor");
            create_window(app, host, Some(monitor.clone()));
        }
    }
}

fn create_window(app: &gtk::Application, host: &Rc<Host>, monitor: Option<gdk::Monitor>) {
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("QARA Desktop")
        .default_width(1440)
        .default_height(900)
        .build();

    if !host.options.windowed {
        // Comportamento de wallpaper (SPEC §3): fundo, ancorado nos 4 cantos,
        // sem teclado, sem exclusive zone, fora do Alt+Tab.
        window.init_layer_shell();
        window.set_namespace(Some("qara-desktop"));
        window.set_layer(Layer::Background);
        window.set_keyboard_mode(KeyboardMode::None);
        window.set_exclusive_zone(0);
        for edge in [Edge::Top, Edge::Right, Edge::Bottom, Edge::Left] {
            window.set_anchor(edge, true);
        }
        if let Some(monitor) = &monitor {
            window.set_monitor(Some(monitor));
        }
    }

    let (webview, ucm) = build_webview(host);
    window.set_child(Some(&webview));
    window.present();

    webview.load_uri(UI_ENTRY_URI);

    host.windows.borrow_mut().push(WindowEntry {
        monitor,
        window,
        webview,
        ucm,
    });
}

// ---------------------------------------------------------------------------
// WebView endurecida
// ---------------------------------------------------------------------------

fn build_settings(options: &cli::Options) -> Settings {
    let settings = Settings::new();
    // DevTools somente com --devtools (produção: desabilitado).
    settings.set_enable_developer_extras(options.devtools);
    settings.set_enable_write_console_messages_to_stdout(options.devtools);
    settings.set_javascript_can_open_windows_automatically(false);
    settings.set_allow_modal_dialogs(false);
    settings
}

fn build_webview(host: &Rc<Host>) -> (WebView, UserContentManager) {
    let ucm = UserContentManager::new();
    install_bootstrap_script(host, &ucm);
    if !ucm.register_script_message_handler("qara", None) {
        tracing::error!("falha ao registrar o message handler \"qara\"");
    }

    let webview = WebView::builder()
        .web_context(&host.web_context)
        .network_session(&host.network_session)
        .user_content_manager(&ucm)
        .settings(&host.settings)
        .build();

    // Bridge UI → host.
    let host_msg = host.clone();
    let webview_weak = webview.downgrade();
    ucm.connect_script_message_received(Some("qara"), move |_, value| {
        let Some(webview) = webview_weak.upgrade() else {
            return;
        };
        let raw = if value.is_string() {
            value.to_str().to_string()
        } else {
            // Mensagem não-string: erro de contrato, respondido como invalid_message.
            String::new()
        };
        handle_bridge_message(&host_msg, &webview, &raw);
    });

    // Diagnóstico do ciclo de carga da UI (SPEC §10).
    webview.connect_load_changed(|_, event| {
        if event == webkit6::LoadEvent::Finished {
            tracing::info!("UI carregada na WebView");
        }
    });
    webview.connect_load_failed(|_, _, uri, error| {
        tracing::error!(uri = %uri, erro = %error, "falha ao carregar a UI");
        false // deixa o WebKit seguir o tratamento padrão
    });

    // Nega criação de novas webviews (popups, window.open, target=_blank).
    webview.connect_create(|_, _| {
        tracing::warn!("tentativa de abrir nova webview bloqueada");
        None
    });

    // Nega QUALQUER navegação fora de qara://.
    webview.connect_decide_policy(|_, decision, decision_type| {
        match decision_type {
            PolicyDecisionType::NavigationAction | PolicyDecisionType::NewWindowAction => {
                let uri = decision
                    .downcast_ref::<NavigationPolicyDecision>()
                    .and_then(|d| d.navigation_action())
                    .and_then(|a| a.request())
                    .and_then(|r| r.uri())
                    .map(|u| u.to_string())
                    .unwrap_or_default();
                if uri.starts_with("qara://")
                    && decision_type == PolicyDecisionType::NavigationAction
                {
                    false // segue o fluxo padrão (permitido)
                } else {
                    tracing::warn!(uri = %sanitized_uri(&uri), "navegação fora de qara:// negada");
                    decision.ignore();
                    true
                }
            }
            _ => false,
        }
    });

    (webview, ucm)
}

/// Loga só esquema+host de URIs negadas — nunca a URI completa (pode conter
/// query com dado sensível).
fn sanitized_uri(uri: &str) -> String {
    match url::Url::parse(uri) {
        Ok(u) => {
            let host = u.host_str().unwrap_or("");
            format!("{}://{}/…", u.scheme(), host)
        }
        Err(_) => "<uri não analisável>".to_string(),
    }
}

/// (Re)instala o user script de `document-start` com config + prefs atuais.
fn install_bootstrap_script(host: &Rc<Host>, ucm: &UserContentManager) {
    let config_json = serde_json::to_string(&*host.config.borrow())
        .expect("Config sempre serializa (tipos fechados)");
    let prefs_json = serde_json::to_string(&*host.prefs.borrow())
        .expect("Prefs sempre serializa (tipos fechados)");
    let source = bridge::bootstrap_script(&config_json, &prefs_json);
    ucm.remove_all_scripts();
    ucm.add_script(&UserScript::new(
        &source,
        UserContentInjectedFrames::TopFrame,
        UserScriptInjectionTime::Start,
        &[],
        &[],
    ));
}

fn refresh_bootstrap_scripts(host: &Rc<Host>, reload: bool) {
    for entry in host.windows.borrow().iter() {
        install_bootstrap_script(host, &entry.ucm);
        if reload {
            entry.webview.reload();
        }
    }
}

// ---------------------------------------------------------------------------
// Custom scheme qara://
// ---------------------------------------------------------------------------

fn register_qara_scheme(host: &Rc<Host>) {
    let ui_dir = host.options.ui_dir.clone();
    host.web_context
        .register_uri_scheme("qara", move |request| {
            serve_qara_request(request, ui_dir.as_deref());
        });
}

fn serve_qara_request(request: &URISchemeRequest, ui_dir: Option<&Path>) {
    let uri = request.uri().map(|u| u.to_string()).unwrap_or_default();
    match resolve_ui_asset(&uri, ui_dir) {
        Some((body, content_type)) => {
            let len = body.len() as i64;
            let stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from_owned(body));
            request.finish(&stream, len, Some(content_type));
        }
        None => {
            tracing::warn!(uri = %uri, "asset qara:// não encontrado");
            let mut error = glib::Error::new(
                glib::FileError::Noent,
                "recurso não encontrado sob qara://ui/",
            );
            request.finish_error(&mut error);
        }
    }
}

/// Resolve `qara://ui/<asset>` para (conteúdo, content-type). `ui_dir`
/// presente → lê do disco (desenvolvimento); ausente → assets embutidos.
fn resolve_ui_asset(uri: &str, ui_dir: Option<&Path>) -> Option<(Vec<u8>, &'static str)> {
    let rest = uri.strip_prefix("qara://")?;
    let rest = rest.split(['?', '#']).next().unwrap_or("");
    let name = match rest.strip_prefix("ui/") {
        Some("") | None if rest == "ui" => "index.html",
        Some("") => "index.html",
        Some(name) => name,
        None => return None,
    };

    // Nada de path traversal, nem em modo desenvolvimento.
    if name
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return None;
    }

    match ui_dir {
        Some(dir) => {
            let path = dir.join(name);
            let body = std::fs::read(&path)
                .map_err(|e| {
                    tracing::warn!(caminho = %path.display(), erro = %e, "falha lendo asset de --ui-dir");
                })
                .ok()?;
            Some((body, content_type_for(name)))
        }
        None => {
            let body: &[u8] = match name {
                "index.html" => UI_INDEX_HTML,
                "styles.css" => UI_STYLES_CSS,
                "app.js" => UI_APP_JS,
                _ => return None,
            };
            Some((body.to_vec(), content_type_for(name)))
        }
    }
}

fn content_type_for(name: &str) -> &'static str {
    match name.rsplit('.').next() {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "text/javascript",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("webp") => "image/webp",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}

// ---------------------------------------------------------------------------
// Bridge: tratamento das mensagens
// ---------------------------------------------------------------------------

fn handle_bridge_message(host: &Rc<Host>, webview: &WebView, raw: &str) {
    let request = match bridge::parse_message(raw) {
        Ok(request) => request,
        Err(failure) => {
            tracing::error!(
                codigo = failure.code.as_str(),
                detalhe = %failure.detail,
                "mensagem inválida da bridge"
            );
            // Sem req_id não há como correlacionar; ainda assim respondemos
            // com req_id vazio para a UI poder logar o erro.
            let req_id = failure.req_id.unwrap_or_default();
            send_response(webview, &Response::err(req_id, failure.code));
            return;
        }
    };

    let response = match &request.action {
        Action::OpenUrl { id } => do_open_url(host, &request.req_id, id),
        Action::LaunchApp { id } => do_launch_app(host, &request.req_id, id),
        Action::GetSystemInfo => do_system_info(host, &request.req_id),
        Action::SetUiPreference { key, value } => {
            do_set_preference(host, &request.req_id, key, value)
        }
    };
    send_response(webview, &response);
}

fn do_open_url(host: &Rc<Host>, req_id: &str, id: &str) -> Response {
    let config = host.config.borrow();
    let url = match launcher::resolve_url(&config, id) {
        Ok(url) => url.to_string(),
        Err(code) => {
            tracing::error!(id = %id, codigo = code.as_str(), "open_url recusado");
            return Response::err(req_id, code);
        }
    };
    drop(config);

    // Defesa em profundidade: a config já foi validada, mas conferimos o
    // esquema de novo imediatamente antes de abrir.
    if !launcher::url_scheme_allowed(&url) {
        tracing::error!(id = %id, "open_url recusado: esquema não permitido");
        return Response::err(req_id, ErrorCode::UnknownId);
    }

    match gio::AppInfo::launch_default_for_uri(&url, None::<&gio::AppLaunchContext>) {
        Ok(()) => {
            tracing::info!(id = %id, "URL aberta no navegador padrão");
            Response::ok(req_id, None)
        }
        Err(e) => {
            tracing::error!(id = %id, erro = %e, "falha ao abrir URL no navegador padrão");
            Response::err(req_id, ErrorCode::LaunchFailed)
        }
    }
}

fn do_launch_app(host: &Rc<Host>, req_id: &str, id: &str) -> Response {
    let config = host.config.borrow();
    let argv = match launcher::resolve_app(&config, id) {
        Ok(argv) => argv.to_vec(),
        Err(code) => {
            tracing::error!(id = %id, codigo = code.as_str(), "launch_app recusado");
            return Response::err(req_id, code);
        }
    };
    drop(config);

    match launcher::spawn_app(&argv) {
        Ok(pid) => {
            tracing::info!(id = %id, executavel = %argv[0], pid, "aplicativo iniciado");
            Response::ok(req_id, None)
        }
        Err(e) => {
            tracing::error!(id = %id, executavel = %argv[0], erro = %e, "falha ao iniciar aplicativo");
            Response::err(req_id, ErrorCode::LaunchFailed)
        }
    }
}

fn do_system_info(host: &Rc<Host>, req_id: &str) -> Response {
    let data = serde_json::json!({
        "hostname": glib::host_name().to_string(),
        "profile": host.config.borrow().profile,
        "version": env!("CARGO_PKG_VERSION"),
    });
    Response::ok(req_id, Some(data))
}

fn do_set_preference(
    host: &Rc<Host>,
    req_id: &str,
    key: &str,
    value: &serde_json::Value,
) -> Response {
    let mut prefs = host.prefs.borrow_mut();
    if let Err(code) = prefs::apply(&mut prefs, key, value) {
        tracing::error!(key = %key, "preferência recusada");
        return Response::err(req_id, code);
    }
    let snapshot = *prefs;
    drop(prefs);

    // Persistência: falha vira log (a preferência já vale em memória).
    if let Err(e) = prefs::save_atomic(&paths::prefs_file(), &snapshot) {
        tracing::error!(erro = %e, "falha ao persistir state.json");
    }
    // Atualiza os user scripts para o próximo load refletir a preferência.
    refresh_bootstrap_scripts(host, false);
    Response::ok(req_id, None)
}

fn send_response(webview: &WebView, response: &Response) {
    let script = bridge::response_script(response);
    webview.evaluate_javascript(&script, None, None, None::<&gio::Cancellable>, |result| {
        if let Err(e) = result {
            tracing::warn!(erro = %e, "falha ao entregar resposta à UI");
        }
    });
}

// ---------------------------------------------------------------------------
// Watcher de config (P2.6)
// ---------------------------------------------------------------------------

fn watch_config(host: &Rc<Host>) {
    let path = paths::config_file();
    let file = gio::File::for_path(&path);
    match file.monitor_file(gio::FileMonitorFlags::NONE, None::<&gio::Cancellable>) {
        Ok(monitor) => {
            let host_watch = host.clone();
            let path_watch = path.clone();
            monitor.connect_changed(move |_, _, _, _| {
                schedule_config_reload(&host_watch, path_watch.clone());
            });
            host.config_monitor.replace(Some(monitor));
            tracing::info!(caminho = %path.display(), "observando config.json");
        }
        Err(e) => {
            tracing::warn!(caminho = %path.display(), erro = %e, "não foi possível observar config.json");
        }
    }
}

fn schedule_config_reload(host: &Rc<Host>, path: PathBuf) {
    // Debounce: editores disparam rajadas de eventos; só a última conta.
    if let Some(source) = host.debounce_source.borrow_mut().take() {
        source.remove();
    }
    let host_reload = host.clone();
    let source = glib::timeout_add_local_once(CONFIG_DEBOUNCE, move || {
        host_reload.debounce_source.borrow_mut().take();
        reload_config(&host_reload, &path);
    });
    host.debounce_source.replace(Some(source));
}

/// Revalida a config após mudança no arquivo. Válida → aplica e recarrega
/// as webviews; inválida → mantém a config anterior (só loga).
fn reload_config(host: &Rc<Host>, path: &Path) {
    let (new_config, new_source) = match std::fs::read_to_string(path) {
        Ok(raw) => match config::parse_and_validate(&raw) {
            Ok(cfg) => (cfg, ConfigSource::User(path.to_path_buf())),
            Err(problems) => {
                tracing::error!(
                    caminho = %path.display(),
                    problemas = %problems,
                    "config.json editado ficou inválido; mantendo config anterior"
                );
                return;
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::warn!(
                caminho = %path.display(),
                "config.json removido; voltando ao fallback embutido"
            );
            (config::fallback_config(), ConfigSource::Fallback)
        }
        Err(e) => {
            tracing::error!(
                caminho = %path.display(),
                erro = %e,
                "falha ao reler config.json; mantendo config anterior"
            );
            return;
        }
    };

    if *host.config.borrow() == new_config {
        tracing::info!("config relida sem mudanças efetivas");
        return;
    }

    tracing::info!(origem = %new_source, perfil = %new_config.profile, "aplicando nova configuração");
    host.config.replace(new_config);
    host.config_source.replace(new_source);
    refresh_bootstrap_scripts(host, true);
}
