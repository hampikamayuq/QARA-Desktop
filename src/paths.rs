//! Resolução de caminhos XDG do QARA Desktop.
//!
//! Feito à mão (sem crate `dirs`/`etcetera`) porque só precisamos de duas
//! variáveis XDG com fallback padrão — não justifica dependência nova.

use std::env;
use std::path::PathBuf;

const APP_DIR: &str = "qara-desktop";

/// Retorna o valor de `var` se for um caminho absoluto não vazio.
/// A especificação XDG manda ignorar valores relativos.
fn xdg_dir(var: &str) -> Option<PathBuf> {
    let value = env::var_os(var)?;
    if value.is_empty() {
        return None;
    }
    let path = PathBuf::from(value);
    path.is_absolute().then_some(path)
}

fn home() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

/// `$XDG_CONFIG_HOME/qara-desktop` (padrão `~/.config/qara-desktop`).
pub fn config_dir() -> PathBuf {
    xdg_dir("XDG_CONFIG_HOME")
        .unwrap_or_else(|| home().join(".config"))
        .join(APP_DIR)
}

/// Caminho do `config.json` do operador.
pub fn config_file() -> PathBuf {
    config_dir().join("config.json")
}

/// Caminho do `state.json` (preferências persistidas).
pub fn prefs_file() -> PathBuf {
    config_dir().join("state.json")
}

/// `$XDG_STATE_HOME/qara-desktop` (padrão `~/.local/state/qara-desktop`).
pub fn state_dir() -> PathBuf {
    xdg_dir("XDG_STATE_HOME")
        .unwrap_or_else(|| home().join(".local").join("state"))
        .join(APP_DIR)
}
