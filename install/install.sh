#!/usr/bin/env bash
# install.sh — instala o QARA Desktop para o usuário atual.
#
# O que este script faz:
#   1. compila o binário em modo release (`cargo build --release`);
#   2. copia o binário para ~/.local/bin/qara-desktop;
#   3. gera ~/.config/autostart/qara-desktop.desktop com o Exec apontando
#      para o caminho real do binário instalado (autostart ativado por
#      padrão — veja a flag --no-autostart abaixo);
#   4. copia config/shortcuts.example.json para
#      ~/.config/qara-desktop/config.json SOMENTE se esse arquivo ainda
#      não existir (nunca sobrescreve configuração já personalizada);
#   5. instala o ícone install/qara-desktop.svg em
#      ~/.local/share/icons/hicolor/scalable/apps/qara-desktop.svg, se o
#      arquivo já existir no repositório (caso contrário, avisa e segue).
#
# Uso:
#   ./install/install.sh                # instala com autostart ativado
#   ./install/install.sh --no-autostart  # instala sem gerar o autostart
#
# Ativar/desativar autostart manualmente depois de instalado:
#   Ativar:    ln -sf ~/.local/share/applications/qara-desktop.desktop \
#                      ~/.config/autostart/qara-desktop.desktop
#              (ou simplesmente rode ./install/install.sh de novo sem a flag)
#   Desativar: rm -f ~/.config/autostart/qara-desktop.desktop
#              (ou edite esse arquivo e defina X-GNOME-Autostart-enabled=false)
#
# Este script é idempotente: rodar mais de uma vez não duplica nada nem
# falha, e nunca sobrescreve ~/.config/qara-desktop/config.json existente.
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd -P)"
repo_root="$(cd -- "${script_dir}/.." >/dev/null 2>&1 && pwd -P)"

autostart=1
for arg in "$@"; do
  case "$arg" in
    --no-autostart)
      autostart=0
      ;;
    *)
      echo "Opção desconhecida: $arg" >&2
      echo "Uso: $0 [--no-autostart]" >&2
      exit 1
      ;;
  esac
done

bin_dir="${HOME}/.local/bin"
app_dir="${HOME}/.local/share/applications"
autostart_dir="${HOME}/.config/autostart"
config_dir="${HOME}/.config/qara-desktop"
icon_dir="${HOME}/.local/share/icons/hicolor/scalable/apps"

bin_dest="${bin_dir}/qara-desktop"
desktop_dest="${app_dir}/qara-desktop.desktop"
autostart_dest="${autostart_dir}/qara-desktop.desktop"
config_dest="${config_dir}/config.json"
icon_dest="${icon_dir}/qara-desktop.svg"

echo "==> Compilando qara-desktop em modo release"
( cd "${repo_root}" && cargo build --release )

release_bin="${repo_root}/target/release/qara-desktop"
if [[ ! -f "${release_bin}" ]]; then
  echo "ERRO: binário não encontrado em ${release_bin} após o build." >&2
  exit 1
fi

echo "==> Instalando binário em ${bin_dest}"
mkdir -p "${bin_dir}"
install -m 0755 "${release_bin}" "${bin_dest}"

echo "==> Gerando entrada .desktop em ${desktop_dest}"
mkdir -p "${app_dir}"
{
  echo "[Desktop Entry]"
  echo "Type=Application"
  echo "Name=QARA Desktop"
  echo "Comment=Superfície interativa da Clínica QARA"
  echo "Icon=qara-desktop"
  echo "Exec=${bin_dest}"
  echo "Terminal=false"
  echo "X-GNOME-Autostart-enabled=true"
  echo "NoDisplay=true"
} > "${desktop_dest}"

if [[ "${autostart}" -eq 1 ]]; then
  echo "==> Ativando autostart em ${autostart_dest}"
  mkdir -p "${autostart_dir}"
  cp "${desktop_dest}" "${autostart_dest}"
else
  echo "==> --no-autostart informado: não instalando em ${autostart_dir}"
  if [[ -f "${autostart_dest}" ]]; then
    echo "    (havia um autostart anterior em ${autostart_dest}; removendo)"
    rm -f "${autostart_dest}"
  fi
fi

echo "==> Configuração do usuário"
mkdir -p "${config_dir}"
if [[ -f "${config_dest}" ]]; then
  echo "    Já existe ${config_dest}; mantendo (não sobrescrito)."
else
  example_config="${repo_root}/config/shortcuts.example.json"
  if [[ -f "${example_config}" ]]; then
    cp "${example_config}" "${config_dest}"
    echo "    Config padrão copiada para ${config_dest}."
  else
    echo "    AVISO: ${example_config} não encontrado; pulei a criação da config." >&2
  fi
fi

echo "==> Ícone"
repo_icon="${repo_root}/install/qara-desktop.svg"
if [[ -f "${repo_icon}" ]]; then
  mkdir -p "${icon_dir}"
  install -m 0644 "${repo_icon}" "${icon_dest}"
  echo "    Ícone instalado em ${icon_dest}."
  if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "${HOME}/.local/share/icons/hicolor" >/dev/null 2>&1 || true
  fi
else
  echo "    AVISO: ${repo_icon} não encontrado; pulando instalação do ícone." >&2
  echo "    (o .desktop referencia Icon=qara-desktop; instale o SVG depois e rode este script de novo)" >&2
fi

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "${app_dir}" >/dev/null 2>&1 || true
fi

echo "==> Instalação concluída."
echo "    Binário:    ${bin_dest}"
echo "    Desktop:    ${desktop_dest}"
if [[ "${autostart}" -eq 1 ]]; then
  echo "    Autostart:  ativado (${autostart_dest})"
else
  echo "    Autostart:  desativado"
fi
echo "    Config:     ${config_dest}"
