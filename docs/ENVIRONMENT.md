# Ambiente de execução (máquina alvo)

Este documento registra o diagnóstico real da máquina Pop!_OS onde o QARA
Desktop vai rodar. Ele **precisa ser preenchido na máquina alvo de verdade**
— não é um documento especulativo. Os valores aqui influenciam decisões de
código (por exemplo, a feature flag de versão do crate `gtk4` no
`Cargo.toml` deve casar com a versão do GTK reportada abaixo).

## Como preencher

Na máquina Pop!_OS/COSMIC alvo, rode:

```bash
chmod +x scripts/diagnose.sh
./scripts/diagnose.sh
```

Copie a saída relevante para as seções abaixo (ou anexe a saída completa em
um bloco de código no final deste arquivo). Atualize este documento sempre
que a estação for reinstalada, atualizada de versão de SO, ou trocar de
monitor/escala.

> Status atual: **não preenchido**. Nenhum diagnóstico real foi rodado numa
> máquina Pop!_OS ainda; os valores abaixo são placeholders a substituir.

## Sistema operacional

| Item | Valor |
|---|---|
| Distribuição (`/etc/os-release`) | _preencher_ |
| Versão do kernel (`uname -a`) | _preencher_ |
| Sessão gráfica (`XDG_SESSION_TYPE`) | _preencher_ |
| Desktop atual (`XDG_CURRENT_DESKTOP`) | _preencher_ |
| Compositor (`cosmic-comp` ativo?) | _preencher_ |

## Bibliotecas gráficas

| Biblioteca | Versão (pkg-config) | Observações |
|---|---|---|
| GTK4 (`gtk4`) | _preencher_ | Ubuntu/Pop!_OS 24.04 tipicamente traz GTK 4.14 — ver nota abaixo |
| WebKitGTK 6 (`webkitgtk-6.0`) | _preencher_ | |
| gtk4-layer-shell (`gtk4-layer-shell-0`) | _preencher_ | Pacote `libgtk4-layer-shell-dev` pode não existir fora dos repos Pop!_OS; ver `scripts/install-deps-popos.sh` |

## Toolchain Rust

| Item | Valor |
|---|---|
| `rustc --version` | _preencher_ |
| `cargo --version` | _preencher_ |
| Instalado via rustup? | _preencher_ (ver `rust-toolchain.toml` — edition 2024 exige Rust >= 1.85, geralmente indisponível no apt do Pop!_OS 24.04) |

## Monitores e escala

| Item | Valor |
|---|---|
| Número de monitores | _preencher_ |
| Resolução(ões) | _preencher_ |
| Escala(s) de UI (100%/125%/150%/200%) | _preencher_ |
| Ferramenta usada para consultar (`cosmic-randr`/`wlr-randr`) | _preencher_ |

## Suporte a layer-shell

| Item | Valor |
|---|---|
| `gtk4_layer_shell::is_supported()` retorna verdadeiro? | _preencher_ (validar rodando o host, não apenas o diagnóstico) |
| Comportamento observado (Alt+Tab, foco, exclusive zone) | _preencher_ |

## Nota sobre feature flags do crate `gtk4`

O crate `gtk4` (gtk4-rs) usa feature flags de versão (`v4_10`, `v4_12`,
`v4_14`, ...) para expor APIs condicionalmente conforme a versão mínima do
GTK do sistema. Essas features **devem corresponder à versão de GTK
reportada na tabela acima**, e são declaradas em `Cargo.toml` (fora do
escopo deste documento — ajuste é feito por quem edita `Cargo.toml`).

Como referência: Ubuntu/Pop!_OS 24.04 normalmente traz **GTK 4.14**, o que
sugere a feature `v4_14`. Confirme sempre com o valor real reportado por
`pkg-config --modversion gtk4` na máquina alvo antes de fixar a flag —
não assuma a partir deste documento sozinho enquanto ele não estiver
preenchido.

## Saída bruta do diagnóstico (opcional)

Cole aqui a saída completa de `./scripts/diagnose.sh`, para referência
futura:

```text
(colar saída de ./scripts/diagnose.sh aqui)
```
