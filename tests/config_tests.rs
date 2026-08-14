//! Testes da validação de configuração (docs/PROTOCOL.md).

use qara_desktop::config::{self, ConfigSource, ItemKind};

fn valid_config_json(branding: bool) -> String {
    let branding_block = if branding {
        r#""branding": {
            "kicker": "CLÍNICA DERMATOLÓGICA",
            "wordmark": "QARA",
            "headline": ["Conhecimento que cuida.", "Presença que transforma."],
            "tagline": "Precisão dermatológica com presença humana.",
            "location": "COPACABANA · RIO DE JANEIRO"
        },"#
    } else {
        ""
    };
    format!(
        r#"{{
            "profile": "reception",
            "locale": "pt-BR",
            {branding_block}
            "groups": [
                {{ "id": "attendance", "label": "Atendimento", "items": [
                    {{ "id": "gmail", "label": "Gmail", "type": "url", "target": "https://mail.google.com/" }},
                    {{ "id": "contato", "label": "Contato", "type": "url", "target": "mailto:contato@clinicaqara.com.br" }},
                    {{ "id": "files", "label": "Arquivos", "type": "app", "target": "files" }}
                ]}}
            ],
            "app_allowlist": {{ "files": ["cosmic-files", "--new-window"] }}
        }}"#
    )
}

#[test]
fn config_valida_completa_com_branding() {
    let config = config::parse_and_validate(&valid_config_json(true)).expect("deveria ser válida");
    let branding = config.branding.expect("branding presente");
    assert_eq!(branding.wordmark.as_deref(), Some("QARA"));
    assert_eq!(
        branding.headline.as_deref().map(<[String]>::len),
        Some(2),
        "headline deve manter as duas linhas"
    );
    assert_eq!(config.groups[0].items[2].kind, ItemKind::App);
}

#[test]
fn config_valida_sem_branding() {
    let config = config::parse_and_validate(&valid_config_json(false)).expect("deveria ser válida");
    assert!(config.branding.is_none(), "branding é opcional");
}

#[test]
fn fallback_embutido_e_valido() {
    // Se este teste quebrar, config/shortcuts.example.json foi corrompida.
    let config = config::fallback_config();
    assert!(!config.groups.is_empty());
}

#[test]
fn esquema_http_e_rejeitado() {
    let json =
        valid_config_json(false).replace("https://mail.google.com/", "http://mail.google.com/");
    let err = config::parse_and_validate(&json).unwrap_err();
    assert!(err.contains("http"), "erro deve citar o esquema: {err}");
    assert!(err.contains("gmail"), "erro deve apontar o item: {err}");
}

#[test]
fn esquema_file_e_rejeitado() {
    let json = valid_config_json(false).replace("https://mail.google.com/", "file:///etc/passwd");
    config::parse_and_validate(&json).unwrap_err();
}

#[test]
fn campo_desconhecido_e_rejeitado() {
    let json = valid_config_json(false).replace(
        "\"profile\": \"reception\",",
        "\"profile\": \"reception\", \"telemetria\": true,",
    );
    let err = config::parse_and_validate(&json).unwrap_err();
    assert!(err.contains("telemetria"), "erro deve citar o campo: {err}");
}

#[test]
fn tipo_desconhecido_e_rejeitado() {
    let json = valid_config_json(false).replace("\"type\": \"app\"", "\"type\": \"shell\"");
    config::parse_and_validate(&json).unwrap_err();
}

#[test]
fn app_fora_da_allowlist_e_rejeitado() {
    let json =
        valid_config_json(false).replace("\"target\": \"files\"", "\"target\": \"terminal\"");
    let err = config::parse_and_validate(&json).unwrap_err();
    assert!(
        err.contains("app_allowlist"),
        "erro deve citar a allowlist: {err}"
    );
    assert!(err.contains("terminal"), "erro deve citar o alvo: {err}");
}

#[test]
fn ids_de_grupo_duplicados_sao_rejeitados() {
    let dup = valid_config_json(false).replace(
        r#""groups": ["#,
        r#""groups": [
            { "id": "attendance", "label": "Duplicado", "items": [] },"#,
    );
    let err = config::parse_and_validate(&dup).unwrap_err();
    assert!(
        err.contains("duplicado"),
        "erro deve citar duplicidade: {err}"
    );
    assert!(err.contains("attendance"), "erro deve citar o id: {err}");
}

#[test]
fn ids_de_item_duplicados_sao_rejeitados() {
    let dup = valid_config_json(false).replace(
        r#"{ "id": "files", "label": "Arquivos", "type": "app", "target": "files" }"#,
        r#"{ "id": "gmail", "label": "Arquivos", "type": "app", "target": "files" }"#,
    );
    let err = config::parse_and_validate(&dup).unwrap_err();
    assert!(
        err.contains("duplicado"),
        "erro deve citar duplicidade: {err}"
    );
}

#[test]
fn argv_vazio_na_allowlist_e_rejeitado() {
    let json = valid_config_json(false).replace(r#"["cosmic-files", "--new-window"]"#, "[]");
    let err = config::parse_and_validate(&json).unwrap_err();
    assert!(err.contains("argv"), "erro deve citar argv: {err}");
    assert!(err.contains("files"), "erro deve citar a chave: {err}");
}

#[test]
fn executavel_vazio_na_allowlist_e_rejeitado() {
    let json =
        valid_config_json(false).replace(r#"["cosmic-files", "--new-window"]"#, r#"["", "abc"]"#);
    config::parse_and_validate(&json).unwrap_err();
}

#[test]
fn campo_obrigatorio_ausente_gera_erro_legivel() {
    let json = valid_config_json(false).replace("\"locale\": \"pt-BR\",", "");
    let err = config::parse_and_validate(&json).unwrap_err();
    assert!(
        err.contains("locale"),
        "erro deve citar o campo faltante: {err}"
    );
}

#[test]
fn load_from_arquivo_invalido_cai_no_fallback() {
    let dir = std::env::temp_dir().join(format!("qara-test-config-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("config.json");
    std::fs::write(&path, "{ isso nao é json").unwrap();
    let loaded = config::load_from(&path);
    assert_eq!(loaded.source, ConfigSource::Fallback);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_from_arquivo_ausente_cai_no_fallback() {
    let loaded = config::load_from(std::path::Path::new("/nao/existe/config.json"));
    assert_eq!(loaded.source, ConfigSource::Fallback);
}
