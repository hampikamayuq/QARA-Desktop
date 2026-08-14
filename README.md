# QARA Desktop

Desktop interativo institucional para computadores da Clínica QARA em Pop!_OS/COSMIC.

## Objetivo

Transformar o fundo do desktop em uma superfície QARA limpa e interativa, com atalhos para ferramentas de trabalho sem reproduzir um desktop convencional cheio de ícones.

O projeto separa:

- **host nativo**: GTK4 + `gtk4-layer-shell` + WebKitGTK 6;
- **UI**: HTML/CSS/JavaScript;
- **configuração**: atalhos declarativos em JSON;
- **segurança**: execução local exclusivamente por allowlist.

## Identidade visual

Tokens oficiais do design system QARA:

- Graphite: `#404041`
- Ink: `#29292A`
- Taupe: `#A28A7F`
- Taupe deep: `#796960`
- Blush: `#FDE6DC`
- Blush soft: `#FFF1EB`
- Gray 050: `#F1F1F2`
- Gray 200: `#D0D2D3`
- Copy: `#5F5F61`
- White: `#FFFFFF`
- Focus: `#735F56`

Diretriz: clínica contemporânea, clara, silenciosa e médica. Não transformar a interface em estética de spa; evitar dourado ornamental.

## Estado do starter

Este pacote contém:

1. especificação técnica (`SPEC.md`);
2. instruções para Codex (`AGENTS.md` e `CODEX_PROMPT.md`);
3. protótipo web funcional (`ui/`);
4. esqueleto do host Rust (`src/main.rs`);
5. configuração inicial de atalhos (`config/shortcuts.example.json`);
6. diagnóstico do ambiente (`scripts/diagnose.sh`);
7. instalador de dependências base (`scripts/install-deps-popos.sh`);
8. autostart (`install/qara-desktop.desktop`).

## Primeiro passo no computador Pop!_OS

```bash
chmod +x scripts/*.sh
./scripts/diagnose.sh
```

Depois abra o projeto no Codex e use `CODEX_PROMPT.md` como tarefa inicial.

## Preview da UI

Antes do host nativo estar pronto:

```bash
python3 -m http.server 8080 -d ui
```

Abra `http://localhost:8080`.

## Princípio de segurança

A página HTML nunca recebe acesso arbitrário ao shell. O host aceita somente ações conhecidas e configuradas, por exemplo `open_url`, `launch_app` e, futuramente, ações internas do CRM. Nenhum conteúdo remoto pode injetar comandos locais.
