//! Logs estruturados (SPEC §10): arquivo em XDG state + espelho no stderr.
//!
//! Nunca logar credenciais, conteúdo clínico nem a config completa —
//! apenas caminho/perfil e metadados de diagnóstico.

use std::path::Path;

use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Mantém o worker do appender vivo; se cair, o log em arquivo para.
pub struct LogGuard {
    _file_guard: Option<tracing_appender::non_blocking::WorkerGuard>,
}

/// Inicializa tracing com camada de arquivo (`<state>/qara-desktop.log`)
/// e camada stderr. Se o diretório de state não puder ser criado, segue
/// só com stderr (logando o motivo) — logging nunca derruba o app.
pub fn init(state_dir: &Path) -> LogGuard {
    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_target(false);

    let file_layer = match std::fs::create_dir_all(state_dir) {
        Ok(()) => {
            let appender = tracing_appender::rolling::never(state_dir, "qara-desktop.log");
            let (writer, guard) = tracing_appender::non_blocking(appender);
            let layer = tracing_subscriber::fmt::layer()
                .with_writer(writer)
                .with_ansi(false)
                .with_target(false);
            Some((layer, guard))
        }
        Err(e) => {
            eprintln!(
                "qara-desktop: não foi possível criar {}: {e}; log só no stderr",
                state_dir.display()
            );
            None
        }
    };

    let (file_layer, file_guard) = match file_layer {
        Some((layer, guard)) => (Some(layer), Some(guard)),
        None => (None, None),
    };

    tracing_subscriber::registry()
        .with(stderr_layer.with_filter(tracing::level_filters::LevelFilter::INFO))
        .with(file_layer.map(|l| l.with_filter(tracing::level_filters::LevelFilter::INFO)))
        .init();

    LogGuard {
        _file_guard: file_guard,
    }
}

/// Hook de panic que registra no log antes de abortar (SPEC §10: crash/panic).
pub fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "payload de panic não textual".to_string());
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "local desconhecido".to_string());
        tracing::error!(local = %location, mensagem = %payload, "panic — abortando");
        default_hook(info);
    }));
}

/// Registra o ambiente detectado (SPEC §10). Só metadados de sessão,
/// nada sensível.
pub fn log_environment() {
    let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "?".into());
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "?".into());
    let wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
    tracing::info!(
        sessao = %session_type,
        desktop = %desktop,
        wayland,
        "ambiente detectado"
    );
}
