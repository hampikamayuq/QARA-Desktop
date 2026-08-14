#!/usr/bin/env bash
set -u

section() { printf '\n== %s ==\n' "$1"; }
cmd() {
  printf '$ %s\n' "$*"
  "$@" 2>&1 || true
}

section "Sistema"
cmd cat /etc/os-release
cmd uname -a

section "Sessão gráfica"
printf 'XDG_SESSION_TYPE=%s\n' "${XDG_SESSION_TYPE:-}"
printf 'XDG_CURRENT_DESKTOP=%s\n' "${XDG_CURRENT_DESKTOP:-}"
printf 'XDG_SESSION_DESKTOP=%s\n' "${XDG_SESSION_DESKTOP:-}"
printf 'WAYLAND_DISPLAY=%s\n' "${WAYLAND_DISPLAY:-}"
printf 'DISPLAY=%s\n' "${DISPLAY:-}"

section "COSMIC"
cmd pgrep -a cosmic-comp
cmd pgrep -a cosmic-session

section "Bibliotecas pkg-config"
for pkg in gtk4 webkitgtk-6.0 gtk4-layer-shell-0; do
  if command -v pkg-config >/dev/null 2>&1; then
    printf '%-24s ' "$pkg"
    pkg-config --modversion "$pkg" 2>/dev/null || echo "não encontrado"
  fi
done

section "Pacotes relevantes"
for pkg in libgtk-4-dev libwebkitgtk-6.0-dev libgtk4-layer-shell-dev pkg-config cargo rustc; do
  printf '%-30s ' "$pkg"
  dpkg-query -W -f='${Version}\n' "$pkg" 2>/dev/null || echo "não instalado"
done

section "Rust"
cmd rustc --version
cmd cargo --version

section "Monitores"
if command -v cosmic-randr >/dev/null 2>&1; then
  cmd cosmic-randr
elif command -v wlr-randr >/dev/null 2>&1; then
  cmd wlr-randr
else
  echo "cosmic-randr/wlr-randr não encontrado; coletar resolução pelo COSMIC Settings."
fi

section "Resumo"
if [[ "${XDG_SESSION_TYPE:-}" == "wayland" ]]; then
  echo "OK: sessão Wayland detectada."
else
  echo "ATENÇÃO: sessão atual não parece Wayland."
fi
