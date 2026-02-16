use common::config::AppConfig;
use iced::Element;
use iced::widget::{column, text};

#[derive(Debug, Clone)]
pub struct State {
    pub auto_start: bool,
    pub language: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Message {
    ToggleAutoStart(bool),
    LanguageSelected(String),
}

impl State {
    pub fn from_config(config: &AppConfig) -> Self {
        Self {
            auto_start: config.general.auto_start,
            language: config.general.language.clone(),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::ToggleAutoStart(v) => self.auto_start = v,
            Message::LanguageSelected(lang) => self.language = lang,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        column![text("General Settings").size(24),]
            .spacing(12)
            .padding(20)
            .into()
    }

    pub fn apply_to(&self, config: &mut AppConfig) {
        config.general.auto_start = self.auto_start;
        config.general.language.clone_from(&self.language);
    }
}
