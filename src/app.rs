use std::time::Duration;
use cosmic::app::{Core, Task};
use cosmic::iced::core::window;
use cosmic::iced::window::Id;
use cosmic::iced::{time, Alignment, Length, Rectangle, Subscription};
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::widget::{button, column, divider, progress_bar, row, text};
use cosmic::Element;

use crate::config::{AppConfig, DisplayFormat, RamFormat};
use crate::disk::{format_bytes, get_disks, DiskInfo};
use crate::system::{format_net_speed, read_memory, CpuMonitor, GpuMonitor, MemInfo, NetMonitor};

fn horizontal_space() -> cosmic::iced::widget::Space {
    cosmic::iced::widget::Space::new().width(Length::Fill)
}

pub struct SystemMonitorApplet {
    core: Core,
    popup: Option<Id>,
    cpu: CpuMonitor,
    mem: MemInfo,
    gpu: GpuMonitor,
    net: NetMonitor,
    disks: Vec<DiskInfo>,
    config: AppConfig,
    tick_counter: u64,
}

#[derive(Clone, Debug)]
pub enum Message {
    PopupClosed(Id),
    Surface(cosmic::surface::Action<Message>),
    Tick,
    Refresh,
    ToggleCpu,
    ToggleRam,
    ToggleGpu,
    ToggleGpuTemp,
    ToggleDisk,
    ToggleNet,
    SetDisplayFormat(DisplayFormat),
    SetRamFormat(RamFormat),
    SetDiskLabel(String),
    ToggleDecimals,
    SelectMount(String),
    LaunchTool(&'static str, String),
}

impl SystemMonitorApplet {
    fn current_disk(&self) -> Option<&DiskInfo> {
        self.disks
            .iter()
            .find(|d| d.mount_point == self.config.disk_mount_point)
            .or_else(|| self.disks.first())
    }

    fn save_config(&self) {
        let _ = self.config.save();
    }

