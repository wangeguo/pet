use iced::Element;
use iced::widget::{column, row, slider, text, text_input, toggler};

use common::config::AppConfig;

#[derive(Debug, Clone)]
pub struct State {
    pub pet_scale: f32,
    pub position_x: String,
    pub position_y: String,
    pub always_on_top: bool,
    pub opacity: f32,
}

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
        column![
            text("Appearance Settings").size(24),
            row![
                text("Pet Scale"),
                slider(0.5..=3.0, self.pet_scale, Message::ScaleChanged).step(0.1),
                text(format!("{:.1}", self.pet_scale)),
            ]
            .spacing(10),
            row![
                text("Position X"),
                text_input("0", &self.position_x).on_input(Message::PositionXChanged),
            ]
            .spacing(10),
            row![
                text("Position Y"),
                text_input("0", &self.position_y).on_input(Message::PositionYChanged),
            ]
            .spacing(10),
            toggler(self.always_on_top)
                .label("Always on Top")
                .on_toggle(Message::ToggleAlwaysOnTop),
            row![
                text("Opacity"),
                slider(0.1..=1.0, self.opacity, Message::OpacityChanged).step(0.05),
                text(format!("{:.0}%", self.opacity * 100.0)),
            ]
            .spacing(10),
        ]
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
