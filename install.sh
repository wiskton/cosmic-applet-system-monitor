#!/usr/bin/env bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"

# Configura PKG_CONFIG_PATH caso dependências locais existam
if [ -d "$DIR/.local-deps/usr/lib/x86_64-linux-gnu/pkgconfig" ]; then
    export PKG_CONFIG_PATH="$DIR/.local-deps/usr/lib/x86_64-linux-gnu/pkgconfig:${PKG_CONFIG_PATH}"
fi

echo "🔨 Compilando o cosmic-applet-disk-usage em modo release..."
if [ ! -f "target/release/cosmic-applet-disk-usage" ]; then
    cargo build --release
else
    echo "⚡ Binário já compilado encontrado em target/release/cosmic-applet-disk-usage"
fi

echo "📦 Instalando binário e arquivos de integração do COSMIC..."
mkdir -p "$HOME/.local/bin"
mkdir -p "$HOME/.local/share/applications"
mkdir -p "$HOME/.local/share/icons/hicolor/scalable/apps"

# Para processo anterior se estiver rodando para permitir a substituição do binário
killall cosmic-applet-disk-usage 2>/dev/null || true

cp --remove-destination target/release/cosmic-applet-disk-usage "$HOME/.local/bin/"
chmod +x "$HOME/.local/bin/cosmic-applet-disk-usage"

# Reduz o tamanho do binário se strip estiver disponível
if command -v strip >/dev/null 2>&1; then
    strip "$HOME/.local/bin/cosmic-applet-disk-usage"
fi

# Ajusta o caminho no arquivo .desktop e instala
sed "s|Exec=cosmic-applet-disk-usage|Exec=$HOME/.local/bin/cosmic-applet-disk-usage|g" \
    resources/com.wisk.cosmic-applet-disk-usage.desktop > "$HOME/.local/share/applications/com.wisk.cosmic-applet-disk-usage.desktop"

cp resources/com.wisk.cosmic-applet-disk-usage.svg "$HOME/.local/share/icons/hicolor/scalable/apps/"

# Atualiza cache de aplicações do desktop
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$HOME/.local/share/applications" || true
fi

# Adiciona automaticamente ao painel do COSMIC caso o arquivo exista e ainda não contenha o applet
PANEL_CONFIG="$HOME/.config/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings"
if [ -f "$PANEL_CONFIG" ]; then
    if ! grep -q "com.wisk.cosmic-applet-disk-usage" "$PANEL_CONFIG"; then
        echo "⚙️ Adicionando automaticamente o applet à barra do COSMIC..."
        python3 - <<EOF
import os

path = os.path.expanduser("$PANEL_CONFIG")
with open(path, "r") as f:
    content = f.read()

# Posiciona logo após o minimon ou bitcoin
applet_entry = '    "com.wisk.cosmic-applet-disk-usage",\n'
target1 = '"com.wisk.cosmic-applet-bitcoin",'
target2 = '"io.github.cosmic_utils.minimon-applet",'

if target1 in content:
    new_content = content.replace(target1, target1 + '\n' + applet_entry, 1)
elif target2 in content:
    new_content = content.replace(target2, target2 + '\n' + applet_entry, 1)
else:
    new_content = content.replace('Some(([', 'Some(([\n' + applet_entry, 1)

with open(path, "w") as f:
    f.write(new_content)
EOF
        echo "✅ Adicionado ao painel!"
    fi
fi

# Recarrega o painel caso esteja em execução para refletir imediatamente
if pgrep -x cosmic-panel >/dev/null 2>&1; then
    killall -HUP cosmic-panel 2>/dev/null || killall cosmic-panel 2>/dev/null || true
fi

echo ""
echo "🎉 cosmic-applet-disk-usage instalado com sucesso!"
echo ""
echo "📌 O monitor de disco já está ativo na sua barra superior ao lado do CPU/RAM!"
echo "Caso queira reposicionar ou personalizar:"
echo "1. Clique no applet na barra para abrir a central detalhada de disco e atalhos."
echo "2. Em Configurações do Sistema -> Painel -> Configurar Applets, você pode movê-lo de lugar."
