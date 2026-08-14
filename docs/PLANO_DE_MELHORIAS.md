# QARA Desktop — Plano de Melhorias

> **Status (2026-08-14):** P0, P1 e os itens de código de P2/P3 foram implementados neste repositório (ver histórico da branch). Permanecem pendentes apenas os itens que exigem a máquina Pop!_OS/COSMIC real: preencher `docs/ENVIRONMENT.md` via `scripts/diagnose.sh`, validar o comportamento layer-shell (Background, sem Alt+Tab, hotplug de monitores, escala 100–200%), medir CPU/memória em idle (SPEC §8/§12) e gerar o `.deb` com `cargo deb`. Widgets agregados (Fase 5) seguem como trabalho futuro.

Data: 2026-08-14
Base analisada: `qara-desktop-starter` (SPEC v0.1, protótipo `ui/`, esqueleto `src/main.rs`, scripts e configuração).

Este documento organiza o que falta e o que pode melhorar, em ordem de prioridade. Cada item indica o arquivo afetado e o critério de "pronto". As prioridades:

- **P0 — Fundação**: corrige problemas reais já presentes e prepara o repositório.
- **P1 — MVP funcional**: torna o produto utilizável (Fases 1–3 da SPEC).
- **P2 — Robustez**: layer-shell em produção, testes, empacotamento.
- **P3 — Polimento e futuro**: perfis, preferências, widgets agregados.

---

## Diagnóstico do estado atual

O starter está bem encaminhado no que importa: a SPEC é clara, o modelo de segurança (allowlist, sem `sh -c`, CSP na UI) está correto desde o início, e o protótipo web já usa os tokens oficiais QARA com acessibilidade básica (foco visível, `prefers-reduced-motion`, hit-area adequada).

O que **não existe ainda**:

- O host Rust não exibe a UI — `src/main.rs` mostra apenas um `gtk::Label` placeholder; não há WebView, bridge, carregamento de config nem logs.
- A UI usa somente o `fallbackConfig` embutido em `ui/app.js`; o arquivo `config/shortcuts.example.json` nunca é lido por ninguém (o CSP com `connect-src 'none'` inclusive impede `fetch` — a config precisa vir do host).
- O protocolo da bridge (`window.qaraNative.invoke`) é uma convenção inventada no JS que não corresponde ao mecanismo real do WebKitGTK (`window.webkit.messageHandlers.<nome>.postMessage`). Precisa ser definido e implementado dos dois lados.
- Não há CI, testes, LICENSE (o `Cargo.toml` declara MIT mas o arquivo não existe), nem `docs/ENVIRONMENT.md`.

Problemas concretos encontrados:

1. **`Cargo.toml` — toolchain**: `edition = "2024"` exige Rust ≥ 1.85. O `rustc` dos repositórios do Pop!_OS/Ubuntu 24.04 é mais antigo. Ou se documenta rustup como caminho obrigatório, ou se rebaixa para `edition = "2021"`. As versões dos crates estão corretas e mutuamente compatíveis (gtk4 0.11.x, gtk4-layer-shell 0.8.x e webkit6 0.6.x, todos sobre gtk4-rs 0.11 — verificado em crates.io), mas falta fixar as *feature flags* de versão (`v4_14` etc.) conforme o GTK do sistema alvo.
2. **`scripts/install-deps-popos.sh` — bug real**: se nenhum pacote estiver disponível, `missing` fica vazio e `sudo apt-get install -y "${missing[@]}"` aborta com `set -u` (expansão de array vazio). Além disso a variável se chama `missing` mas contém "disponíveis", não "faltantes" — o script reinstala pacotes já presentes.
3. **`ui/app.js` — relógio**: `setInterval(..., 30_000)` faz o minuto virar com até 30 s de atraso e acorda o processo 2× por minuto sem necessidade. Alinhar o timer à virada do minuto resolve os dois pontos.
4. **`ui/app.js` — `window.open`**: no WebKitGTK, `window.open` cria uma nova WebView (que a SPEC manda bloquear), não abre o navegador do sistema. Só funciona no preview via `python3 -m http.server`. O caminho real é sempre a bridge.
5. **`install/qara-desktop.desktop`**: sem `Icon=`, sem instrução de instalação em `~/.config/autostart/`, e `Exec=/usr/local/bin/qara-desktop` fixo. Falta também documentar como ativar/desativar (critério de aceite da SPEC).
6. **`src/main.rs` — `KeyboardMode::None`**: correto para um background, mas torna inúteis os estilos `:focus-visible` da UI (nunca haverá foco por teclado em produção). Registrar a decisão (ou avaliar `OnDemand`) em `docs/DECISIONS.md`.
7. **`ui/index.html`**: textos institucionais ("CLÍNICA DERMATOLÓGICA", "COPACABANA · RIO DE JANEIRO", tagline) estão hardcoded; deveriam vir da config para servir múltiplas estações/unidades sem editar HTML.

