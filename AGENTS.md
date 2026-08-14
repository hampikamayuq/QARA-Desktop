# AGENTS.md — QARA Desktop

## Missão

Implementar um desktop interativo para a Clínica QARA em Pop!_OS/COSMIC, usando GTK4 + gtk4-layer-shell + WebKitGTK 6, com UI HTML/CSS/JS e foco forte em segurança, estabilidade e coerência visual.

Leia primeiro:

1. `SPEC.md`
2. `README.md`
3. `config/shortcuts.example.json`

## Regras obrigatórias

1. Não executar conteúdo recebido do WebView com `sh -c`, `bash -c`, `eval`, `system()` ou equivalentes.
2. Aplicativos locais são sempre resolvidos por allowlist nativa/config validada.
3. URLs externas aceitam apenas esquemas explicitamente permitidos.
4. Não exibir dados identificáveis de pacientes.
5. Não adicionar dependências pesadas sem justificar.
6. Não introduzir Electron/Chromium.
7. Não substituir GTK4/WebKitGTK por Tauri/Wry sem evidência de que a implementação nativa falhou no ambiente alvo.
8. Manter modo `--windowed` para desenvolvimento.
9. Antes de alterar arquitetura, registrar decisão em `docs/DECISIONS.md`.
10. Não redistribuir fontes proprietárias. Usar Telegraf apenas se já instalada, com fallbacks.

## Design

Usar os tokens QARA definidos em `ui/styles.css` e `SPEC.md`.

A marca deve parecer:

- médica;
- contemporânea;
- precisa;
- silenciosa;
- acolhedora sem estética de spa.

Evitar:

- dourado ornamental;
- glassmorphism excessivo;
- gradientes chamativos;
- ícones 3D;
- sombras pesadas;
- cards em excesso;
- animação contínua.

## Estratégia de implementação

### Primeiro

Faça o host abrir `ui/index.html` em modo `--windowed`.

### Depois

Adicione layer-shell e valide `gtk4_layer_shell::is_supported()`.

### Depois

Implemente a bridge de ações com mensagens tipadas e allowlist.

### Finalmente

Adicione autostart, logs, tratamento de múltiplos monitores e testes.

## Qualidade

Antes de declarar uma etapa concluída:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- executar smoke test manual no COSMIC;
- conferir logs;
- conferir CPU/memória em idle.

## Diagnóstico antes de instalar dependências

Execute:

```bash
./scripts/diagnose.sh
```

Não assuma o compositor ou a versão do sistema.
