#!/usr/bin/env bash
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

missing=()
for p in "${packages[@]}"; do
  if apt-cache show "$p" >/dev/null 2>&1; then
    missing+=("$p")
  else
    echo "Pacote não encontrado no repositório atual: $p" >&2
  fi
done

sudo apt-get install -y "${missing[@]}"

if ! command -v cargo >/dev/null 2>&1; then
  cat <<'MSG'
Rust/Cargo não encontrado.
Instale pelo método preferido da estação e execute novamente o diagnóstico.
Não instalamos rustup automaticamente para evitar alterar toolchains sem revisão.
MSG
fi
