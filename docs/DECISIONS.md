# Architecture Decisions

## ADR-001 — Host nativo + UI web local

**Status:** accepted

Usaremos GTK4/Wayland como host e WebKitGTK 6 para a interface HTML/CSS/JS.

Motivos:

- acesso direto ao layer-shell;
- UI altamente customizável;
- footprint menor que Electron;
- separação clara entre apresentação e privilégios locais.

## ADR-002 — Rust para o host

**Status:** accepted for MVP implementation attempt

Rust concentra bridge, validação, execução de processos e integração GTK. Se incompatibilidades específicas da distribuição tornarem o caminho Rust desproporcionalmente complexo, Python/PyGObject pode ser avaliado em novo ADR, sem alterar a UI web.

## ADR-003 — Sem dados identificáveis no desktop

**Status:** accepted

Qualquer integração futura deve apresentar apenas indicadores agregados por padrão.
