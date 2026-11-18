#!/usr/bin/env bash
set -e

echo "🗑️ Desinstalando cosmic-applet-disk-usage..."

PANEL_CONFIG="$HOME/.config/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings"
if [ -f "$PANEL_CONFIG" ]; then
    echo "⚙️ Removendo applet da barra do COSMIC..."
    sed -i '/com.wisk.cosmic-applet-disk-usage/d' "$PANEL_CONFIG"
fi

rm -f "$HOME/.local/bin/cosmic-applet-disk-usage"
rm -f "$HOME/.local/share/applications/com.wisk.cosmic-applet-disk-usage.desktop"
rm -f "$HOME/.local/share/icons/hicolor/scalable/apps/com.wisk.cosmic-applet-disk-usage.svg"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$HOME/.local/share/applications" || true
fi

if pgrep -x cosmic-panel >/dev/null 2>&1; then
    killall cosmic-panel 2>/dev/null || true
fi

echo "✅ cosmic-applet-disk-usage desinstalado com sucesso!"
