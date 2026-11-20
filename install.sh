#!/usr/bin/env bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"

# Configura PKG_CONFIG_PATH caso dependências locais existam
if [ -d "$DIR/.local-deps/usr/lib/x86_64-linux-gnu/pkgconfig" ]; then
    export PKG_CONFIG_PATH="$DIR/.local-deps/usr/lib/x86_64-linux-gnu/pkgconfig:${PKG_CONFIG_PATH}"
fi

echo "🔨 Compilando o cosmic-applet-system-monitor em modo release..."
cargo build --release

echo "📦 Instalando binário e arquivos de integração do COSMIC..."
mkdir -p "$HOME/.local/bin"
mkdir -p "$HOME/.local/share/applications"
mkdir -p "$HOME/.local/share/icons/hicolor/scalable/apps"

# Para processo anterior se estiver rodando para permitir a substituição do binário
killall cosmic-applet-system-monitor 2>/dev/null || true
killall cosmic-applet-disk-usage 2>/dev/null || true

rm -f "$HOME/.local/bin/cosmic-applet-disk-usage"
rm -f "$HOME/.local/share/applications/com.wisk.cosmic-applet-disk-usage.desktop"
rm -f "$HOME/.local/share/icons/hicolor/scalable/apps/com.wisk.cosmic-applet-disk-usage.svg"

cp --remove-destination target/release/cosmic-applet-system-monitor "$HOME/.local/bin/"
chmod +x "$HOME/.local/bin/cosmic-applet-system-monitor"

# Reduz o tamanho do binário se strip estiver disponível
if command -v strip >/dev/null 2>&1; then
    strip "$HOME/.local/bin/cosmic-applet-system-monitor"
fi

# Ajusta o caminho no arquivo .desktop e instala
sed "s|Exec=cosmic-applet-system-monitor|Exec=$HOME/.local/bin/cosmic-applet-system-monitor|g" \
    resources/com.wisk.cosmic-applet-system-monitor.desktop > "$HOME/.local/share/applications/com.wisk.cosmic-applet-system-monitor.desktop"

cp resources/com.wisk.cosmic-applet-system-monitor.svg "$HOME/.local/share/icons/hicolor/scalable/apps/"

# Migra as preferências da versão anterior sem sobrescrever configuração nova.
if [ -d "$HOME/.config/cosmic-applet-disk-usage" ] && [ ! -e "$HOME/.config/cosmic-applet-system-monitor" ]; then
    mv "$HOME/.config/cosmic-applet-disk-usage" "$HOME/.config/cosmic-applet-system-monitor"
fi

# Atualiza cache de aplicações do desktop
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$HOME/.local/share/applications" || true
fi

# Adiciona automaticamente à asa direita (alinhado à direita) do painel do COSMIC
PANEL_CONFIG="$HOME/.config/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings"
if [ -f "$PANEL_CONFIG" ]; then
    echo "⚙️ Configurando alinhamento à direita no painel do COSMIC..."
    python3 - <<EOF
import os

path = os.path.expanduser("$PANEL_CONFIG")
with open(path, "r") as f:
    content = f.read()

# Remove ocorrências antigas para evitar duplicidade
old_ids = (
    '"com.wisk.cosmic-applet-disk-usage"',
    '"com.wisk.cosmic-applet-system-monitor"',
)
lines = [l for l in content.splitlines() if not any(app_id in l for app_id in old_ids)]
clean_content = "\n".join(lines)

# Insere na asa direita (segundo bloco do plugins_wings)
target = '], ['
applet_entry = '], [\n    "com.wisk.cosmic-applet-system-monitor",'
new_content = clean_content.replace(target, applet_entry, 1)

with open(path, "w") as f:
    f.write(new_content)
EOF
    echo "✅ Alinhado à direita na barra!"
fi

# Recarrega o painel caso esteja em execução para refletir imediatamente
if pgrep -x cosmic-panel >/dev/null 2>&1; then
    killall -HUP cosmic-panel 2>/dev/null || killall cosmic-panel 2>/dev/null || true
fi

echo ""
echo "🎉 cosmic-applet-system-monitor instalado com sucesso!"
echo ""
echo "📌 O monitor do sistema já está ativo na sua barra superior!"
echo "Caso queira reposicionar ou personalizar:"
echo "1. Clique no applet na barra para abrir a central detalhada de recursos e atalhos."
echo "2. Em Configurações do Sistema -> Painel -> Configurar Applets, você pode movê-lo de lugar."