    fn render_popup(&self) -> Element<'_, Message> {
        let header = row![
            text("💻 Monitor do Sistema & Recursos")
                .size(17)
                .font(cosmic::font::bold()),
            horizontal_space(),
            button::standard("↻ Atualizar")
                .padding([3, 10])
                .on_press(Message::Refresh),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        // --- 1. CPU Card ---
        let cpu_color = if self.cpu.usage_percent >= self.config.crit_percent {
            cosmic::iced::Color::from_rgb(0.95, 0.3, 0.3)
        } else if self.cpu.usage_percent >= self.config.warn_percent {
            cosmic::iced::Color::from_rgb(0.96, 0.62, 0.45)
        } else {
            cosmic::iced::Color::from_rgb(0.3, 0.8, 0.5)
        };

        let cpu_pct_val = (self.cpu.usage_percent as f32 / 100.0).clamp(0.0, 1.0);
        let cpu_row = row![
            text("Processador (CPU)").size(13).font(cosmic::font::bold()),
            horizontal_space(),
            text(format!("{:.1}%", self.cpu.usage_percent))
                .size(14)
                .font(cosmic::font::bold())
                .class(cosmic::theme::Text::Color(cpu_color)),
        ]
        .align_y(Alignment::Center);
        let cpu_bar = progress_bar::determinate_linear(cpu_pct_val);
        let cpu_section = column![cpu_row, cpu_bar].spacing(4);

        // --- 2. RAM Card ---
        let ram_color = if self.mem.usage_percent >= self.config.crit_percent {
            cosmic::iced::Color::from_rgb(0.95, 0.3, 0.3)
        } else if self.mem.usage_percent >= self.config.warn_percent {
            cosmic::iced::Color::from_rgb(0.96, 0.62, 0.45)
        } else {
            cosmic::iced::Color::from_rgb(0.3, 0.8, 0.5)
        };
        let ram_pct_val = (self.mem.usage_percent as f32 / 100.0).clamp(0.0, 1.0);
        let ram_used = format_bytes(self.mem.used_bytes, 1);
        let ram_total = format_bytes(self.mem.total_bytes, 1);
        let ram_avail = format_bytes(self.mem.avail_bytes, 1);

        let ram_row = row![
            text("Memória RAM").size(13).font(cosmic::font::bold()),
            text(format!("({} usado / {} total, {} livre)", ram_used, ram_total, ram_avail))
                .size(11)
                .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.6, 0.6, 0.6))),
            horizontal_space(),
            text(format!("{:.1}%", self.mem.usage_percent))
                .size(14)
                .font(cosmic::font::bold())
                .class(cosmic::theme::Text::Color(ram_color)),
        ]
        .align_y(Alignment::Center);
        let ram_bar = progress_bar::determinate_linear(ram_pct_val);
        let ram_section = column![ram_row, ram_bar].spacing(4);

        // --- 3. GPU Card ---
        let gpu_info = self.gpu.get();
        let gpu_section = if let Some(gpu) = gpu_info {
            let gpu_color = if gpu.usage_percent >= self.config.crit_percent {
                cosmic::iced::Color::from_rgb(0.95, 0.3, 0.3)
            } else if gpu.usage_percent >= self.config.warn_percent {
                cosmic::iced::Color::from_rgb(0.96, 0.62, 0.45)
            } else {
                cosmic::iced::Color::from_rgb(0.3, 0.8, 0.5)
            };
            let gpu_pct_val = (gpu.usage_percent as f32 / 100.0).clamp(0.0, 1.0);

            let vram_text = if gpu.vram_total_mb > 0 {
                let vram_pct = (gpu.vram_used_mb as f64 / gpu.vram_total_mb as f64) * 100.0;
                format!("VRAM: {} MiB / {} MiB ({:.0}%)  |  ", gpu.vram_used_mb, gpu.vram_total_mb, vram_pct)
            } else {
                String::new()
            };

            let temp_text = if gpu.temperature_c > 0 {
                format!("Temp: {}°C", gpu.temperature_c)
            } else {
                String::new()
            };

            let gpu_row = row![
                text(format!("Placa de Vídeo ({})", gpu.name))
                    .size(13)
                    .font(cosmic::font::bold()),
                text(format!("{}{}", vram_text, temp_text))
                    .size(11)
                    .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.6, 0.6, 0.6))),
                horizontal_space(),
                text(format!("{:.1}%", gpu.usage_percent))
                    .size(14)
                    .font(cosmic::font::bold())
                    .class(cosmic::theme::Text::Color(gpu_color)),
            ]
            .align_y(Alignment::Center);

            let gpu_bar = progress_bar::determinate_linear(gpu_pct_val);
            column![gpu_row, gpu_bar].spacing(4)
        } else {
            column![
                text("GPU").size(13).font(cosmic::font::bold()),
                text("Nenhuma GPU dedicada com suporte detectada").size(11).class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.5, 0.5, 0.5)))
            ].spacing(2)
        };

        // --- 4. DISCO Card ---
        let current = self.current_disk();
        let disk_section = if let Some(d) = current {
            let decimals = if self.config.show_decimals { 1 } else { 0 };
            let used_str = format_bytes(d.used_bytes, decimals);
            let total_str = format_bytes(d.total_bytes, decimals);
            let free_str = format_bytes(d.avail_bytes, decimals);
            let pct_val = (d.usage_percent as f32 / 100.0).clamp(0.0, 1.0);

            let status_color = if d.is_critical(self.config.crit_percent) {
                cosmic::iced::Color::from_rgb(0.95, 0.3, 0.3)
            } else if d.is_warning(self.config.warn_percent) {
                cosmic::iced::Color::from_rgb(0.96, 0.62, 0.45)
            } else {
                cosmic::iced::Color::from_rgb(0.3, 0.8, 0.5)
            };

            let disk_header = row![
                text(format!("Disco ({})", d.mount_point))
                    .size(13)
                    .font(cosmic::font::bold()),
                text(format!("{} / {} (livre: {})", used_str, total_str, free_str))
                    .size(11)
                    .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.6, 0.6, 0.6))),
                horizontal_space(),
                text(format!("{:.1}%", d.usage_percent))
                    .size(14)
                    .font(cosmic::font::bold())
                    .class(cosmic::theme::Text::Color(status_color)),
            ]
            .align_y(Alignment::Center);

            let d_bar = progress_bar::determinate_linear(pct_val);
            let mut col = column![disk_header, d_bar].spacing(4);

            if self.disks.len() > 1 {
                let mut part_buttons = row![
                    text("Partição:")
                        .size(11)
                        .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.6, 0.6, 0.6))),
                ]
                .spacing(4)
                .align_y(Alignment::Center);
                for p in &self.disks {
                    let is_curr = p.mount_point == d.mount_point;
                    let label = if is_curr {
                        format!("● {}", p.mount_point)
                    } else {
                        p.mount_point.clone()
                    };
                    part_buttons = part_buttons.push(
                        button::standard(label)
                            .padding([1, 6])
                            .on_press(Message::SelectMount(p.mount_point.clone())),
                    );
                }
                col = col.push(part_buttons);
            }

            col
        } else {
            column![text("Nenhum disco detectado").size(12)]
        };

        // --- 5. INTERNET / REDE Card ---
        let net_rx_str = format_net_speed(self.net.rx_speed);
        let net_tx_str = format_net_speed(self.net.tx_speed);
        let total_rx_str = format_bytes(self.net.total_rx, 1);
        let total_tx_str = format_bytes(self.net.total_tx, 1);

        let net_row = row![
            text("Rede & Internet").size(13).font(cosmic::font::bold()),
            horizontal_space(),
            text(format!("↓ {}", net_rx_str))
                .size(13)
                .font(cosmic::font::bold())
                .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.4, 0.75, 1.0))),
            text(format!("↑ {}", net_tx_str))
                .size(13)
                .font(cosmic::font::bold())
                .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.3, 0.85, 0.6))),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        let net_totals = row![
            text(format!("Total na sessão: ↓ {}  |  ↑ {}", total_rx_str, total_tx_str))
                .size(11)
                .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.55, 0.55, 0.55))),
        ];
        let net_section = column![net_row, net_totals].spacing(4);

        // --- 6. Configuração dos Itens na Barra ---
        let toggles_row = row![
            text("Mostrar na barra:")
                .size(12)
                .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.65, 0.65, 0.65))),
            button::standard(if self.config.show_cpu { "✓ CPU" } else { "CPU" })
                .padding([2, 8])
                .on_press(Message::ToggleCpu),
            button::standard(if self.config.show_ram { "✓ RAM" } else { "RAM" })
                .padding([2, 8])
                .on_press(Message::ToggleRam),
            button::standard(if self.config.show_gpu { "✓ GPU" } else { "GPU" })
                .padding([2, 8])
                .on_press(Message::ToggleGpu),
            button::standard(if self.config.show_disk { "✓ Disco" } else { "Disco" })
                .padding([2, 8])
                .on_press(Message::ToggleDisk),
            button::standard(if self.config.show_net { "✓ Internet" } else { "Internet" })
                .padding([2, 8])
                .on_press(Message::ToggleNet),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        // Opções da GPU
        let gpu_options_row = row![
            text("Opções GPU:")
                .size(12)
                .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.65, 0.65, 0.65))),
            button::standard(if self.config.show_gpu_temp { "Exibir Temp: Ligado (ex: 55% 61°C)" } else { "Exibir Temp: Desligado (ex: 55%)" })
                .padding([2, 8])
                .on_press(Message::ToggleGpuTemp),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        // Seletor de Formato de Disco
        let disk_format_row = row![
            text("Formato Disco:")
                .size(12)
                .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.65, 0.65, 0.65))),
            button::standard("377G / 460G")
                .padding([2, 6])
                .on_press(Message::SetDisplayFormat(DisplayFormat::UsedSlashTotal)),
            button::standard("377G/460G")
                .padding([2, 6])
                .on_press(Message::SetDisplayFormat(DisplayFormat::Compact)),
            button::standard("Livre: 60G")
                .padding([2, 6])
                .on_press(Message::SetDisplayFormat(DisplayFormat::FreeSlashTotal)),
            button::standard("86%")
                .padding([2, 6])
                .on_press(Message::SetDisplayFormat(DisplayFormat::PercentOnly)),
            horizontal_space(),
            button::standard(if self.config.show_decimals { "Decimais: 1" } else { "Decimais: 0" })
                .padding([2, 6])
                .on_press(Message::ToggleDecimals),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        // Seletor de Rótulo do Disco
        let label_row = row![
            text("Rótulo Disco:")
                .size(12)
                .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.65, 0.65, 0.65))),
            button::standard("DISK")
                .padding([2, 6])
                .on_press(Message::SetDiskLabel("DISK".to_string())),
            button::standard("SSD")
                .padding([2, 6])
                .on_press(Message::SetDiskLabel("SSD".to_string())),
            button::standard("HD")
                .padding([2, 6])
                .on_press(Message::SetDiskLabel("HD".to_string())),
            button::standard("💾")
                .padding([2, 6])
                .on_press(Message::SetDiskLabel("💾".to_string())),
            button::standard("Nenhum")
                .padding([2, 6])
                .on_press(Message::SetDiskLabel("".to_string())),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        // Seletor de Formato da RAM
        let ram_format_row = row![
            text("Formato RAM:")
                .size(12)
                .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.65, 0.65, 0.65))),
            button::standard("63%")
                .padding([2, 6])
                .on_press(Message::SetRamFormat(RamFormat::Percent)),
            button::standard("3.2G / 7.5G")
                .padding([2, 6])
                .on_press(Message::SetRamFormat(RamFormat::UsedSlashTotal)),
            button::standard("3.2G / 7.5G (42%)")
                .padding([2, 6])
                .on_press(Message::SetRamFormat(RamFormat::UsedSlashTotalPercent)),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        // Atalhos de Ferramentas do Sistema
        let mount_target = current.map(|d| d.mount_point.clone()).unwrap_or_else(|| "/".to_string());
        let tools_row = row![
            text("Atalhos:")
                .size(12)
                .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.65, 0.65, 0.65))),
            button::standard("📊 Monitor do Sistema")
                .padding([3, 10])
                .on_press(Message::LaunchTool("gnome-system-monitor", String::new())),
            button::standard("📁 Arquivos")
                .padding([3, 10])
                .on_press(Message::LaunchTool("cosmic-files", mount_target.clone())),
            button::standard("⚙️ Discos")
                .padding([3, 10])
                .on_press(Message::LaunchTool("gnome-disks", String::new())),
            button::standard("📈 Analisador")
                .padding([3, 10])
                .on_press(Message::LaunchTool("baobab", mount_target)),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        let footer = row![
            text("Atualização automática em tempo real (1s)")
                .size(11)
                .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.5, 0.5, 0.5))),
        ];

        let content = column![
            header,
            divider::horizontal::default(),
            cpu_section,
            divider::horizontal::default(),
            gpu_section,
            divider::horizontal::default(),
            ram_section,
            divider::horizontal::default(),
            disk_section,
            divider::horizontal::default(),
            net_section,
            divider::horizontal::default(),
            toggles_row,
            gpu_options_row,
            disk_format_row,
            label_row,
            ram_format_row,
            divider::horizontal::default(),
            tools_row,
            divider::horizontal::default(),
            footer,
        ]
        .spacing(10)
        .padding(16);

        Element::from(self.core.applet.popup_container(content))
    }
}

