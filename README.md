# QARA Desktop

Desktop interativo institucional para computadores da Clínica QARA em Pop!_OS/COSMIC.

Transforma o fundo do desktop em uma superfície QARA limpa e interativa (camada Wayland de background via layer-shell), com atalhos para ferramentas de trabalho — sem reproduzir um desktop convencional cheio de ícones.

## Arquitetura

- **Host nativo** (`src/`): Rust + GTK4 + `gtk4-layer-shell` + WebKitGTK 6. Serve a UI por scheme local `qara://`, valida a configuração, executa ações por allowlist e escreve logs.
- **UI** (`ui/`): HTML/CSS/JavaScript puros, carregados na WebView. Conversa com o host apenas pelo protocolo tipado de `docs/PROTOCOL.md`.
- **Configuração** (`config/`): JSON declarativo por estação (`~/.config/qara-desktop/config.json`), com perfis de exemplo em `config/profiles/`.
- **Segurança**: a página nunca executa shell. O host aceita somente ações fechadas (`open_url`, `launch_app`, `get_system_info`, `set_ui_preference`) resolvidas por ID contra a config validada. Detalhes em `docs/PROTOCOL.md` e `docs/DECISIONS.md`.

## Compilar e rodar

Pré-requisitos (Pop!_OS/Ubuntu 24.04):

```bash
./scripts/diagnose.sh              # confirma ambiente (registre em docs/ENVIRONMENT.md)
./scripts/install-deps-popos.sh    # GTK4, WebKitGTK 6, layer-shell (ou instrução de build do fonte)
# Rust via rustup (o rustc do apt é antigo demais para edition 2024):
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Desenvolvimento:

```bash
cargo run -- --windowed            # janela normal, para depuração
cargo run -- --windowed --devtools # com inspetor WebKit
cargo run -- --ui-dir ui           # UI carregada do disco em vez dos assets embutidos
cargo run -- --check-config        # valida a config e sai (0/1)
```

Produção (na máquina COSMIC): rode sem flags para o modo layer-shell (background em todos os monitores) e instale com:

```bash
./install/install.sh               # binário em ~/.local/bin + autostart (--no-autostart para pular)
./install/uninstall.sh             # remove tudo, preserva a config do usuário
```

## Preview da UI sem host

```bash
./scripts/run-preview.sh           # http://localhost:8080 (modo degradado, sem bridge)
```

## Configuração

`~/.config/qara-desktop/config.json` (fallback: `config/shortcuts.example.json`, embutido no binário). Perfis prontos por estação em `config/profiles/{reception,clinical,admin}.json`. O arquivo é observado: edições válidas recarregam a UI ao vivo; inválidas são logadas e ignoradas.

Logs: `~/.local/state/qara-desktop/qara-desktop.log`.

## Qualidade

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test                         # 52 testes (config, bridge, launcher, prefs, CLI)
```

A CI (`.github/workflows/ci.yml`) roda tudo isso em ubuntu-24.04, compilando o gtk4-layer-shell do fonte (não há pacote Ubuntu). A UI tem verificação Playwright documentada no histórico do repositório.

## Identidade visual

Tokens oficiais QARA em `ui/styles.css` e SPEC §5. Diretriz: clínica contemporânea, clara, silenciosa e médica — sem dourado ornamental, sem estética de spa (lista completa de "evitar" em `AGENTS.md`).

## Privacidade

O wallpaper é visível a terceiros: nunca exibir nomes de pacientes, telefones, diagnósticos ou conversas (SPEC §7, ADR-003). Widgets futuros mostram apenas agregados.

## Documentos

| Documento | Conteúdo |
|-----------|----------|
| `SPEC.md` | Especificação técnica completa |
| `docs/PROTOCOL.md` | Contrato host ↔ UI (bridge v1) |
| `docs/DECISIONS.md` | ADRs |
| `docs/PLANO_DE_MELHORIAS.md` | Plano executado e pendências |
| `docs/ENVIRONMENT.md` | Diagnóstico da máquina alvo (preencher) |
| `AGENTS.md` | Guia para agentes/contribuidores |
