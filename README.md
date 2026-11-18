# 💽 COSMIC Applet - Disk Usage Monitor

Applet nativo em **Rust** para a barra superior do **COSMIC Desktop** (Pop!_OS / Linux) que exibe o espaço em disco ocupado e o total em tempo real, combinando perfeitamente com os monitores de CPU e RAM (`minimon` / `sysinfo`).

![Exemplo na barra](resources/com.wisk.cosmic-applet-disk-usage.svg)

---

## ✨ Funcionalidades

- **Exibição na Barra Superior**:
  - Mostra o espaço ocupado e o total do disco (ex: `DISK 377G / 460G`).
  - **Cores dinâmicas**:
    - **Normal (< 75%)**: Neutro / Branco limpo.
    - **Alerta (≥ 75%)**: Tom pêssego/laranja (idêntico ao alerta de RAM na imagem de referência).
    - **Crítico (≥ 90%)**: Vermelho vibrante.
  - Vários formatos selecionáveis com 1 clique:
    - `377G / 460G` (Padrão)
    - `377G / 460G (86%)`
    - `377G/460G` (Compacto)
    - `Livre: 60G`
    - `86%`
  - Rótulos customizáveis: `DISK`, `SSD`, `HD`, `💾` ou oculto.
  - Opção de alternar entre números inteiros (`377G`) e decimais (`376.5G`).

- **Popup Interativo ao Clicar**:
  - Barra de progresso visual do uso do disco principal (`/`).
  - Lista de todas as partições e mídias montadas (`/boot/efi`, `/recovery`, pendrives, discos externos).
  - Botão para trocar qual partição exibir na barra com 1 clique.
  - **Atalhos rápidos do sistema**:
    - 📁 Abrir no Gerenciador de Arquivos (`cosmic-files`).
    - 📊 Abrir Analisador de Uso de Disco (`baobab`).
    - ⚙️ Abrir Gerenciador de Discos (`gnome-disks`).
  - Botão de atualização imediata e atualização automática contínua a cada 3s.

- **Leve e Eficiente**:
  - Desenvolvido em Rust puro com `libcosmic` e chamadas diretas de sistema `statvfs`.
  - Zero uso perceptível de CPU (0.0%).

---

## 🚀 Instalação Rápida

Dentro da pasta do projeto, execute:

```bash
./install.sh
```

O script irá:
1. Compilar o binário em modo release otimizado.
2. Instalar o executável em `~/.local/bin/cosmic-applet-disk-usage`.
3. Instalar o arquivo `.desktop` e o ícone SVG no sistema.
4. Adicionar automaticamente o applet na barra superior do COSMIC (ao lado dos monitores de CPU/RAM).
5. Recarregar o painel para que o applet apareça imediatamente.

---

## ⚙️ Configuração Manual / Personalização

As preferências são salvas automaticamente em:
`~/.config/cosmic-applet-disk-usage/config.json`

Você pode alterar os formatos, rótulos e partições diretamente na janela do applet ao clicar nele na barra, ou editar o arquivo JSON:

```json
{
  "label": "DISK",
  "mount_point": "/",
  "display_format": "UsedSlashTotal",
  "show_decimals": false,
  "refresh_interval_secs": 3,
  "warn_percent": 75.0,
  "crit_percent": 90.0
}
```

Para mover ou reposicionar o applet na barra:
1. Abra **Configurações do Sistema -> Área de Trabalho -> Painel**.
2. Clique em **Configurar Applets** e arraste o **Disk Usage Applet** para a posição desejada.

---

## 🗑️ Desinstalação

Se desejar remover o applet da barra e do sistema:

```bash
./uninstall.sh
```

---

## 📂 Estrutura do Código

- `src/main.rs`: Ponto de entrada do applet COSMIC.
- `src/app.rs`: Interface gráfica (barra superior, estilo de texto/cores e janela popup).
- `src/disk.rs`: Leitura dos discos via `/proc/mounts` e medição via `libc::statvfs`.
- `src/config.rs`: Gerenciamento e persistência das configurações.
- `resources/`: Arquivo de integração `.desktop` e ícone SVG vetorial.
