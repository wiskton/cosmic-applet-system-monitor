use std::time::Duration;
use cosmic::app::{Core, Task};
use cosmic::iced::core::window;
use cosmic::iced::window::Id;
use cosmic::iced::{time, Alignment, Length, Rectangle, Subscription};
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::widget::{button, column, divider, progress_bar, row, scrollable, text};
use cosmic::Element;

fn horizontal_space() -> cosmic::iced::widget::Space {
    cosmic::iced::widget::Space::new().width(Length::Fill)
}

use crate::config::{AppConfig, DisplayFormat};
use crate::disk::{format_bytes, get_disks, DiskInfo};

pub struct DiskApplet {
    core: Core,
    popup: Option<Id>,
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
    SelectMount(String),
    SetDisplayFormat(DisplayFormat),
    SetLabel(String),
    ToggleDecimals,
    LaunchTool(&'static str, String),
}

impl DiskApplet {
    fn current_disk(&self) -> Option<&DiskInfo> {
        self.disks
            .iter()
            .find(|d| d.mount_point == self.config.mount_point)
            .or_else(|| self.disks.first())
    }

    fn save_config(&self) {
        let _ = self.config.save();
    }

    fn render_popup(&self) -> Element<'_, Message> {
        let current = self.current_disk();

        let header = row![
            text("💽 Armazenamento & Disco")
                .size(17)
                .font(cosmic::font::bold()),
            horizontal_space(),
            button::standard("↻ Atualizar")
                .padding([3, 10])
                .on_press(Message::Refresh),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        // Seção principal do disco selecionado
        let main_card = if let Some(d) = current {
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

            let info_row = row![
                text(format!("Ponto: {}", d.mount_point))
                    .size(13)
                    .font(cosmic::font::bold()),
                text(format!("({})", d.filesystem))
                    .size(12)
                    .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.6, 0.6, 0.6))),
                horizontal_space(),
                text(&d.device)
                    .size(11)
                    .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.5, 0.5, 0.5))),
            ]
            .align_y(Alignment::Center);

            let stats_row = row![
                column![
                    text("Usado").size(11).class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.6, 0.6, 0.6))),
                    text(used_str)
                        .size(18)
                        .font(cosmic::font::bold())
                        .class(cosmic::theme::Text::Color(status_color)),
                ]
                .spacing(2),
                column![
                    text("Livre").size(11).class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.6, 0.6, 0.6))),
                    text(free_str).size(18).font(cosmic::font::bold()),
                ]
                .spacing(2),
                column![
                    text("Total").size(11).class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.6, 0.6, 0.6))),
                    text(total_str).size(18).font(cosmic::font::bold()),
                ]
                .spacing(2),
                horizontal_space(),
                column![
                    text("Uso").size(11).class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.6, 0.6, 0.6))),
                    text(format!("{:.1}%", d.usage_percent))
                        .size(18)
                        .font(cosmic::font::bold())
                        .class(cosmic::theme::Text::Color(status_color)),
                ]
                .spacing(2)
                .align_x(Alignment::End),
            ]
            .spacing(20)
            .align_y(Alignment::Center);

            let pbar = progress_bar::determinate_linear(pct_val);

            column![info_row, stats_row, pbar].spacing(10)
        } else {
            column![text("Nenhum disco detectado").size(14)]
        };

        // Lista de outros discos / partições montadas
        let mut partitions_col = column![].spacing(6);
        let current_mount = self.config.mount_point.clone();

        for d in &self.disks {
            let is_active = d.mount_point == current_mount;
            let decimals = if self.config.show_decimals { 1 } else { 0 };
            let used_str = format_bytes(d.used_bytes, decimals);
            let total_str = format_bytes(d.total_bytes, decimals);

            let row_color = if d.is_critical(self.config.crit_percent) {
                cosmic::iced::Color::from_rgb(0.95, 0.3, 0.3)
            } else if d.is_warning(self.config.warn_percent) {
                cosmic::iced::Color::from_rgb(0.96, 0.62, 0.45)
            } else {
                cosmic::iced::Color::from_rgb(0.8, 0.8, 0.8)
            };

            let mount_lbl = if is_active {
                format!("● {}", d.mount_point)
            } else {
                format!("  {}", d.mount_point)
            };

            let target_mount = d.mount_point.clone();
            let select_btn = if !is_active {
                button::standard("Monitorar")
                    .padding([2, 8])
                    .on_press(Message::SelectMount(target_mount))
            } else {
                button::standard("Na barra")
                    .padding([2, 8])
            };

            let part_row = row![
                text(mount_lbl).size(13).font(cosmic::font::bold()),
                text(format!("{} / {}", used_str, total_str))
                    .size(12)
                    .class(cosmic::theme::Text::Color(row_color)),
                text(format!("({:.0}%)", d.usage_percent))
                    .size(12)
                    .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.6, 0.6, 0.6))),
                horizontal_space(),
                select_btn,
            ]
            .spacing(8)
            .align_y(Alignment::Center);

            partitions_col = partitions_col.push(part_row);
        }

        let partitions_section = column![
            text("Partições & Discos Montados")
                .size(13)
                .font(cosmic::font::bold())
                .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.7, 0.7, 0.7))),
            scrollable(partitions_col).height(Length::Fixed(110.0)),
        ]
        .spacing(6);

        // Atalhos para ferramentas do sistema
        let mount_target = current.map(|d| d.mount_point.clone()).unwrap_or_else(|| "/".to_string());
        let tools_row = row![
            text("Abrir:")
                .size(12)
                .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.6, 0.6, 0.6))),
            button::standard("📁 Arquivos")
                .padding([3, 10])
                .on_press(Message::LaunchTool("cosmic-files", mount_target.clone())),
            button::standard("📊 Analisador de Disco")
                .padding([3, 10])
                .on_press(Message::LaunchTool("baobab", mount_target)),
            button::standard("⚙️ Gerenciador de Discos")
                .padding([3, 10])
                .on_press(Message::LaunchTool("gnome-disks", String::new())),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        // Seletor de Formato de Exibição na Barra
        let format_row = row![
            text("Formato:")
                .size(12)
                .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.6, 0.6, 0.6))),
            button::standard("377G / 460G")
                .padding([2, 8])
                .on_press(Message::SetDisplayFormat(DisplayFormat::UsedSlashTotal)),
            button::standard("377G / 460G (86%)")
                .padding([2, 8])
                .on_press(Message::SetDisplayFormat(DisplayFormat::UsedSlashTotalPercent)),
            button::standard("377G/460G")
                .padding([2, 8])
                .on_press(Message::SetDisplayFormat(DisplayFormat::Compact)),
            button::standard("Livre: 60G")
                .padding([2, 8])
                .on_press(Message::SetDisplayFormat(DisplayFormat::FreeSlashTotal)),
            button::standard("86%")
                .padding([2, 8])
                .on_press(Message::SetDisplayFormat(DisplayFormat::PercentOnly)),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        // Seletor de Rótulo / Prefixo
        let label_row = row![
            text("Texto:")
                .size(12)
                .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.6, 0.6, 0.6))),
            button::standard("DISK")
                .padding([2, 8])
                .on_press(Message::SetLabel("DISK".to_string())),
            button::standard("SSD")
                .padding([2, 8])
                .on_press(Message::SetLabel("SSD".to_string())),
            button::standard("HD")
                .padding([2, 8])
                .on_press(Message::SetLabel("HD".to_string())),
            button::standard("💾")
                .padding([2, 8])
                .on_press(Message::SetLabel("💾".to_string())),
            button::standard("Nenhum")
                .padding([2, 8])
                .on_press(Message::SetLabel("".to_string())),
            horizontal_space(),
            button::standard(if self.config.show_decimals {
                "Decimais: 1 (ex: 376.5G)"
            } else {
                "Decimais: 0 (ex: 377G)"
            })
            .padding([2, 8])
            .on_press(Message::ToggleDecimals),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        // Rodapé com auto-refresh
        let footer = row![
            text("Atualização automática a cada 3s")
                .size(11)
                .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.5, 0.5, 0.5))),
            horizontal_space(),
            text(format!("Monitorando: {}", self.config.mount_point))
                .size(11)
                .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.5, 0.5, 0.5))),
        ]
        .align_y(Alignment::Center);

        let content = column![
            header,
            divider::horizontal::default(),
            main_card,
            divider::horizontal::default(),
            partitions_section,
            divider::horizontal::default(),
            tools_row,
            divider::horizontal::default(),
            format_row,
            label_row,
            divider::horizontal::default(),
            footer,
        ]
        .spacing(10)
        .padding(16);

        Element::from(self.core.applet.popup_container(content))
    }
}

