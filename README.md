# 🖥️ COSMIC Applet - System & Resource Monitor (CPU, RAM, GPU, DISK, NET)

Applet nativo em **Rust** para a barra superior do **COSMIC Desktop** (Pop!_OS / Linux) alinhado à direita que exibe em tempo real:
- **CPU**: Uso do processador (ex: `CPU 2%`)
- **RAM**: Uso da memória (ex: `RAM 63%` ou `3.2G / 7.5G`)
- **GPU**: Uso da placa de vídeo NVIDIA / Intel / AMD (ex: `GPU 45%` ou `GPU 45% 61°C`)
- **DISCO**: Espaço ocupado e total do sistema (ex: `DISK 377G / 460G`)
- **INTERNET / REDE**: Velocidade de download e upload em tempo real (ex: `↓ 0.0M/s ↑ 0.0M/s`)

Tudo em um único monitor compacto, elegante e alinhado à direita da barra do sistema.

![Ícone do Applet](resources/com.wisk.cosmic-applet-disk-usage.svg)

---

## ✨ Recursos

### 1. Barra Superior (Alinhada à Direita)
```text
CPU 2%   RAM 63%   GPU 45%   DISK 377G / 460G   ↓ 0.0M/s ↑ 0.0M/s
```
- **Alinhamento à Direita**: Posicionado na asa direita do painel do COSMIC (ao lado da bandeja e status do sistema).
- **Cores dinâmicas**:
  - **Normal (< 75%)**: Neutro / Branco limpo.
  - **Alerta (≥ 75%)**: Pêssego / Laranja (aviso de carga moderada/alta).
  - **Crítico (≥ 90%)**: Vermelho vibrante.
- **Toggles individuais**: Ligue ou desligue qualquer indicador (`CPU`, `RAM`, `GPU`, `Disco` ou `Internet`) com 1 clique na janela do applet.

### 2. Central Popup Detalhada
Ao clicar no monitor na barra:
- **Processador (CPU)**: Gráfico de progresso e porcentagem de uso.
- **Memória RAM**: Quantidade usada, livre, total e porcentagem.
- **Placa de Vídeo (GPU)**:
  - Modelo detectado (ex: `NVIDIA GeForce RTX 3050`).
  - Uso da GPU em tempo real.
  - VRAM utilizada e total (`MiB / MiB`).
  - Temperatura da GPU em `°C`.
  - Opção para exibir a temperatura direto na barra (`GPU 45% 61°C`).
- **Armazenamento (Disco)**:
  - Capacidade e barra de progresso da partição raiz (`/`).
  - Seletor rápido para monitorar outras partições (`/boot/efi`, `/recovery`, discos externos).
- **Rede & Internet**:
  - Download e upload atuais.
  - Tráfego total transferido na sessão.
- **Atalhos**: Monitor do Sistema (`gnome-system-monitor`), Arquivos (`cosmic-files`), Discos (`gnome-disks`) e Analisador (`baobab`).

---

## 🚀 Instalação e Atualização

Na pasta `~/Documents/GitHub/cosmic-applet-disk-usage`:

```bash
./install.sh
```

---

## 🗑️ Desinstalação

```bash
./uninstall.sh
```
