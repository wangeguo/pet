use crate::views::{self, Page};
use common::config::AppConfig;
use common::paths::AppPaths;
use iced::widget::{button, column, container, row, rule, text};
use iced::{Element, Length, Task, Theme};
use tracing::{error, info};

pub struct SettingsApp {
    paths: AppPaths,
    config: AppConfig,
    current_page: Page,
    general_state: views::general::State,
    appearance_state: views::appearance::State,
    ai_state: views::ai::State,
    meshy_state: views::meshy::State,
    dirty: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    NavigateTo(Page),
    Save,
    General(views::general::Message),
    Appearance(views::appearance::Message),
    Ai(views::ai::Message),
    Meshy(views::meshy::Message),
}

impl SettingsApp {
    pub fn new() -> (Self, Task<Message>) {
        let paths = AppPaths::new().expect("Failed to initialize paths");
        let config = AppConfig::load(&paths).unwrap_or_default();

        let general_state = views::general::State::from_config(&config);
        let appearance_state = views::appearance::State::from_config(&config);
        let ai_state = views::ai::State::from_config(&config);
        let meshy_state = views::meshy::State::from_config(&config);

        (
            Self {
                paths,
                config,
                current_page: Page::General,
                general_state,
                appearance_state,
                ai_state,
                meshy_state,
                dirty: false,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NavigateTo(page) => {
                self.current_page = page;
            }
            Message::Save => {
                self.apply_state_to_config();
                if let Err(e) = self.config.save(&self.paths) {
                    error!("Failed to save config: {e}");
                } else {
                    info!("Config saved");
                    self.dirty = false;
                }
            }
            Message::General(msg) => {
                self.general_state.update(msg);
                self.dirty = true;
            }
            Message::Appearance(msg) => {
                self.appearance_state.update(msg);
                self.dirty = true;
            }
            Message::Ai(msg) => {
                self.ai_state.update(msg);
                self.dirty = true;
            }
            Message::Meshy(msg) => {
                self.meshy_state.update(msg);
                self.dirty = true;
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let sidebar = self.sidebar();

        let content: Element<'_, Message> = match self.current_page {
            Page::General => self.general_state.view().map(Message::General),
            Page::Appearance => self.appearance_state.view().map(Message::Appearance),
            Page::Ai => self.ai_state.view().map(Message::Ai),
            Page::Meshy => self.meshy_state.view().map(Message::Meshy),
            Page::About => views::about::view(),
        };

        let save_bar: Element<'_, Message> = if self.dirty {
            container(button(text("Save")).on_press(Message::Save))
                .padding(10)
                .into()
        } else {
            column![].into()
        };

        let main_area = column![content, save_bar]
            .width(Length::Fill)
            .height(Length::Fill);

        row![sidebar, rule::vertical(1), main_area]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn theme(&self) -> Theme {
        Theme::Light
    }

    fn sidebar(&self) -> Element<'_, Message> {
        let pages = [
            (Page::General, "General"),
            (Page::Appearance, "Appearance"),
            (Page::Ai, "AI"),
            (Page::Meshy, "Meshy AI"),
            (Page::About, "About"),
        ];

        let buttons: Vec<Element<'_, Message>> = pages
            .iter()
            .map(|(page, label)| {
                button(text(*label))
                    .on_press(Message::NavigateTo(*page))
                    .width(Length::Fill)
                    .into()
            })
            .collect();

        container(column(buttons).spacing(4).padding(10).width(160))
            .height(Length::Fill)
            .into()
    }

    fn apply_state_to_config(&mut self) {
        self.general_state.apply_to(&mut self.config);
        self.appearance_state.apply_to(&mut self.config);
        self.ai_state.apply_to(&mut self.config);
        self.meshy_state.apply_to(&mut self.config);
    }
}
