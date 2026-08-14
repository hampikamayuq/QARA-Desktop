//! Testes da resolução de IDs (allowlist) do launcher.

use std::collections::BTreeMap;

use qara_desktop::bridge::ErrorCode;
use qara_desktop::config::{Config, Group, Item, ItemKind};
use qara_desktop::launcher::{resolve_app, resolve_url, url_scheme_allowed};

fn test_config() -> Config {
    Config {
        profile: "reception".into(),
        locale: "pt-BR".into(),
        branding: None,
        groups: vec![Group {
            id: "g1".into(),
            label: "Grupo".into(),
            items: vec![
                Item {
                    id: "gmail".into(),
                    label: "Gmail".into(),
                    kind: ItemKind::Url,
                    target: "https://mail.google.com/".into(),
                },
                Item {
                    id: "files".into(),
                    label: "Arquivos".into(),
                    kind: ItemKind::App,
                    target: "files".into(),
                },
                Item {
                    id: "orfao".into(),
                    label: "Sem allowlist".into(),
                    kind: ItemKind::App,
                    target: "nao-listado".into(),
                },
            ],
        }],
        app_allowlist: BTreeMap::from([("files".to_string(), vec!["cosmic-files".to_string()])]),
    }
}

#[test]
fn resolve_url_devolve_url_da_config() {
    let config = test_config();
    assert_eq!(
        resolve_url(&config, "gmail"),
        Ok("https://mail.google.com/")
    );
}

#[test]
fn resolve_url_id_inexistente_e_unknown_id() {
    let config = test_config();
    assert_eq!(
        resolve_url(&config, "nao-existe"),
        Err(ErrorCode::UnknownId)
    );
}

#[test]
fn resolve_url_de_item_app_e_unknown_id() {
    let config = test_config();
    assert_eq!(resolve_url(&config, "files"), Err(ErrorCode::UnknownId));
}

#[test]
fn resolve_app_devolve_argv_da_allowlist() {
    let config = test_config();
    assert_eq!(
        resolve_app(&config, "files"),
        Ok(&["cosmic-files".to_string()][..])
    );
}

#[test]
fn resolve_app_id_inexistente_e_unknown_id() {
    let config = test_config();
    assert_eq!(resolve_app(&config, "nada"), Err(ErrorCode::UnknownId));
}

#[test]
fn resolve_app_de_item_url_e_unknown_id() {
    let config = test_config();
    assert_eq!(resolve_app(&config, "gmail"), Err(ErrorCode::UnknownId));
}

#[test]
fn resolve_app_fora_da_allowlist_e_not_allowlisted() {
    // Config montada à mão simulando estado inconsistente pós-validação:
    // item app cujo target não está na allowlist.
    let config = test_config();
    assert_eq!(
        resolve_app(&config, "orfao"),
        Err(ErrorCode::NotAllowlisted)
    );
}

#[test]
fn esquemas_permitidos_sao_somente_https_e_mailto() {
    assert!(url_scheme_allowed("https://clinicaqara.com.br/"));
    assert!(url_scheme_allowed("mailto:contato@clinicaqara.com.br"));
    assert!(!url_scheme_allowed("http://example.com/"));
    assert!(!url_scheme_allowed("file:///etc/passwd"));
    assert!(!url_scheme_allowed("javascript:alert(1)"));
    assert!(!url_scheme_allowed("não é uma url"));
}
