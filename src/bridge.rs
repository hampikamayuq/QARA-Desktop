//! Parse das mensagens UI → host e montagem das respostas host → UI.
//!
//! Contrato em `docs/PROTOCOL.md`. Mensagem desconhecida ou malformada é
//! sempre um ERRO respondido (`invalid_message`/`unknown_action`), nunca
//! ignorada em silêncio. Nenhuma ação aceita URL ou executável vindos do JS:
//! o envelope é achatado e qualquer campo extra é rejeitado.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Versão do protocolo aceita pelo host.
pub const PROTOCOL_VERSION: u64 = 1;

/// Códigos de erro fechados do protocolo (docs/PROTOCOL.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    InvalidMessage,
    UnknownAction,
    UnknownId,
    NotAllowlisted,
    LaunchFailed,
    InvalidPreference,
}

impl ErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::InvalidMessage => "invalid_message",
            ErrorCode::UnknownAction => "unknown_action",
            ErrorCode::UnknownId => "unknown_id",
            ErrorCode::NotAllowlisted => "not_allowlisted",
            ErrorCode::LaunchFailed => "launch_failed",
            ErrorCode::InvalidPreference => "invalid_preference",
        }
    }
}

/// Ações fechadas do protocolo. Os parâmetros carregam somente IDs/keys —
/// nunca URLs ou executáveis.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    OpenUrl { id: String },
    LaunchApp { id: String },
    GetSystemInfo,
    SetUiPreference { key: String, value: Value },
}

/// Mensagem válida da UI.
#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    pub req_id: String,
    pub action: Action,
}

/// Falha de parse. `req_id` é recuperado quando possível para a UI
/// conseguir correlacionar a resposta de erro.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseFailure {
    pub req_id: Option<String>,
    pub code: ErrorCode,
    /// Detalhe técnico para o log (pt-BR). Nunca vai para a UI.
    pub detail: String,
}

impl ParseFailure {
    fn invalid(req_id: Option<String>, detail: impl Into<String>) -> Self {
        ParseFailure {
            req_id,
            code: ErrorCode::InvalidMessage,
            detail: detail.into(),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IdParams {
    id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EmptyParams {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrefParams {
    key: String,
    value: Value,
}

/// Faz o parse estrito de uma mensagem crua vinda do WebView.
///
/// O envelope (`v`, `req_id`, `action`) é validado à mão e os campos
/// restantes são desserializados com `deny_unknown_fields` — assim uma
/// mensagem `open_url` com um campo extra `url` é rejeitada, garantindo
/// que o JS jamais forneça alvo de navegação/execução.
pub fn parse_message(raw: &str) -> Result<Request, ParseFailure> {
    let value: Value = serde_json::from_str(raw)
        .map_err(|e| ParseFailure::invalid(None, format!("JSON malformado: {e}")))?;
    let Value::Object(mut fields) = value else {
        return Err(ParseFailure::invalid(
            None,
            "mensagem deve ser um objeto JSON",
        ));
    };

    // req_id primeiro: se existir, aproveitamos até nos erros seguintes.
    let req_id = match fields.remove("req_id") {
        Some(Value::String(s)) if !s.is_empty() => s,
        Some(_) => {
            return Err(ParseFailure::invalid(
                None,
                "campo \"req_id\" deve ser string não vazia",
            ));
        }
        None => {
            return Err(ParseFailure::invalid(
                None,
                "campo obrigatório \"req_id\" ausente",
            ));
        }
    };

    match fields.remove("v") {
        Some(Value::Number(n)) if n.as_u64() == Some(PROTOCOL_VERSION) => {}
        Some(other) => {
            return Err(ParseFailure::invalid(
                Some(req_id),
                format!("versão de protocolo não suportada: {other}"),
            ));
        }
        None => {
            return Err(ParseFailure::invalid(
                Some(req_id),
                "campo obrigatório \"v\" ausente",
            ));
        }
    }

    let action_name = match fields.remove("action") {
        Some(Value::String(s)) => s,
        Some(_) => {
            return Err(ParseFailure::invalid(
                Some(req_id),
                "campo \"action\" deve ser string",
            ));
        }
        None => {
            return Err(ParseFailure::invalid(
                Some(req_id),
                "campo obrigatório \"action\" ausente",
            ));
        }
    };

    let action = match action_name.as_str() {
        "open_url" => Action::OpenUrl {
            id: parse_params::<IdParams>(fields, &req_id, "open_url")?.id,
        },
        "launch_app" => Action::LaunchApp {
            id: parse_params::<IdParams>(fields, &req_id, "launch_app")?.id,
        },
        "get_system_info" => {
            parse_params::<EmptyParams>(fields, &req_id, "get_system_info")?;
            Action::GetSystemInfo
        }
        "set_ui_preference" => {
            let p = parse_params::<PrefParams>(fields, &req_id, "set_ui_preference")?;
            Action::SetUiPreference {
                key: p.key,
                value: p.value,
            }
        }
        other => {
            return Err(ParseFailure {
                req_id: Some(req_id),
                code: ErrorCode::UnknownAction,
                detail: format!("ação desconhecida: \"{other}\""),
            });
        }
    };

    Ok(Request { req_id, action })
}

fn parse_params<T: for<'de> Deserialize<'de>>(
    rest: Map<String, Value>,
    req_id: &str,
    action: &str,
) -> Result<T, ParseFailure> {
    serde_json::from_value(Value::Object(rest)).map_err(|e| {
        ParseFailure::invalid(
            Some(req_id.to_string()),
            format!("parâmetros inválidos para \"{action}\": {e}"),
        )
    })
}

/// Resposta host → UI, sempre entregue via `QaraBridge.onHostResponse`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Response {
    pub req_id: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<&'static str>,
}

impl Response {
    pub fn ok(req_id: impl Into<String>, data: Option<Value>) -> Self {
        Response {
            req_id: req_id.into(),
            ok: true,
            data,
            error: None,
        }
    }

    pub fn err(req_id: impl Into<String>, code: ErrorCode) -> Self {
        Response {
            req_id: req_id.into(),
            ok: false,
            data: None,
            error: Some(code.as_str()),
        }
    }
}

/// Escapa `s` como literal de string JS entre aspas duplas (sem as aspas).
/// Cobre aspas, contrabarra, controles e U+2028/U+2029 (válidos em JSON,
/// inválidos em fonte JS antigo — escapar é sempre seguro).
pub fn js_string_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Gera o JS que entrega uma resposta à UI. O payload viaja como string
/// escapada + `JSON.parse`, eliminando qualquer risco de injeção.
pub fn response_script(response: &Response) -> String {
    let json = serde_json::to_string(response)
        .expect("serialização de Response nunca falha (tipos fechados)");
    format!(
        "if (window.QaraBridge && typeof window.QaraBridge.onHostResponse === 'function') {{ \
         window.QaraBridge.onHostResponse(JSON.parse(\"{}\")); }}",
        js_string_escape(&json)
    )
}

/// Gera o user script de `document-start` que injeta o estado do host
/// (`__QARA_HOST__`, `__QARA_CONFIG__`, `__QARA_PREFS__`), conforme
/// docs/PROTOCOL.md. Config e prefs viajam como JSON escapado + `JSON.parse`.
pub fn bootstrap_script(config_json: &str, prefs_json: &str) -> String {
    format!(
        "window.__QARA_HOST__ = true;\n\
         window.__QARA_CONFIG__ = JSON.parse(\"{}\");\n\
         window.__QARA_PREFS__ = JSON.parse(\"{}\");",
        js_string_escape(config_json),
        js_string_escape(prefs_json)
    )
}
