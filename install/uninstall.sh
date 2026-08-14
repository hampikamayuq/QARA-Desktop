#!/usr/bin/env bash
# uninstall.sh — remove o QARA Desktop instalado para o usuário atual.
#
# O que este script remove:
#   - o binário em ~/.local/bin/qara-desktop;
#   - a entrada .desktop em ~/.local/share/applications/qara-desktop.desktop;
#   - o autostart em ~/.config/autostart/qara-desktop.desktop;
#   - o ícone em
#     ~/.local/share/icons/hicolor/scalable/apps/qara-desktop.svg.
#
# O que este script PRESERVA (de propósito):
#   - a configuração do usuário em ~/.config/qara-desktop/ (inclui
#     config.json com os atalhos personalizados da estação). Para remover
#     também a configuração, apague manualmente esse diretório:
#       rm -rf ~/.config/qara-desktop
#
# Desativar/ativar autostart sem desinstalar:
#   Desativar: rm -f ~/.config/autostart/qara-desktop.desktop
#   Ativar:    cp ~/.local/share/applications/qara-desktop.desktop \
#                 ~/.config/autostart/qara-desktop.desktop
#
# Uso:
#   ./install/uninstall.sh
#
# Idempotente: pode ser executado mais de uma vez sem erro, mesmo se a
# instalação já tiver sido parcialmente removida.
set -euo pipefail

bin_dest="${HOME}/.local/bin/qara-desktop"
desktop_dest="${HOME}/.local/share/applications/qara-desktop.desktop"
autostart_dest="${HOME}/.config/autostart/qara-desktop.desktop"
icon_dest="${HOME}/.local/share/icons/hicolor/scalable/apps/qara-desktop.svg"
config_dir="${HOME}/.config/qara-desktop"

remove_if_exists() {
  local path="$1"
  if [[ -e "${path}" ]]; then
    rm -f "${path}"
    echo "    Removido: ${path}"
  else
    echo "    Já não existia: ${path}"
  fi
}

echo "==> Removendo binário"
remove_if_exists "${bin_dest}"

echo "==> Removendo entrada .desktop"
remove_if_exists "${desktop_dest}"

echo "==> Removendo autostart"
remove_if_exists "${autostart_dest}"

echo "==> Removendo ícone"
remove_if_exists "${icon_dest}"

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f -t "${HOME}/.local/share/icons/hicolor" >/dev/null 2>&1 || true
fi
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "${HOME}/.local/share/applications" >/dev/null 2>&1 || true
fi

echo "==> Desinstalação concluída."
echo "    A configuração do usuário foi PRESERVADA em: ${config_dir}"
echo "    Para removê-la também, rode: rm -rf ${config_dir}"
