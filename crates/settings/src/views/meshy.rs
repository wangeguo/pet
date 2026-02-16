use common::config::AppConfig;
use iced::Element;
use iced::widget::{column, text};

#[derive(Debug, Clone)]
pub struct State {
    pub api_key: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Message {
    ApiKeyChanged(String),
}

impl State {
    pub fn from_config(config: &AppConfig) -> Self {
        Self {
            api_key: config.meshy.api_key.clone().unwrap_or_default(),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::ApiKeyChanged(v) => self.api_key = v,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        column![text("Meshy AI Settings").size(24),]
            .spacing(12)
            .padding(20)
            .into()
    }

    pub fn apply_to(&self, config: &mut AppConfig) {
        config.meshy.api_key = if self.api_key.is_empty() {
            None
        } else {
            Some(self.api_key.clone())
        };
    }
}
