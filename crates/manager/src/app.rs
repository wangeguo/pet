use common::config::AppConfig;
use common::paths::AppPaths;
use common::storage::StorageService;
use iced::widget::{button, column, container, row, text};
use iced::{Element, Length, Theme};
use tracing::{error, info};
use uuid::Uuid;

use crate::views::{self, View};

pub struct PetManager {
    pub paths: AppPaths,
    pub config: AppConfig,
    pub storage: StorageService,
    pub current_view: View,
    pub delete_confirmation: Option<Uuid>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    NavigateToCreate,
    NavigateTo(View),
    SwitchPet(Uuid),
    DeletePet(Uuid),
    ConfirmDelete(Uuid),
    CancelDelete,
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
            delete_confirmation: None,
            error_message: None,
        };

        (manager, iced::Task::none())
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::NavigateTo(view) => {
                self.current_view = view;
            }
            Message::NavigateToCreate => {
                self.current_view = View::CreatePet;
            }
            Message::SwitchPet(id) => {
                self.config.set_active_pet(id);
                if let Err(e) = self.config.save(&self.paths) {
                    error!("Failed to save config: {e}");
                    self.error_message = Some(format!("Failed to save: {e}"));
                } else {
                    info!("Switched active pet to {id}");
                }
            }
            Message::DeletePet(id) => {
                self.delete_confirmation = Some(id);
            }
            Message::ConfirmDelete(id) => {
                if let Some(pet) = self.config.get_pet(id).cloned()
                    && let Err(e) = self.storage.delete_pet_data(&pet)
                {
                    error!("Failed to delete pet data: {e}");
                    self.error_message = Some(format!("Failed to delete: {e}"));
                }
                self.config.remove_pet(id);
                if let Err(e) = self.config.save(&self.paths) {
                    error!("Failed to save config: {e}");
                    self.error_message = Some(format!("Failed to save: {e}"));
                } else {
                    info!("Deleted pet {id}");
                }
                self.delete_confirmation = None;
            }
            Message::CancelDelete => {
                self.delete_confirmation = None;
            }
            Message::DismissError => {
                self.error_message = None;
            }
        }
        iced::Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let content = match self.current_view {
            View::PetList => views::pet_list::view(&self.config, self.delete_confirmation),
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
