//! Testes do parse de mensagens da bridge e da montagem de respostas.

use qara_desktop::bridge::{
    self, Action, ErrorCode, Response, bootstrap_script, js_string_escape, parse_message,
    response_script,
};

#[test]
fn open_url_valida() {
    let req = parse_message(r#"{"v":1,"req_id":"r-17","action":"open_url","id":"gmail"}"#)
        .expect("mensagem válida");
    assert_eq!(req.req_id, "r-17");
    assert_eq!(req.action, Action::OpenUrl { id: "gmail".into() });
}

#[test]
fn launch_app_valida() {
    let req = parse_message(r#"{"v":1,"req_id":"r-2","action":"launch_app","id":"files"}"#)
        .expect("mensagem válida");
    assert_eq!(req.action, Action::LaunchApp { id: "files".into() });
}

#[test]
fn get_system_info_valida() {
    let req = parse_message(r#"{"v":1,"req_id":"r-3","action":"get_system_info"}"#)
        .expect("mensagem válida");
    assert_eq!(req.action, Action::GetSystemInfo);
}

#[test]
fn set_ui_preference_valida() {
    let req = parse_message(
        r#"{"v":1,"req_id":"r-4","action":"set_ui_preference","key":"compact","value":true}"#,
    )
    .expect("mensagem válida");
    assert_eq!(
        req.action,
        Action::SetUiPreference {
            key: "compact".into(),
            value: serde_json::Value::Bool(true),
        }
    );
}

#[test]
fn json_malformado_e_invalid_message() {
    let failure = parse_message("{isso nao é json").unwrap_err();
    assert_eq!(failure.code, ErrorCode::InvalidMessage);
    assert_eq!(failure.req_id, None);
}

#[test]
fn nao_objeto_e_invalid_message() {
    let failure = parse_message(r#""apenas uma string""#).unwrap_err();
    assert_eq!(failure.code, ErrorCode::InvalidMessage);
}

#[test]
fn acao_desconhecida_e_unknown_action_com_req_id() {
    let failure = parse_message(r#"{"v":1,"req_id":"r-9","action":"run_shell","cmd":"rm -rf /"}"#)
        .unwrap_err();
    assert_eq!(failure.code, ErrorCode::UnknownAction);
    assert_eq!(failure.req_id.as_deref(), Some("r-9"));
}

#[test]
fn req_id_ausente_e_invalid_message() {
    let failure = parse_message(r#"{"v":1,"action":"get_system_info"}"#).unwrap_err();
    assert_eq!(failure.code, ErrorCode::InvalidMessage);
    assert_eq!(failure.req_id, None);
}

#[test]
fn versao_ausente_ou_errada_e_invalid_message() {
    let failure = parse_message(r#"{"req_id":"r-1","action":"get_system_info"}"#).unwrap_err();
    assert_eq!(failure.code, ErrorCode::InvalidMessage);
    assert_eq!(failure.req_id.as_deref(), Some("r-1"));

    let failure =
        parse_message(r#"{"v":2,"req_id":"r-1","action":"get_system_info"}"#).unwrap_err();
    assert_eq!(failure.code, ErrorCode::InvalidMessage);
}

/// Garantia central de segurança: NENHUMA ação aceita URL/comando vindos da
/// mensagem. Campos extras são rejeitados por deny_unknown_fields.
#[test]
fn nenhuma_acao_aceita_alvo_vindo_da_mensagem() {
    let casos = [
        r#"{"v":1,"req_id":"r-1","action":"open_url","id":"gmail","url":"https://mal.example/"}"#,
        r#"{"v":1,"req_id":"r-2","action":"launch_app","id":"files","command":"bash"}"#,
        r#"{"v":1,"req_id":"r-3","action":"launch_app","id":"files","argv":["sh","-c","id"]}"#,
        r#"{"v":1,"req_id":"r-4","action":"get_system_info","exec":"id"}"#,
        r#"{"v":1,"req_id":"r-5","action":"set_ui_preference","key":"compact","value":true,"target":"x"}"#,
    ];
    for raw in casos {
        let failure = parse_message(raw).expect_err(raw);
        assert_eq!(failure.code, ErrorCode::InvalidMessage, "mensagem: {raw}");
        assert!(failure.req_id.is_some(), "req_id recuperado para responder");
    }
}

#[test]
fn parametro_obrigatorio_ausente_e_invalid_message() {
    let failure = parse_message(r#"{"v":1,"req_id":"r-1","action":"open_url"}"#).unwrap_err();
    assert_eq!(failure.code, ErrorCode::InvalidMessage);
}

#[test]
fn resposta_ok_serializa_sem_campos_nulos() {
    let script = response_script(&Response::ok("r-1", None));
    assert!(script.contains("QaraBridge.onHostResponse"));
    assert!(script.contains(r#"\"ok\":true"#));
    assert!(
        !script.contains("data"),
        "data ausente não deve ser emitido"
    );
}

#[test]
fn resposta_de_erro_usa_codigo_exato() {
    let script = response_script(&Response::err("r-2", ErrorCode::NotAllowlisted));
    assert!(script.contains("not_allowlisted"));
    assert!(script.contains(r#"\"ok\":false"#));
}

#[test]
fn codigos_de_erro_batem_com_o_protocolo() {
    assert_eq!(ErrorCode::InvalidMessage.as_str(), "invalid_message");
    assert_eq!(ErrorCode::UnknownAction.as_str(), "unknown_action");
    assert_eq!(ErrorCode::UnknownId.as_str(), "unknown_id");
    assert_eq!(ErrorCode::NotAllowlisted.as_str(), "not_allowlisted");
    assert_eq!(ErrorCode::LaunchFailed.as_str(), "launch_failed");
    assert_eq!(ErrorCode::InvalidPreference.as_str(), "invalid_preference");
}

#[test]
fn escape_de_string_js_cobre_casos_perigosos() {
    assert_eq!(js_string_escape(r#"a"b"#), r#"a\"b"#);
    assert_eq!(js_string_escape(r"a\b"), r"a\\b");
    assert_eq!(js_string_escape("a\nb"), r"a\nb");
    assert_eq!(js_string_escape("a\u{2028}b"), "a\\u2028b");
    assert_eq!(js_string_escape("a\u{0007}b"), "a\\u0007b");
}

#[test]
fn bootstrap_script_injeta_estado_do_host() {
    let script = bootstrap_script(r#"{"profile":"reception"}"#, r#"{"compact":false}"#);
    assert!(script.contains("window.__QARA_HOST__ = true"));
    assert!(script.contains("window.__QARA_CONFIG__ = JSON.parse("));
    assert!(script.contains("window.__QARA_PREFS__ = JSON.parse("));
    assert!(script.contains(r#"\"profile\":\"reception\""#));
}

#[test]
fn versao_do_protocolo_e_1() {
    assert_eq!(bridge::PROTOCOL_VERSION, 1);
}
