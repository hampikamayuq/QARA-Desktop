# Protocolo Host ↔ UI (v1)

Contrato entre o host nativo (Rust/WebKitGTK 6) e a UI (`ui/`). Qualquer mudança aqui exige atualizar os dois lados e este documento.

## Carregamento da UI

- O host serve a UI por custom scheme: `qara://ui/index.html`. Assets relativos resolvem sob `qara://ui/`.
- Navegação para qualquer URI fora de `qara://` é **negada** dentro da WebView.
- No preview standalone (`python3 -m http.server`), não há host; a UI deve detectar isso e degradar (ver "Modo preview").

## Injeção de estado (host → UI, antes do load)

O host injeta um user script em `document-start` definindo:

```js
window.__QARA_HOST__ = true;
window.__QARA_CONFIG__ = { /* config validada, mesmo shape do config.json */ };
window.__QARA_PREFS__ = { compact: false /* preferências persistidas */ };
```

A UI lê essas globais no `DOMContentLoaded`. Se `window.__QARA_HOST__` não existir, a UI está em modo preview.

## Mensagens UI → host

Canal: `window.webkit.messageHandlers.qara.postMessage(JSON.stringify(msg))`.

A UI usa sempre o wrapper `QaraBridge.invoke(action, params)` (definido em `app.js`), que gera `req_id` único e serializa:

```json
{ "v": 1, "req_id": "r-17", "action": "open_url", "id": "gmail" }
```

Ações fechadas (qualquer outra é erro `unknown_action`):

| action | params | efeito |
|--------|--------|--------|
| `open_url` | `id` | Abre no navegador padrão a URL do item `id` da config. A UI **nunca** envia a URL. |
| `launch_app` | `id` | Executa o argv da `app_allowlist` para o item `id`. A UI **nunca** envia executável. |
| `get_system_info` | — | Responde com `{ hostname, profile, version }` no campo `data`. |
| `set_ui_preference` | `key`, `value` | Persiste preferência não sensível. Keys aceitas: `compact` (bool). |

## Respostas host → UI

O host avalia:

```js
window.QaraBridge.onHostResponse({ req_id: "r-17", ok: true, data: null });
window.QaraBridge.onHostResponse({ req_id: "r-17", ok: false, error: "not_allowlisted" });
```

Códigos de erro: `invalid_message`, `unknown_action`, `unknown_id`, `not_allowlisted`, `launch_failed`, `invalid_preference`.

A UI mostra toast discreto em erro; nunca loga conteúdo sensível.

## Shape da config (config.json)

```json
{
  "profile": "reception",
  "locale": "pt-BR",
  "branding": {
    "kicker": "CLÍNICA DERMATOLÓGICA",
    "wordmark": "QARA",
    "headline": ["Conhecimento que cuida.", "Presença que transforma."],
    "tagline": "Precisão dermatológica com presença humana.",
    "location": "COPACABANA · RIO DE JANEIRO"
  },
  "groups": [
    { "id": "attendance", "label": "Atendimento", "items": [
      { "id": "gmail", "label": "Gmail", "type": "url", "target": "https://mail.google.com/" },
      { "id": "files", "label": "Arquivos", "type": "app", "target": "files" }
    ]}
  ],
  "app_allowlist": { "files": ["cosmic-files"] }
}
```

Regras de validação (host, erro legível; campos desconhecidos rejeitados):

- `type` ∈ {`url`, `app`};
- item `url`: `target` obrigatório, esquema `https:` ou `mailto:` apenas;
- item `app`: `target` deve ser chave existente em `app_allowlist`;
- IDs de grupos e itens únicos;
- `app_allowlist`: valores são argv não vazio; executado sem shell (`Command::new(argv[0]).args(...)`).

`branding` é opcional; ausente, a UI usa os textos padrão atuais.

## Preferências persistidas

Arquivo: `~/.config/qara-desktop/state.json` (`{ "compact": true }`). Somente keys da tabela acima; nada sensível.

## Modo preview (sem host)

- `QaraBridge.invoke('open_url', ...)` → `window.open(target)` usando a config de fallback local;
- `launch_app` → toast informativo;
- `set_ui_preference` → `localStorage`;
- config: fallback embutido em `app.js`.
