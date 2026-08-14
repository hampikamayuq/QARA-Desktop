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

## ADR-004 — UI servida por custom scheme com assets embutidos

**Status:** accepted

A UI é carregada via `qara://ui/index.html` (scheme registrado no WebKitGTK), com os arquivos de `ui/` embutidos no binário em release e a flag `--ui-dir` para carregar do disco em desenvolvimento.

Motivos:

- não expõe o filesystem via `file://`;
- CSP e origem estáveis, independentes do caminho de instalação;
- binário único simplifica instalação nas estações;
- `decide-policy` nega qualquer navegação fora de `qara://`, fechando a superfície de navegação.

## ADR-005 — Protocolo de bridge v1 (mensagens tipadas por ID)

**Status:** accepted

O contrato host ↔ UI está em `docs/PROTOCOL.md`: mensagens JSON versionadas (`v: 1`) com conjunto fechado de ações, correlação por `req_id` e respostas com códigos de erro fixos. O JavaScript nunca envia URL nem executável — apenas IDs resolvidos pelo host contra a configuração validada (`deny_unknown_fields` rejeita campos extras). Mudanças no protocolo exigem atualizar os dois lados e o documento na mesma alteração.

## ADR-006 — Teclado desabilitado na camada de background

**Status:** accepted

Em modo layer-shell a janela usa `KeyboardMode::None`: um wallpaper interativo não deve disputar foco de teclado com as janelas de trabalho da recepção. Consequência aceita: navegação por teclado e os estilos `:focus-visible` da UI só operam em `--windowed` (desenvolvimento). Se surgir demanda real de acessibilidade por teclado no desktop, reavaliar `KeyboardMode::OnDemand` em novo ADR.

## ADR-007 — Arte institucional oficial como fundo

**Status:** accepted

O fundo do desktop usa a arte oficial da marca QARA (`ui/assets/wallpaper.webp`, fornecida pela clínica, otimizada de PNG 1,7 MB para WebP ~60 KB), embutida no binário e servida via `qara://ui/assets/wallpaper.webp`. Como a arte já contém wordmark, kicker, tagline e localização, os equivalentes em HTML (`.brand-panel`, `.location`, formas `.ambient`) ficam ocultos por CSS — o markup e o `config.branding` permanecem no código para permitir reverter a um fundo neutro sem retrabalho. O launcher ocupa a área vazia à esquerda da composição. Nota: a diretriz "evitar dourado ornamental" de AGENTS.md vale para elementos de UI criados por nós; a arte oficial da marca, fornecida pela clínica, prevalece.
