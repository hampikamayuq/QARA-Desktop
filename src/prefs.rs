//! Preferências de UI persistidas (`state.json`).
//!
//! Somente preferências não sensíveis; hoje apenas `compact` (bool).
//! Key desconhecida ou valor de tipo errado → `invalid_preference`.
//! Escrita atômica: temp no mesmo diretório + rename.

use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::bridge::ErrorCode;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Prefs {
    pub compact: bool,
}

/// Carrega as preferências; arquivo ausente ou inválido cai no padrão
/// (preferência perdida não é motivo para crash — apenas log).
pub fn load(path: &Path) -> Prefs {
    match std::fs::read_to_string(path) {
        Ok(raw) => match serde_json::from_str::<Prefs>(&raw) {
            Ok(prefs) => prefs,
            Err(e) => {
                tracing::warn!(
                    caminho = %path.display(),
                    erro = %e,
                    "state.json inválido; usando preferências padrão"
                );
                Prefs::default()
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Prefs::default(),
        Err(e) => {
            tracing::warn!(
                caminho = %path.display(),
                erro = %e,
                "falha ao ler state.json; usando preferências padrão"
            );
            Prefs::default()
        }
    }
}

/// Aplica `set_ui_preference`. Keys aceitas: `compact` (bool).
pub fn apply(prefs: &mut Prefs, key: &str, value: &Value) -> Result<(), ErrorCode> {
    match (key, value) {
        ("compact", Value::Bool(b)) => {
            prefs.compact = *b;
            Ok(())
        }
        _ => Err(ErrorCode::InvalidPreference),
    }
}

/// Grava atomicamente: escreve em arquivo temporário no mesmo diretório
/// e faz rename por cima do destino.
pub fn save_atomic(path: &Path, prefs: &Prefs) -> std::io::Result<()> {
    let dir = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "caminho de state.json sem diretório pai",
        )
    })?;
    std::fs::create_dir_all(dir)?;
    let mut tmp = tempfile_in(dir)?;
    let json = serde_json::to_string_pretty(prefs).expect("Prefs sempre serializa");
    tmp.file.write_all(json.as_bytes())?;
    tmp.file.write_all(b"\n")?;
    tmp.file.sync_all()?;
    std::fs::rename(&tmp.path, path)?;
    tmp.persisted = true;
    Ok(())
}

/// Arquivo temporário mínimo (evita a dependência `tempfile` para um uso só).
struct TempFile {
    file: std::fs::File,
    path: std::path::PathBuf,
    persisted: bool,
}

impl Drop for TempFile {
    fn drop(&mut self) {
        if !self.persisted {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

fn tempfile_in(dir: &Path) -> std::io::Result<TempFile> {
    let path = dir.join(format!(".state.json.tmp-{}", std::process::id()));
    let file = std::fs::File::create(&path)?;
    Ok(TempFile {
        file,
        path,
        persisted: false,
    })
}
