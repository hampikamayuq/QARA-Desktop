//! Schema e validação da configuração (`config.json`).
//!
//! Shape exato definido em `docs/PROTOCOL.md`. Campos desconhecidos são
//! rejeitados (`deny_unknown_fields`) e toda validação produz erro legível
//! em pt-BR apontando o campo problemático.

use std::collections::BTreeMap;
use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Config de fallback embutida no binário. Mantida em `config/shortcuts.example.json`.
pub const FALLBACK_CONFIG_JSON: &str = include_str!("../config/shortcuts.example.json");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub profile: String,
    pub locale: String,
    /// Textos institucionais opcionais; ausentes, a UI usa os padrões dela.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branding: Option<Branding>,
    pub groups: Vec<Group>,
    /// id lógico -> argv executado sem shell (`Command::new(argv[0]).args(&argv[1..])`).
    pub app_allowlist: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Branding {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kicker: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wordmark: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headline: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tagline: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub id: String,
    pub label: String,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub kind: ItemKind,
    pub target: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    Url,
    App,
}

impl fmt::Display for ItemKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemKind::Url => write!(f, "url"),
            ItemKind::App => write!(f, "app"),
        }
    }
}

/// Origem da config efetivamente carregada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigSource {
    /// Lida e validada de `~/.config/qara-desktop/config.json`.
    User(PathBuf),
    /// Fallback embutido no binário.
    Fallback,
}

impl fmt::Display for ConfigSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigSource::User(path) => write!(f, "{}", path.display()),
            ConfigSource::Fallback => write!(f, "fallback embutido"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoadedConfig {
    pub config: Config,
    pub source: ConfigSource,
}

/// Faz parse estrito + validação semântica. Erro é uma mensagem legível
/// em pt-BR (possivelmente multi-linha) apontando os campos problemáticos.
pub fn parse_and_validate(json: &str) -> Result<Config, String> {
    let config: Config =
        serde_json::from_str(json).map_err(|e| format!("JSON da configuração inválido: {e}"))?;
    let problems = validate(&config);
    if problems.is_empty() {
        Ok(config)
    } else {
        Err(problems.join("\n"))
    }
}

/// Valida as regras de `docs/PROTOCOL.md`; retorna a lista de problemas (vazia = ok).
pub fn validate(config: &Config) -> Vec<String> {
    let mut problems = Vec::new();
    let mut group_ids: HashSet<&str> = HashSet::new();
    let mut item_ids: HashSet<&str> = HashSet::new();

    for group in &config.groups {
        if !group_ids.insert(group.id.as_str()) {
            problems.push(format!("groups: id de grupo duplicado: \"{}\"", group.id));
        }
        for item in &group.items {
            let ctx = format!("groups[\"{}\"].items[\"{}\"]", group.id, item.id);
            if !item_ids.insert(item.id.as_str()) {
                problems.push(format!("{ctx}: id de item duplicado: \"{}\"", item.id));
            }
            match item.kind {
                ItemKind::Url => match url::Url::parse(&item.target) {
                    Ok(parsed) if matches!(parsed.scheme(), "https" | "mailto") => {}
                    Ok(parsed) => problems.push(format!(
                        "{ctx}.target: esquema \"{}:\" não permitido (apenas https: e mailto:)",
                        parsed.scheme()
                    )),
                    Err(e) => problems.push(format!("{ctx}.target: URL inválida: {e}")),
                },
                ItemKind::App => {
                    if !config.app_allowlist.contains_key(&item.target) {
                        problems.push(format!(
                            "{ctx}.target: \"{}\" não existe em app_allowlist",
                            item.target
                        ));
                    }
                }
            }
        }
    }

    for (id, argv) in &config.app_allowlist {
        if argv.is_empty() {
            problems.push(format!("app_allowlist[\"{id}\"]: argv não pode ser vazio"));
        } else if argv[0].trim().is_empty() {
            problems.push(format!(
                "app_allowlist[\"{id}\"]: o executável (argv[0]) não pode ser vazio"
            ));
        }
    }

    problems
}

/// Config de fallback embutida. Garantida por teste; se corromper no build,
/// é bug de empacotamento e o panic aponta direto para o arquivo.
pub fn fallback_config() -> Config {
    parse_and_validate(FALLBACK_CONFIG_JSON)
        .expect("config/shortcuts.example.json embutida deveria ser sempre válida")
}

/// Carrega a config do caminho do usuário; erro (leitura, parse ou validação)
/// é logado e o fallback embutido entra no lugar — nunca crasha.
pub fn load_from(path: &Path) -> LoadedConfig {
    match std::fs::read_to_string(path) {
        Ok(raw) => match parse_and_validate(&raw) {
            Ok(config) => {
                tracing::info!(caminho = %path.display(), perfil = %config.profile, "configuração do operador carregada");
                LoadedConfig {
                    config,
                    source: ConfigSource::User(path.to_path_buf()),
                }
            }
            Err(problems) => {
                tracing::error!(
                    caminho = %path.display(),
                    problemas = %problems,
                    "configuração inválida; usando fallback embutido"
                );
                LoadedConfig {
                    config: fallback_config(),
                    source: ConfigSource::Fallback,
                }
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::info!(
                caminho = %path.display(),
                "config.json ausente; usando fallback embutido"
            );
            LoadedConfig {
                config: fallback_config(),
                source: ConfigSource::Fallback,
            }
        }
        Err(e) => {
            tracing::error!(
                caminho = %path.display(),
                erro = %e,
                "falha ao ler configuração; usando fallback embutido"
            );
            LoadedConfig {
                config: fallback_config(),
                source: ConfigSource::Fallback,
            }
        }
    }
}

/// Carrega a config do caminho XDG padrão.
pub fn load() -> LoadedConfig {
    load_from(&crate::paths::config_file())
}
