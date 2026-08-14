# QARA Desktop — Especificação Técnica v0.1

## 1. Produto

**Nome:** QARA Desktop  
**Plataforma alvo:** Pop!_OS 24.04 / COSMIC / Wayland  
**Uso:** computadores administrativos e clínicos da Clínica QARA.

### Objetivo

Criar uma camada de desktop interativa que funcione como wallpaper institucional e launcher de ferramentas, preservando uma aparência minimalista e premium alinhada ao site da QARA.

### Não objetivos do MVP

- substituir o compositor COSMIC;
- implementar gerenciador de arquivos;
- replicar dock ou launcher do sistema;
- exibir dados identificáveis de pacientes;
- embutir páginas externas inteiras no desktop;
- permitir execução arbitrária de comandos pelo JavaScript.

## 2. Stack escolhida

### Host

- Rust
- GTK4
- `gtk4-layer-shell`
- WebKitGTK 6.0 (`webkit6` no ecossistema Rust)

### Interface

- HTML sem framework no MVP
- CSS puro
- JavaScript ES2022+

### Configuração

- JSON local em `~/.config/qara-desktop/config.json`
- valores padrão empacotados em `config/shortcuts.example.json`

### Execução

- processo de usuário
- autostart via `.desktop` inicialmente
- systemd --user apenas se necessário em uma fase posterior

## 3. Comportamento da janela

A janela deve:

- usar layer-shell;
- utilizar `Layer::Background`;
- ancorar em top/right/bottom/left;
- não reservar exclusive zone;
- ocupar o monitor inteiro;
- não aparecer como janela normal no Alt+Tab;
- não solicitar teclado por padrão;
- permitir mouse nos componentes interativos;
- sobreviver a alterações de resolução e escala;
- oferecer suporte futuro a múltiplos monitores.

### Fallback

Se layer-shell não estiver disponível:

1. registrar erro claro;
2. iniciar opcionalmente em modo `--windowed` para desenvolvimento;
3. nunca tentar hacks X11 silenciosamente em sessão Wayland.

## 4. UX do MVP

### Estado repouso

- fundo claro QARA;
- wordmark QARA;
- frase “Conhecimento que cuida. Presença que transforma.”;
- relógio e data discretos;
- launcher compacto no lado direito;
- ausência de animações contínuas.

### Launcher

Grupos iniciais:

**Atendimento**
- Doctoralia
- Kommo
- WhatsApp

**Organização**
- Gmail
- Google Calendar
- Arquivos

**QARA**
- CRM QARA
- Site QARA

### Interação

- cards/atalhos com hit-area mínima de 44 px;
- hover discreto;
- foco visível;
- animações entre 120–220 ms;
- nenhum efeito que prejudique desempenho em idle.

## 5. Design tokens oficiais

```css
--qara-graphite: #404041;
--qara-ink: #29292A;
--qara-taupe: #A28A7F;
--qara-taupe-deep: #796960;
--qara-blush: #FDE6DC;
--qara-blush-soft: #FFF1EB;
--qara-gray-050: #F1F1F2;
--qara-gray-200: #D0D2D3;
--qara-gray-600: #7A7A7A;
--qara-copy: #5F5F61;
--qara-white: #FFFFFF;
--qara-focus: #735F56;
```

### Tipografia

- display: Telegraf se disponível;
- fallback display: Arial/sans-serif;
- corpo: Roboto, Arial, Helvetica, sans-serif.

O projeto não deve redistribuir arquivos de fonte proprietários. Se Telegraf já estiver instalada no computador, usá-la; caso contrário, fallback.

## 6. Segurança

### Regra central

**HTML/JS não executa shell.**

A ponte nativa expõe um conjunto fechado de comandos:

- `open_url(id)`
- `launch_app(id)`
- `get_system_info()`
- `get_config()`
- `set_ui_preference(key, value)` — somente preferências não sensíveis.

### Allowlist de URL

Cada item possui URL declarada no arquivo de configuração. Apenas esquemas `https:` e, quando estritamente necessário, `mailto:` são aceitos.

### Allowlist de aplicativo