---

## P0 — Fundação (antes de qualquer feature)

| # | Item | Arquivos | Pronto quando |
|---|------|----------|---------------|
| 0.1 | Adicionar `LICENSE` (MIT, como declarado no Cargo.toml) | `LICENSE` | Arquivo presente |
| 0.2 | Decidir toolchain: documentar rustup + `rust-toolchain.toml` (ou rebaixar para edition 2021) | `rust-toolchain.toml`, `README.md` | `cargo check` funciona na máquina alvo com instrução reproduzível |
| 0.3 | Corrigir `install-deps-popos.sh` (array vazio + semântica de `missing`) e instalar só o que falta | `scripts/install-deps-popos.sh` | Script idempotente; roda 2× sem erro |
| 0.4 | CI mínima no GitHub Actions: `cargo fmt --check`, `clippy -D warnings`, `cargo test` (com libs GTK/WebKit instaladas no runner) | `.github/workflows/ci.yml` | CI verde no push |
| 0.5 | Rodar `scripts/diagnose.sh` na máquina alvo e registrar em `docs/ENVIRONMENT.md` (versões GTK/WebKit/layer-shell, escala, monitores) | `docs/ENVIRONMENT.md` | Versões reais documentadas; feature flags do gtk4 fixadas de acordo |
| 0.6 | `.editorconfig` + `rustfmt.toml` para consistência | raiz | Presentes |

## P1 — MVP funcional (Fases 1–3 da SPEC)

### 1.1 WebView exibindo a UI (`--windowed`)

- Substituir o `gtk::Label` por `webkit6::WebView`.
- Carregar a UI por **custom scheme local** (`qara://ui/index.html`) em vez de `file://` — evita expor o filesystem, permite CSP estável e resolve caminho relativo dos assets.
- Endurecer o `WebKitSettings`: DevTools desligado fora de builds de desenvolvimento, `javascript-can-open-windows-automatically = false`, popups bloqueados, `WebsiteDataManager` efêmero (nada de cache/cookies persistentes — a UI é local).
- Interceptar `decide-policy`: qualquer navegação para fora de `qara://` é negada (e, se for `https:` de item conhecido, delegada ao navegador padrão via bridge).

**Pronto quando:** `cargo run -- --windowed` mostra a UI com relógio funcionando; nenhuma navegação externa acontece dentro da WebView.

### 1.2 Config real: carregar, validar, injetar

- Structs `serde` com validação estrita (`deny_unknown_fields`): `Config { profile, locale, branding, groups[], app_allowlist }`.
- Ordem de busca: `~/.config/qara-desktop/config.json` → fallback empacotado (`config/shortcuts.example.json` embutido via `include_str!`).
- Validações que produzem **erro legível** (critério da SPEC): URL só `https:`/`mailto:`, `type` ∈ {url, app}, todo item `app` precisa existir em `app_allowlist`, IDs únicos.
- O host injeta a config na UI (via `WebView::evaluate_javascript` chamando `window.qaraDesktop.setConfig(...)` ou user script no load). Remover a dependência do `fallbackConfig` do `app.js` (mantê-lo apenas para o preview standalone).
- Mover os textos institucionais do HTML para a seção `branding` da config (item 7 do diagnóstico).

**Pronto quando:** editar o JSON muda a UI; JSON inválido gera mensagem clara no log e a UI sobe com o fallback + aviso discreto.

### 1.3 Bridge nativa tipada

Definir o protocolo de verdade (hoje só existe o lado JS, e incompatível com WebKitGTK):

- **JS → host**: `UserContentManager::register_script_message_handler("qara")`; a UI chama `window.webkit.messageHandlers.qara.postMessage({ v: 1, req_id, action, id })`. Manter um wrapper fino `window.qaraNative.invoke()` no `app.js` para o preview continuar funcionando sem host.
- **Ações fechadas** (enum `serde` com `#[serde(tag = "action")]` — mensagem desconhecida é erro, nunca ignorada silenciosamente): `open_url { id }`, `launch_app { id }`, `get_system_info {}`, `set_ui_preference { key, value }`.
- **Resolução sempre por ID**: o JS nunca envia URL nem nome de executável; o host resolve o ID contra a config validada. `open_url` → `gio::AppInfo::launch_default_for_uri` (ou `xdg-open` via `Command` com argumento único). `launch_app` → `std::process::Command::new(argv[0]).args(&argv[1..])` — sem shell, como já exige a SPEC.
- **Host → JS**: resposta `{ req_id, ok, error? }` para a UI dar feedback (toast de erro quando o app não abre — hoje o erro morreria em silêncio).

**Pronto quando:** clicar em atalho web abre o navegador padrão; atalho `app` fora da allowlist é recusado com log + toast; nenhum teste consegue fazer o host executar string arbitrária vinda do JS.

### 1.4 Logs estruturados