impl cosmic::Application for DiskApplet {
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
        core.applet.window_size(180, 36);

        let config = AppConfig::load();
        let disks = get_disks();

        let app = Self {
            core,
            popup: None,
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
                if self.tick_counter % self.config.refresh_interval_secs.max(1) == 0 {
                    self.disks = get_disks();
                }
            }
            Message::Refresh => {
                self.disks = get_disks();
            }
            Message::SelectMount(mount) => {
                self.config.mount_point = mount;
                self.save_config();
                self.disks = get_disks();
            }
            Message::SetDisplayFormat(fmt) => {
                self.config.display_format = fmt;
                self.save_config();
            }
            Message::SetLabel(lbl) => {
                self.config.label = lbl;
                self.save_config();
            }
            Message::ToggleDecimals => {
                self.config.show_decimals = !self.config.show_decimals;
                self.save_config();
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

        let content: Element<Self::Message> = if let Some(d) = self.current_disk() {
            let decimals = if self.config.show_decimals { 1 } else { 0 };
            let used_str = format_bytes(d.used_bytes, decimals);
            let total_str = format_bytes(d.total_bytes, decimals);
            let free_str = format_bytes(d.avail_bytes, decimals);

            // Determina a cor do valor de uso (semelhante ao RAM 63% pêssego)
            let used_color = if d.is_critical(self.config.crit_percent) {
                cosmic::iced::Color::from_rgb(0.95, 0.32, 0.32) // Vermelho alerta
            } else if d.is_warning(self.config.warn_percent) {
                cosmic::iced::Color::from_rgb(0.96, 0.62, 0.45) // Pêssego/Laranja alerta moderado
            } else {
                cosmic::iced::Color::from_rgb(0.9, 0.9, 0.92) // Branco/Neutro limpo
            };

            let muted_color = cosmic::iced::Color::from_rgb(0.65, 0.65, 0.68);
            let label_color = cosmic::iced::Color::from_rgb(0.8, 0.8, 0.82);

            let mut r = row![].spacing(4).align_y(Alignment::Center);

            if !self.config.label.is_empty() {
                r = r.push(
                    text(&self.config.label)
                        .size(13)
                        .font(cosmic::font::bold())
                        .class(cosmic::theme::Text::Color(label_color)),
                );
            }

            match self.config.display_format {
                DisplayFormat::UsedSlashTotal => {
                    r = r
                        .push(
                            text(used_str)
                                .size(13)
                                .font(cosmic::font::bold())
                                .class(cosmic::theme::Text::Color(used_color)),
                        )
                        .push(
                            text("/")
                                .size(12)
                                .class(cosmic::theme::Text::Color(muted_color)),
                        )
                        .push(
                            text(total_str)
                                .size(13)
                                .font(cosmic::font::bold())
                                .class(cosmic::theme::Text::Color(muted_color)),
                        );
                }
                DisplayFormat::UsedSlashTotalPercent => {
                    r = r
                        .push(
                            text(used_str)
                                .size(13)
                                .font(cosmic::font::bold())
                                .class(cosmic::theme::Text::Color(used_color)),
                        )
                        .push(
                            text("/")
                                .size(12)
                                .class(cosmic::theme::Text::Color(muted_color)),
                        )
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
                                .class(cosmic::theme::Text::Color(used_color)),
                        );
                }
                DisplayFormat::Compact => {
                    r = r
                        .push(
                            text(format!("{}/{}", used_str, total_str))
                                .size(13)
                                .font(cosmic::font::bold())
                                .class(cosmic::theme::Text::Color(used_color)),
                        );
                }
                DisplayFormat::FreeSlashTotal => {
                    r = r
                        .push(
                            text("Livre:")
                                .size(12)
                                .class(cosmic::theme::Text::Color(muted_color)),
                        )
                        .push(
                            text(free_str)
                                .size(13)
                                .font(cosmic::font::bold())
                                .class(cosmic::theme::Text::Color(used_color)),
                        );
                }
                DisplayFormat::PercentOnly => {
                    r = r.push(
                        text(format!("{:.0}%", d.usage_percent))
                            .size(13)
                            .font(cosmic::font::bold())
                            .class(cosmic::theme::Text::Color(used_color)),
                    );
                }
            }

            r.into()
        } else {
            row![
                text("DISK")
                    .size(13)
                    .font(cosmic::font::bold()),
                text("--/--")
                    .size(13)
                    .class(cosmic::theme::Text::Color(cosmic::iced::Color::from_rgb(0.6, 0.6, 0.6))),
            ]
            .spacing(4)
            .align_y(Alignment::Center)
            .into()
        };

        let btn = button::custom(content)
            .padding([0, 6])
            .class(cosmic::theme::Button::AppletIcon)
            .on_press_with_rectangle(move |offset, bounds| {
                if let Some(id) = have_popup {
                    Message::Surface(destroy_popup(id))
                } else {
                    Message::Surface(app_popup::<DiskApplet>(
                        |_| Default::default(),
                        move |state: &mut DiskApplet| {
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
                        Some(Box::new(move |state: &DiskApplet| {
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