Aplicativos são identificados por ID lógico. O executável real fica na configuração nativa e nunca é recebido como string livre do JavaScript.

Exemplo:

```json
{
  "chrome": ["google-chrome-stable"],
  "files": ["cosmic-files"]
}
```

O host deve usar `std::process::Command` sem `sh -c`.

### WebView

- DevTools desativado em produção;
- navegação externa interceptada e aberta no navegador padrão;
- UI local carregada por `file://` ou custom scheme local;
- bloquear popups inesperados;
- nenhum token, senha ou informação clínica no DOM;
- CSP no HTML.

## 7. Privacidade clínica

O wallpaper é visível a terceiros. Portanto, por padrão:

- não mostrar nomes de pacientes;
- não mostrar telefones;
- não mostrar motivo de consulta;
- não mostrar diagnósticos;
- não mostrar conversas do WhatsApp;
- não mostrar resultados laboratoriais.

Widgets futuros podem mostrar apenas agregados, por exemplo:

- “8 consultas hoje”;
- “3 confirmações pendentes”;
- “5 tarefas vencendo hoje”.

## 8. Performance

Metas de idle após estabilização:

- CPU média próxima de 0%;
- sem timers de alta frequência;
- relógio atualizado a cada 30 s ou 60 s;
- nenhuma animação infinita;
- memória monitorada em máquinas de 8 GB;
- imagens otimizadas em WebP/AVIF quando usadas.

## 9. Configuração por estação

Perfis previstos:

### `reception`
Doctoralia, Kommo, WhatsApp, Gmail, Calendar, CRM, arquivos.

### `clinical`
Agenda, prontuário/CRM, site, ferramentas clínicas autorizadas.

### `admin`
Kommo, Gmail, Calendar, CRM, financeiro, site/admin e ferramentas técnicas autorizadas.

O perfil é definido localmente e não deve depender de dados clínicos remotos no MVP.

## 10. Observabilidade

Logs em:

```text
~/.local/state/qara-desktop/qara-desktop.log
```

Registrar:

- startup;
- ambiente detectado;
- suporte a layer-shell;
- falha ao abrir URL;
- falha ao iniciar aplicativo;
- config inválida;
- crash/panic.

Nunca registrar credenciais ou conteúdo clínico.

## 11. Fases

### Fase 0 — Diagnóstico

- confirmar Pop!_OS;
- confirmar Wayland/COSMIC;
- confirmar suporte a layer-shell;
- confirmar versões GTK/WebKit;
- confirmar escala e número de monitores.

### Fase 1 — MVP windowed

- WebKitGTK exibindo `ui/index.html`;
- UI responsiva;
- relógio;
- links web funcionando;
- config JSON carregada.

### Fase 2 — Layer-shell

- background real;
- ancoragem integral;
- comportamento correto sob COSMIC;
- sem Alt+Tab;
- autostart.

### Fase 3 — Bridge nativa

- launch_app;
- open_url;
- allowlist;
- tratamento de erros;
- feedback visual.

### Fase 4 — Polimento

- múltiplos monitores;
- perfis por estação;
- configuração visual;
- testes de DPI/escala;
- empacotamento `.deb` opcional.

### Fase 5 — Widgets QARA opcionais

Somente depois do launcher estar estável:

- indicadores agregados de agenda;
- indicadores agregados de tarefas;
- status de sistemas;
- sem PHI/PII no desktop.

## 12. Critérios de aceite do MVP

- [ ] inicia no Pop!_OS alvo;
- [ ] detecta Wayland/COSMIC corretamente;
- [ ] mostra UI QARA em tela inteira;
- [ ] não interfere em janelas normais;
- [ ] atalhos web abrem navegador padrão;
- [ ] atalhos locais só executam allowlist;
- [ ] não há `sh -c`, `eval` ou shell arbitrário;
- [ ] sem dados identificáveis de pacientes;
- [ ] CPU em idle aceitável;
- [ ] layout correto em 100%, 125%, 150% e 200% de escala;
- [ ] autostart pode ser ativado/desativado facilmente;
- [ ] `--windowed` funciona para depuração.
