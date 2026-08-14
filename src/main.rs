//! QARA Desktop native host.
//!
//! The first Codex task is to validate exact crate/system-library compatibility
//! on the target Pop!_OS machine and complete the WebKitGTK integration.

use anyhow::Result;
use gtk4 as gtk;
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

fn main() -> Result<()> {
    let windowed = std::env::args().any(|arg| arg == "--windowed");

    let app = gtk::Application::builder()
        .application_id("br.com.clinicaqara.desktop")
        .build();

    app.connect_activate(move |app| build_ui(app, windowed));
    app.run();

    Ok(())
}

fn build_ui(app: &gtk::Application, windowed: bool) {
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("QARA Desktop")
        .default_width(1440)
        .default_height(900)
        .build();

    if !windowed {
        if gtk4_layer_shell::is_supported() {
            window.init_layer_shell();
            window.set_namespace(Some("qara-desktop"));
            window.set_layer(Layer::Background);
            window.set_keyboard_mode(KeyboardMode::None);
            window.set_exclusive_zone(0);
            for edge in [Edge::Top, Edge::Right, Edge::Bottom, Edge::Left] {
                window.set_anchor(edge, true);
            }
        } else {
            eprintln!("QARA Desktop: layer-shell indisponível; use --windowed para depuração.");
        }
    }

    // TODO(Fase 1):
    // 1. criar WebKitWebView (webkit6);
    // 2. carregar ui/index.html por caminho absoluto/canonicalizado;
    // 3. interceptar navegação externa;
    // 4. criar bridge tipada para open_url/launch_app;
    // 5. validar config JSON e allowlist.
    //
    // Placeholder deliberado para manter o starter simples antes do diagnóstico
    // das bibliotecas reais do computador alvo.
    let label = gtk::Label::new(Some(
        "QARA Desktop\n\nHost GTK inicializado.\nAbra ui/index.html para visualizar o protótipo."
    ));
    label.set_justify(gtk::Justification::Center);
    window.set_child(Some(&label));

    window.present();
}
