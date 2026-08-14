# Tarefa inicial para Codex

Implemente a Fase 0 e a Fase 1 do QARA Desktop descritas em `SPEC.md`.

## Contexto

Este computador deve usar Pop!_OS/COSMIC. O objetivo final é uma superfície de background Wayland interativa da Clínica QARA. O starter já contém UI HTML/CSS/JS e um esqueleto Rust.

## Ordem obrigatória

1. Leia `AGENTS.md` e `SPEC.md` integralmente.
2. Execute `./scripts/diagnose.sh` e registre os resultados relevantes em `docs/ENVIRONMENT.md`.
3. Verifique quais pacotes GTK4, WebKitGTK 6 e gtk4-layer-shell realmente existem nesta instalação antes de instalar qualquer coisa.
4. Ajuste `Cargo.toml` somente com versões compatíveis com as bibliotecas do sistema.
5. Faça `cargo check` funcionar.
6. Implemente o modo `--windowed` exibindo `ui/index.html` com WebKitGTK 6.
7. Preserve o CSS e os tokens oficiais QARA.
8. Faça links web abrirem no navegador padrão; não implemente shell arbitrário.
9. Implemente carregamento e validação de `config/shortcuts.example.json`.
10. Só depois de o modo windowed funcionar, implemente o layer-shell em uma branch/commit logicamente separado.

## Critérios desta tarefa

- projeto compila;
- `--windowed` funciona;
- UI local carrega;
- relógio funciona;
- links web funcionam;
- configuração inválida produz erro legível;
- sem execução arbitrária de shell;
- README atualizado com comandos realmente testados no computador.

Não faça integrações com CRM, Doctoralia, Kommo ou Google APIs nesta etapa.
