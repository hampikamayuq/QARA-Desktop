//! Resolução de IDs contra a config validada e spawn de processos.
//!
//! Regra central de segurança (SPEC §6): o JS só fornece IDs. URL e argv
//! saem sempre da config validada. Nenhum shell é envolvido — o processo é
//! criado com `Command::new(argv[0]).args(&argv[1..])`.

use std::process::{Command, Stdio};

use crate::bridge::ErrorCode;
use crate::config::{Config, Item, ItemKind};

fn find_item<'a>(config: &'a Config, id: &str) -> Option<&'a Item> {
    config
        .groups
        .iter()
        .flat_map(|g| g.items.iter())
        .find(|item| item.id == id)
}

/// Resolve o `id` de um item `url` para a URL declarada na config.
///
/// `unknown_id` cobre tanto id inexistente quanto id que aponta para item
/// de outro tipo (a UI nunca deveria pedir `open_url` de um item `app`).
pub fn resolve_url<'a>(config: &'a Config, id: &str) -> Result<&'a str, ErrorCode> {
    match find_item(config, id) {
        Some(item) if item.kind == ItemKind::Url => Ok(&item.target),
        _ => Err(ErrorCode::UnknownId),
    }
}

/// Resolve o `id` de um item `app` para o argv da allowlist.
pub fn resolve_app<'a>(config: &'a Config, id: &str) -> Result<&'a [String], ErrorCode> {
    let item = match find_item(config, id) {
        Some(item) if item.kind == ItemKind::App => item,
        _ => return Err(ErrorCode::UnknownId),
    };
    match config.app_allowlist.get(&item.target) {
        Some(argv) if !argv.is_empty() => Ok(argv),
        _ => Err(ErrorCode::NotAllowlisted),
    }
}

/// Defesa em profundidade: mesmo com a config validada, o host confere o
/// esquema imediatamente antes de abrir a URL.
pub fn url_scheme_allowed(url: &str) -> bool {
    match url::Url::parse(url) {
        Ok(parsed) => matches!(parsed.scheme(), "https" | "mailto"),
        Err(_) => false,
    }
}

/// Executa o argv da allowlist sem shell. Retorna o PID em sucesso.
pub fn spawn_app(argv: &[String]) -> std::io::Result<u32> {
    let (program, args) = argv.split_first().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "argv vazio na allowlist (config deveria ter sido validada)",
        )
    })?;
    let child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(child.id())
}