- `tracing` + `tracing-appender` escrevendo em `~/.local/state/qara-desktop/qara-desktop.log` (XDG state dir), com rotação simples e espelho no stderr em modo dev.
- Registrar exatamente o que a SPEC §10 pede (startup, ambiente, layer-shell, falhas de URL/app/config, panics via hook) e nada de conteúdo clínico/credenciais.

**Pronto quando:** cada falha simulada (config quebrada, app inexistente) aparece no log com contexto suficiente para diagnóstico remoto.

## P2 — Robustez e produção

| # | Item | Notas |
|---|------|-------|
| 2.1 | **Layer-shell real no COSMIC** | Já esboçado em `main.rs`; validar na máquina alvo: sem Alt+Tab, sem roubar foco, sobrevive a mudança de resolução/escala. Fallback `--windowed` com erro claro (já previsto). |
| 2.2 | **Múltiplos monitores** | Uma janela layer-shell por `gdk::Monitor`; reagir a `monitors` changed (hotplug). Decidir se a UI replica ou se só o monitor primário recebe o launcher. |
| 2.3 | **Autostart de verdade** | Corrigir o `.desktop` (Icon, path resolvido na instalação), instalar em `~/.config/autostart/`, e um `make install`/script que copia binário + `.desktop` + config exemplo. Documentar como desativar. |
| 2.4 | **Testes** | Unidade: validação de config (casos inválidos), parsing de mensagens da bridge, resolução de allowlist. A lógica de bridge/config deve viver em módulos separados de `main.rs` (`config.rs`, `bridge.rs`, `launcher.rs`) para ser testável sem GTK. |
| 2.5 | **Empacotamento `.deb`** | `cargo-deb` com dependências declaradas (libgtk-4, libwebkitgtk-6.0, libgtk4-layer-shell). Facilita replicar nas estações da clínica. |
| 2.6 | **Watchdog de config** | `gio::FileMonitor` na config: editar o JSON atualiza a UI sem reiniciar (qualidade de vida para ajustar estações). |

## P3 — Polimento e futuro

- **Perfis por estação** (`reception`/`clinical`/`admin`): a config já prevê `profile`; adicionar `profiles/<nome>.json` de exemplo e seletor na instalação.
- **Preferências persistidas**: `set_ui_preference` gravando (ex.: modo compacto) em `~/.config/qara-desktop/state.json`; hoje o toggle "Compactar" se perde a cada boot.
- **Relógio alinhado ao minuto** (item 3 do diagnóstico) — trivial e reduz wakeups.
- **Ícones dos atalhos**: hoje só há a barrinha taupe; ícones monocromáticos SVG inline (sem CDN, o CSP já bloqueia) melhoram o reconhecimento rápido na recepção sem virar "desktop de ícones".
- **Métricas de idle**: verificar CPU ~0% e memória nas máquinas de 8 GB (critério da SPEC §8); WebKitGTK pode manter compositor ativo — medir e, se preciso, pausar rendering quando ocluso.
- **Widgets agregados (Fase 5)**: somente após launcher estável; sempre agregados, nunca PHI/PII (ADR-003). Cada integração (agenda, tarefas) nasce com um ADR próprio.
- **Higiene do repo**: `CODEX_PROMPT.md` cumpriu seu papel de bootstrap — mover para `docs/` ou remover; `AGENTS.md` permanece como guia de agentes.

---

## Riscos e mitigações

1. **Incompatibilidade WebKitGTK 6 / layer-shell no COSMIC** — risco principal do projeto. Mitigação: Fase 0 (diagnóstico na máquina real) antes de investir em features; ADR-002 já prevê rota alternativa (PyGObject) sem trocar a UI.
2. **Versões de sistema vs. crates**: gtk4-rs 0.11 pode pedir feature flag acima do GTK do Pop!_OS 24.04. Mitigação: fixar `features = ["v4_14"]` (ou o que `docs/ENVIRONMENT.md` reportar) e testar `cargo check` cedo.
3. **Regressão de segurança por conveniência**: a tentação de aceitar `target` livre do JS na bridge. Mitigação: teste automatizado que falha se qualquer ação aceitar URL/comando vindo da mensagem (2.4), e a regra já escrita em `AGENTS.md`.
4. **Desvio visual**: adicionar features sem respeitar a diretriz "clínica, silenciosa". Mitigação: manter a lista de "evitar" do `AGENTS.md` como checklist de review.

## Ordem de execução sugerida

1. P0 completo (meio dia de trabalho, desbloqueia todo o resto).
2. 1.1 → 1.2 → 1.3 → 1.4 nessa ordem (cada uma depende da anterior).
3. 2.1 na máquina alvo assim que 1.1 estiver estável (é o maior risco — antecipar).
4. 2.3 + 2.5 juntos (instalação), depois 2.2, 2.4 contínuo desde P1.
5. P3 conforme demanda da clínica.

Critérios de aceite finais permanecem os da SPEC §12 — este plano não os altera, apenas traça o caminho até eles.
