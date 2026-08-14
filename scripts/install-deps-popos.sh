#!/usr/bin/env bash
# Instala as dependências de build do QARA Desktop no Pop!_OS/Ubuntu 24.04.
#
# Idempotente: pode ser executado quantas vezes forem necessárias sem falhar
# nem reinstalar pacotes já presentes.
set -euo pipefail

if ! command -v apt-get >/dev/null 2>&1; then
  echo "Este script é destinado a Pop!_OS/Ubuntu (apt)." >&2
  exit 1
fi

sudo apt-get update

packages=(
  build-essential
  pkg-config
  libgtk-4-dev
  libwebkitgtk-6.0-dev
  libgtk4-layer-shell-dev
)

to_install=()
unavailable=()

for p in "${packages[@]}"; do
  # Já instalado? (dpkg-query só retorna "ok installed" para pacotes presentes)
  if dpkg-query -W -f='${Status}' "$p" 2>/dev/null | grep -q "ok installed"; then
    echo "Já instalado: $p"
    continue
  fi

  # Disponível no repositório atual?
  if apt-cache show "$p" >/dev/null 2>&1; then
    to_install+=("$p")
  else
    unavailable+=("$p")
    echo "Pacote não encontrado no repositório atual: $p" >&2
  fi
done

if [[ ${#to_install[@]} -gt 0 ]]; then
  echo "Instalando: ${to_install[*]}"
  sudo apt-get install -y "${to_install[@]}"
else
  echo "Nenhum pacote novo para instalar via apt."
fi

# libgtk4-layer-shell-dev não existe nos repositórios do Ubuntu 24.04 puro
# (apenas nos repos do Pop!_OS). Se ficou de fora, oriente a compilação do
# fonte em vez de falhar silenciosamente.
for p in "${unavailable[@]}"; do
  if [[ "$p" == "libgtk4-layer-shell-dev" ]]; then
    cat <<'MSG' >&2

ATENÇÃO: libgtk4-layer-shell-dev não está disponível neste repositório
(comum em Ubuntu 24.04 puro; o pacote só existe nos repos do Pop!_OS).

Compile e instale a gtk4-layer-shell a partir do fonte:

  sudo apt-get install -y meson ninja-build libwayland-dev wayland-protocols \
    gobject-introspection libgirepository1.0-dev valac
  git clone https://github.com/wmww/gtk4-layer-shell.git /tmp/gtk4-layer-shell
  cd /tmp/gtk4-layer-shell
  meson setup build -Dexamples=false -Ddocs=false -Dtests=false
  ninja -C build
  sudo ninja -C build install
  sudo ldconfig

MSG
  fi
done

if ! command -v cargo >/dev/null 2>&1; then
  cat <<'MSG'
Rust/Cargo não encontrado.
Instale pelo método preferido da estação e execute novamente o diagnóstico.
Não instalamos rustup automaticamente para evitar alterar toolchains sem revisão.
MSG
fi