impl cosmic::Application for SystemMonitorApplet {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "com.wisk.cosmic-applet-disk-usage";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(mut core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        core.applet.window_size(360, 36);

        let config = AppConfig::load();
        let cpu = CpuMonitor::new();
        let mem = read_memory();
        let gpu = GpuMonitor::new();
        let net = NetMonitor::new();
        let disks = get_disks();

        let app = Self {
            core,
            popup: None,
            cpu,
            mem,
            gpu,
            net,
            disks,
            config,
            tick_counter: 0,
        };

        (app, Task::none())
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::Surface(a) => {
                return cosmic::task::message(cosmic::Action::Surface(a));
            }
            Message::Tick => {
                self.tick_counter += 1;
                self.cpu.update();
                self.mem = read_memory();
                self.net.update();

                // Atualiza discos a cada 3 segundos
                if self.tick_counter % 3 == 0 {
                    self.disks = get_disks();
                }
            }
            Message::Refresh => {
                self.cpu.update();
                self.mem = read_memory();
                self.net.update();
                self.disks = get_disks();
            }
            Message::ToggleCpu => {
                self.config.show_cpu = !self.config.show_cpu;
                self.save_config();
            }
            Message::ToggleRam => {
                self.config.show_ram = !self.config.show_ram;
                self.save_config();
            }
            Message::ToggleGpu => {
                self.config.show_gpu = !self.config.show_gpu;
                self.save_config();
            }
            Message::ToggleGpuTemp => {
                self.config.show_gpu_temp = !self.config.show_gpu_temp;
                self.save_config();
            }
            Message::ToggleDisk => {
                self.config.show_disk = !self.config.show_disk;
                self.save_config();
            }
            Message::ToggleNet => {
                self.config.show_net = !self.config.show_net;
                self.save_config();
            }
            Message::SetDisplayFormat(fmt) => {
                self.config.disk_format = fmt;
                self.save_config();
            }
            Message::SetRamFormat(fmt) => {
                self.config.ram_format = fmt;
                self.save_config();
            }
            Message::SetDiskLabel(lbl) => {
                self.config.disk_label = lbl;
                self.save_config();
            }
            Message::ToggleDecimals => {
                self.config.show_decimals = !self.config.show_decimals;
                self.save_config();
            }
            Message::SelectMount(mount) => {
                self.config.disk_mount_point = mount;
                self.save_config();
                self.disks = get_disks();
            }
            Message::LaunchTool(app_name, arg) => {
                let mut cmd = std::process::Command::new(app_name);
                if !arg.is_empty() {
                    cmd.arg(&arg);
                }
                let _ = cmd.spawn();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let have_popup = self.popup;

        let label_color = cosmic::iced::Color::from_rgb(0.78, 0.78, 0.82);
        let normal_val_color = cosmic::iced::Color::from_rgb(0.90, 0.90, 0.92);
        let muted_color = cosmic::iced::Color::from_rgb(0.60, 0.60, 0.64);
        let peach_warn = cosmic::iced::Color::from_rgb(0.96, 0.62, 0.45);
        let red_crit = cosmic::iced::Color::from_rgb(0.95, 0.32, 0.32);

        let mut panel_row = row![].spacing(10).align_y(Alignment::Center);

        // --- 1. CPU ---
        if self.config.show_cpu {
            let cpu_val_color = if self.cpu.usage_percent >= self.config.crit_percent {
                red_crit
            } else if self.cpu.usage_percent >= self.config.warn_percent {
                peach_warn
            } else {
                normal_val_color
            };

            let cpu_block = row![
                text("CPU")
                    .size(13)
                    .font(cosmic::font::bold())
                    .class(cosmic::theme::Text::Color(label_color)),
                text(format!("{:.0}%", self.cpu.usage_percent))
                    .size(13)
                    .font(cosmic::font::bold())
                    .class(cosmic::theme::Text::Color(cpu_val_color)),
            ]
            .spacing(4)
            .align_y(Alignment::Center);

            panel_row = panel_row.push(cpu_block);
        }

        // --- 2. GPU ---
        if self.config.show_gpu {
            if let Some(gpu) = self.gpu.get() {
                let gpu_val_color = if gpu.usage_percent >= self.config.crit_percent {
                    red_crit
                } else if gpu.usage_percent >= self.config.warn_percent {
                    peach_warn
                } else {
                    normal_val_color
                };

                let gpu_str = if self.config.show_gpu_temp && gpu.temperature_c > 0 {
                    format!("{:.0}% {}°C", gpu.usage_percent, gpu.temperature_c)
                } else {
                    format!("{:.0}%", gpu.usage_percent)
                };

                let gpu_block = row![
                    text("GPU")
                        .size(13)
                        .font(cosmic::font::bold())
                        .class(cosmic::theme::Text::Color(label_color)),
                    text(gpu_str)
                        .size(13)
                        .font(cosmic::font::bold())
                        .class(cosmic::theme::Text::Color(gpu_val_color)),
                ]
                .spacing(4)
                .align_y(Alignment::Center);

                panel_row = panel_row.push(gpu_block);
            }
        }

        // --- 3. RAM (Memória) ---
        if self.config.show_ram {
            let ram_val_color = if self.mem.usage_percent >= self.config.crit_percent {
                red_crit
            } else if self.mem.usage_percent >= self.config.warn_percent {
                peach_warn
            } else {
                normal_val_color
            };

            let mut ram_block = row![
                text("RAM")
                    .size(13)
                    .font(cosmic::font::bold())
                    .class(cosmic::theme::Text::Color(label_color)),
            ]
            .spacing(4)
            .align_y(Alignment::Center);

            match self.config.ram_format {
                RamFormat::Percent => {
                    ram_block = ram_block.push(
                        text(format!("{:.0}%", self.mem.usage_percent))
                            .size(13)
                            .font(cosmic::font::bold())
                            .class(cosmic::theme::Text::Color(ram_val_color)),
                    );
                }
                RamFormat::UsedSlashTotal => {
                    let u = format_bytes(self.mem.used_bytes, 1);
                    let t = format_bytes(self.mem.total_bytes, 1);
                    ram_block = ram_block
                        .push(
                            text(u)
                                .size(13)
                                .font(cosmic::font::bold())
                                .class(cosmic::theme::Text::Color(ram_val_color)),
                        )
                        .push(text("/").size(12).class(cosmic::theme::Text::Color(muted_color)))
                        .push(
                            text(t)
                                .size(13)
                                .font(cosmic::font::bold())
                                .class(cosmic::theme::Text::Color(muted_color)),
                        );
                }
                RamFormat::UsedSlashTotalPercent => {
                    let u = format_bytes(self.mem.used_bytes, 1);
                    let t = format_bytes(self.mem.total_bytes, 1);
                    ram_block = ram_block
                        .push(
                            text(format!("{}/{}", u, t))
                                .size(13)
                                .font(cosmic::font::bold())
                                .class(cosmic::theme::Text::Color(ram_val_color)),
                        )
                        .push(
                            text(format!("({:.0}%)", self.mem.usage_percent))
                                .size(12)
                                .font(cosmic::font::bold())
                                .class(cosmic::theme::Text::Color(ram_val_color)),
                        );
                }
            }

            panel_row = panel_row.push(ram_block);
        }

        // --- 4. DISK ---
        if self.config.show_disk {
            if let Some(d) = self.current_disk() {
                let decimals = if self.config.show_decimals { 1 } else { 0 };
                let used_str = format_bytes(d.used_bytes, decimals);
                let total_str = format_bytes(d.total_bytes, decimals);
                let free_str = format_bytes(d.avail_bytes, decimals);

                let disk_val_color = if d.is_critical(self.config.crit_percent) {
                    red_crit
                } else if d.is_warning(self.config.warn_percent) {
                    peach_warn
                } else {
                    normal_val_color
                };

                let mut disk_block = row![].spacing(4).align_y(Alignment::Center);

                if !self.config.disk_label.is_empty() {
                    disk_block = disk_block.push(
                        text(&self.config.disk_label)
                            .size(13)
                            .font(cosmic::font::bold())
                            .class(cosmic::theme::Text::Color(label_color)),
                    );
                }

                match self.config.disk_format {
                    DisplayFormat::UsedSlashTotal => {
                        disk_block = disk_block
                            .push(
                                text(used_str)
                                    .size(13)
                                    .font(cosmic::font::bold())
                                    .class(cosmic::theme::Text::Color(disk_val_color)),
                            )
                            .push(text("/").size(12).class(cosmic::theme::Text::Color(muted_color)))
                            .push(
                                text(total_str)
                                    .size(13)
                                    .font(cosmic::font::bold())
                                    .class(cosmic::theme::Text::Color(muted_color)),
                            );
                    }
                    DisplayFormat::UsedSlashTotalPercent => {
                        disk_block = disk_block
                            .push(
                                text(used_str)
                                    .size(13)
                                    .font(cosmic::font::bold())
                                    .class(cosmic::theme::Text::Color(disk_val_color)),
                            )
                            .push(text("/").size(12).class(cosmic::theme::Text::Color(muted_color)))
                            .push(
                                text(total_str)
                                    .size(13)
                                    .font(cosmic::font::bold())
                                    .class(cosmic::theme::Text::Color(muted_color)),
                            )
                            .push(
                                text(format!("({:.0}%)", d.usage_percent))
                                    .size(12)
                                    .font(cosmic::font::bold())
                                    .class(cosmic::theme::Text::Color(disk_val_color)),
                            );
                    }
                    DisplayFormat::Compact => {
                        disk_block = disk_block.push(
                            text(format!("{}/{}", used_str, total_str))
                                .size(13)
                                .font(cosmic::font::bold())
                                .class(cosmic::theme::Text::Color(disk_val_color)),
                        );
                    }
                    DisplayFormat::FreeSlashTotal => {
                        disk_block = disk_block
                            .push(text("Livre:").size(12).class(cosmic::theme::Text::Color(muted_color)))
                            .push(
                                text(free_str)
                                    .size(13)
                                    .font(cosmic::font::bold())
                                    .class(cosmic::theme::Text::Color(disk_val_color)),
                            );
                    }
                    DisplayFormat::PercentOnly => {
                        disk_block = disk_block.push(
                            text(format!("{:.0}%", d.usage_percent))
                                .size(13)
                                .font(cosmic::font::bold())
                                .class(cosmic::theme::Text::Color(disk_val_color)),
                        );
                    }
                }

                panel_row = panel_row.push(disk_block);
            }
        }

        // --- 5. INTERNET / REDE (↓ 0.0M/s ↑ 0.0M/s) ---
        if self.config.show_net {
            let rx_str = format_net_speed(self.net.rx_speed);
            let tx_str = format_net_speed(self.net.tx_speed);

            let net_block = row![
                text("↓")
                    .size(13)
                    .font(cosmic::font::bold())
                    .class(cosmic::theme::Text::Color(label_color)),
                text(rx_str)
                    .size(13)
                    .font(cosmic::font::bold())
                    .class(cosmic::theme::Text::Color(normal_val_color)),
                text("↑")
                    .size(13)
                    .font(cosmic::font::bold())
                    .class(cosmic::theme::Text::Color(label_color)),
                text(tx_str)
                    .size(13)
                    .font(cosmic::font::bold())
                    .class(cosmic::theme::Text::Color(normal_val_color)),
            ]
            .spacing(4)
            .align_y(Alignment::Center);

            panel_row = panel_row.push(net_block);
        }

        let btn = button::custom(panel_row)
            .padding([0, 6])
            .class(cosmic::theme::Button::AppletIcon)
            .on_press_with_rectangle(move |offset, bounds| {
                if let Some(id) = have_popup {
                    Message::Surface(destroy_popup(id))
                } else {
                    Message::Surface(app_popup::<SystemMonitorApplet>(
                        |_| Default::default(),
                        move |state: &mut SystemMonitorApplet| {
                            let new_id = Id::unique();
                            state.popup = Some(new_id);
                            let mut popup_settings = state.core.applet.get_popup_settings(
                                state.core.main_window_id().unwrap(),
                                new_id,
                                None,
                                None,
                                None,
                            );

                            popup_settings.positioner.anchor_rect = Rectangle {
                                x: (bounds.x - offset.x) as i32,
                                y: (bounds.y - offset.y) as i32,
                                width: bounds.width as i32,
                                height: bounds.height as i32,
                            };

                            popup_settings
                        },
                        Some(Box::new(move |state: &SystemMonitorApplet| {
                            state.render_popup().map(cosmic::Action::App)
                        })),
                    ))
                }
            });

        self.core.applet.autosize_window(btn).into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        time::every(Duration::from_secs(1)).map(|_| Message::Tick)
    }
}
