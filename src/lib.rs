//! QARA Desktop — host nativo (GTK4 + gtk4-layer-shell + WebKitGTK 6).
//!
//! A lógica de negócio (config, bridge, launcher, prefs, paths, cli) não
//! depende de GTK e é coberta por testes em `tests/`. A integração
//! GTK/WebKit fica em `app`.

pub mod app;
pub mod bridge;
pub mod cli;
pub mod config;
pub mod launcher;
pub mod logging;
pub mod paths;
pub mod prefs;
