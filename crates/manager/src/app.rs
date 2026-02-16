use common::config::AppConfig;
use common::paths::AppPaths;
use common::storage::StorageService;
use iced::widget::{Space, button, column, container, row, text};
use iced::{Element, Length, Theme};
use tracing::info;

use crate::views::View;

pub struct PetManager {
    #[expect(dead_code)]
    pub paths: AppPaths,
    pub config: AppConfig,
    #[expect(dead_code)]
    pub storage: StorageService,
    pub current_view: View,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    NavigateTo(View),
    DismissError,
}

impl PetManager {
    pub fn new() -> (Self, iced::Task<Message>) {
        let paths = AppPaths::new().expect("Failed to resolve paths");
        let config = AppConfig::load(&paths).unwrap_or_default();
        let storage = StorageService::new(paths.clone());

        info!("Manager initialized, {} pets loaded", config.pets.len());

        let manager = Self {
            paths,
            config,
            storage,
            current_view: View::PetList,
            error_message: None,
        };

        (manager, iced::Task::none())
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::NavigateTo(view) => {
                self.current_view = view;
            }
            Message::DismissError => {
                self.error_message = None;
            }
        }
        iced::Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let content = match self.current_view {
            View::PetList => self.view_pet_list(),
            View::CreatePet => self.view_create_pet(),
        };

        let mut page = column![content].spacing(10);

        if let Some(ref error) = self.error_message {
            page = page.push(
                row![
                    text(error.as_str()),
                    button("Dismiss").on_press(Message::DismissError),
                ]
                .spacing(10),
            );
        }

        container(page.padding(20))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn theme(&self) -> Theme {
        Theme::Light
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    fn view_pet_list(&self) -> Element<'_, Message> {
        let header = row![
            text("Pet Manager").size(24),
            Space::new().width(Length::Fill),
            button("+ Create").on_press(Message::NavigateTo(View::CreatePet)),
        ]
        .spacing(10);

        let pet_count = text(format!("{} pets", self.config.pets.len()));

        column![header, pet_count].spacing(15).into()
    }

    fn view_create_pet(&self) -> Element<'_, Message> {
        column![
            button("< Back").on_press(Message::NavigateTo(View::PetList)),
            text("Create New Pet").size(24),
            text("Coming soon..."),
        ]
        .spacing(15)
        .into()
    }
}
