# 🖥️ COSMIC Applet - System & Disk Monitor (CPU, RAM, DISK, NET)

Applet nativo em **Rust** para a barra superior do **COSMIC Desktop** (Pop!_OS / Linux) que exibe em tempo real:
- **CPU**: Porcentagem de uso (ex: `CPU 2%`)
- **RAM**: Porcentagem ou capacidade ocupada (ex: `RAM 63%` ou `3.2G / 7.5G`)
- **DISCO**: Espaço ocupado e total do sistema (ex: `DISK 377G / 460G`)
- **INTERNET / REDE**: Velocidade de download e upload em tempo real (ex: `↓ 0.0M/s ↑ 0.0M/s`)

Combinando todos os recursos em um único applet rápido, unificado e de baixo consumo de recursos.

![Ícone do Applet](resources/com.wisk.cosmic-applet-disk-usage.svg)

---

## ✨ Funcionalidades

### 1. Barra Superior (Top Bar)
- Visual idêntico ao padrão do COSMIC e do Pop!_OS:
  ```
  CPU 2%   RAM 63%   DISK 377G / 460G   ↓ 0.0M/s ↑ 0.0M/s
  ```
- **Cores inteligentes de alerta**:
  - **Normal (< 75%)**: Tom branco/neutro limpo.
  - **Alerta (≥ 75%)**: Tom pêssego/laranja (idêntico à referência de sistema).
  - **Crítico (≥ 90%)**: Vermelho vibrante.
- **Toggles individuais**: Você pode ligar ou desligar qualquer métrica (CPU, RAM, Disco ou Internet) com um clique na janela popup!
- **Formatos selecionáveis**:
  - **Disco**: `377G / 460G`, `377G/460G`, `Livre: 60G` ou `86%`.
  - **RAM**: `63%`, `3.2G / 7.5G` ou `3.2G / 7.5G (42%)`.
  - **Rótulo do Disco**: `DISK`, `SSD`, `HD`, `💾` ou sem rótulo.

### 2. Central Popup ao Clicar
Ao clicar em qualquer parte do monitor na barra superior, abre-se uma central completa:
- **CPU**: Gráfico de progresso com a porcentagem exata de processamento.
- **RAM**: Detalhamento com memória usada, livre, total e porcentagem.
- **Armazenamento / Discos**:
  - Barra gráfica de ocupação da partição ativa (`/`).
  - Botões para alternar entre outras partições montadas (`/boot/efi`, `/recovery`, discos externos/pendrives).
- **Rede / Internet**:
  - Velocidades atuais de Download (`↓`) e Upload (`↑`).
  - Total de tráfego transferido na sessão (`↓ X GB | ↑ Y GB`).
- **Atalhos Rápidos**:
  - 📊 **Monitor do Sistema** (`gnome-system-monitor`)
  - 📁 **Gerenciador de Arquivos** (`cosmic-files`)
  - ⚙️ **Gerenciador de Discos** (`gnome-disks`)
  - 📈 **Analisador de Uso de Disco** (`baobab`)

---

## 🚀 Como Instalar ou Atualizar

Dentro da pasta do projeto (`~/Documents/GitHub/cosmic-applet-disk-usage`):

```bash
./install.sh
```

O script compila (se necessário), instala em `~/.local/bin/`, configura os ícones e o `.desktop`, registra na barra do painel COSMIC e recarrega instantaneamente.

---

## ⚙️ Configuração

As configurações ficam salvas em:
`~/.config/cosmic-applet-disk-usage/config.json`

```json
{
  "show_cpu": true,
  "show_ram": true,
  "show_disk": true,
  "show_net": true,
  "disk_label": "DISK",
  "disk_mount_point": "/",
  "disk_format": "UsedSlashTotal",
  "ram_format": "Percent",
  "show_decimals": false,
  "warn_percent": 75.0,
  "crit_percent": 90.0
}
```

Você pode alterar tudo visualmente clicando no applet na barra do sistema.

---

## 🗑️ Desinstalação

Caso queira remover do painel e do sistema:

```bash
./uninstall.sh
```
