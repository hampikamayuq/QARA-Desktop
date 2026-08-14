//! Testes das preferências persistidas (state.json).

use qara_desktop::bridge::ErrorCode;
use qara_desktop::prefs::{self, Prefs};
use serde_json::json;

#[test]
fn compact_bool_e_aceito() {
    let mut prefs = Prefs::default();
    prefs::apply(&mut prefs, "compact", &json!(true)).expect("compact=true é válido");
    assert!(prefs.compact);
    prefs::apply(&mut prefs, "compact", &json!(false)).expect("compact=false é válido");
    assert!(!prefs.compact);
}

#[test]
fn key_desconhecida_e_invalid_preference() {
    let mut prefs = Prefs::default();
    let err = prefs::apply(&mut prefs, "tema", &json!("escuro")).unwrap_err();
    assert_eq!(err, ErrorCode::InvalidPreference);
}

#[test]
fn valor_de_tipo_errado_e_invalid_preference() {
    let mut prefs = Prefs::default();
    let err = prefs::apply(&mut prefs, "compact", &json!("sim")).unwrap_err();
    assert_eq!(err, ErrorCode::InvalidPreference);
    assert!(!prefs.compact, "preferência não deve mudar em erro");
}

#[test]
fn roundtrip_atomico_de_gravacao_e_leitura() {
    let dir = std::env::temp_dir().join(format!("qara-test-prefs-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("state.json");

    let prefs = Prefs { compact: true };
    prefs::save_atomic(&path, &prefs).expect("gravação atômica");
    assert_eq!(prefs::load(&path), prefs);

    // Nenhum arquivo temporário deve sobrar após a gravação.
    let leftovers: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().contains("tmp"))
        .collect();
    assert!(leftovers.is_empty(), "temporários restantes: {leftovers:?}");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn state_json_invalido_cai_no_padrao() {
    let dir = std::env::temp_dir().join(format!("qara-test-prefs-bad-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("state.json");
    std::fs::write(&path, "{ \"compact\": \"talvez\" }").unwrap();
    assert_eq!(prefs::load(&path), Prefs::default());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn state_json_ausente_cai_no_padrao() {
    assert_eq!(
        prefs::load(std::path::Path::new("/nao/existe/state.json")),
        Prefs::default()
    );
}

#[test]
fn key_desconhecida_no_arquivo_e_rejeitada_no_parse() {
    let dir = std::env::temp_dir().join(format!("qara-test-prefs-extra-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("state.json");
    std::fs::write(&path, "{ \"compact\": true, \"senha\": \"x\" }").unwrap();
    // deny_unknown_fields: arquivo com key estranha volta ao padrão.
    assert_eq!(prefs::load(&path), Prefs::default());
    std::fs::remove_dir_all(&dir).unwrap();
}
