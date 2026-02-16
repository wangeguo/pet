use common::config::AppConfig;
use iced::Element;
use iced::widget::{column, text};

#[derive(Debug, Clone)]
pub struct State {
    pub pet_scale: f32,
    pub position_x: String,
    pub position_y: String,
    pub always_on_top: bool,
    pub opacity: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Message {
    ScaleChanged(f32),
    PositionXChanged(String),
    PositionYChanged(String),
    ToggleAlwaysOnTop(bool),
    OpacityChanged(f32),
}

impl State {
    pub fn from_config(config: &AppConfig) -> Self {
        Self {
            pet_scale: config.appearance.pet_scale,
            position_x: config.appearance.pet_position.x.to_string(),
            position_y: config.appearance.pet_position.y.to_string(),
            always_on_top: config.appearance.always_on_top,
            opacity: config.appearance.opacity,
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::ScaleChanged(v) => self.pet_scale = v,
            Message::PositionXChanged(v) => self.position_x = v,
            Message::PositionYChanged(v) => self.position_y = v,
            Message::ToggleAlwaysOnTop(v) => self.always_on_top = v,
            Message::OpacityChanged(v) => self.opacity = v,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        column![text("Appearance Settings").size(24),]
            .spacing(12)
            .padding(20)
            .into()
    }

    pub fn apply_to(&self, config: &mut AppConfig) {
        config.appearance.pet_scale = self.pet_scale;
        config.appearance.pet_position.x = self.position_x.parse().unwrap_or(0);
        config.appearance.pet_position.y = self.position_y.parse().unwrap_or(0);
        config.appearance.always_on_top = self.always_on_top;
        config.appearance.opacity = self.opacity;
    }
}
