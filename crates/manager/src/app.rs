use std::time::Duration;

use common::config::AppConfig;
use common::models::Pet;
use common::paths::AppPaths;
use common::storage::StorageService;
use iced::widget::{button, column, container, row, text};
use iced::{Element, Length, Subscription, Theme};
use tracing::{error, info};
use uuid::Uuid;

use crate::meshy::{MeshyClient, TaskStatusResponse};
use crate::views::{self, View};

pub struct PetManager {
    pub paths: AppPaths,
    pub config: AppConfig,
    pub storage: StorageService,
    pub current_view: View,
    pub pet_name: String,
    pub pet_description: String,
    pub generation: Option<GenerationState>,
    pub delete_confirmation: Option<Uuid>,
    pub close_confirmation: Option<iced::window::Id>,
    pub error_message: Option<String>,
}

pub struct GenerationState {
    pub status: GenerationStatus,
    pub task_id: Option<String>,
    pub progress: u32,
    pub model_data: Option<Vec<u8>>,
    pub thumbnail_data: Option<Vec<u8>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerationStatus {
    Submitting,
    Pending,
    InProgress,
    Downloading,
    Succeeded,
    Failed,
}

impl GenerationState {
    fn is_active(&self) -> bool {
        matches!(
            self.status,
            GenerationStatus::Submitting
                | GenerationStatus::Pending
                | GenerationStatus::InProgress
                | GenerationStatus::Downloading
        )
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    NavigateToCreate,
    NavigateTo(View),
    UpdatePetName(String),
    UpdatePetDescription(String),
    StartGeneration,
    TaskCreated(Result<String, String>),
    PollStatus,
    StatusReceived(Result<TaskStatusResponse, String>),
    DownloadComplete(Result<(Vec<u8>, Option<Vec<u8>>), String>),
    SavePet { set_active: bool },
    RetryGeneration,
    CloseRequested(iced::window::Id),
    ForceClose,
    CancelClose,
    PreviewPet(Uuid),
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
            pet_name: String::new(),
            pet_description: String::new(),
            generation: None,
            delete_confirmation: None,
            close_confirmation: None,
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
                self.pet_name.clear();
                self.pet_description.clear();
                self.current_view = View::CreatePet;
            }
            Message::UpdatePetName(name) => {
                self.pet_name = name;
            }
            Message::UpdatePetDescription(desc) => {
                self.pet_description = desc;
            }
            Message::StartGeneration => {
                let Some(api_key) = self.config.meshy_api_key.clone() else {
                    self.error_message = Some("Meshy API key not configured".into());
                    return iced::Task::none();
                };
                let description = self.pet_description.clone();

                info!("Starting generation: name={}", self.pet_name);
                self.generation = Some(GenerationState {
                    status: GenerationStatus::Submitting,
                    task_id: None,
                    progress: 0,
                    model_data: None,
                    thumbnail_data: None,
                    error: None,
                });
                self.current_view = View::Generation;

                return iced::Task::perform(
                    async move {
                        let client = MeshyClient::new(api_key);
                        client
                            .create_task(&description)
                            .await
                            .map_err(|e| e.to_string())
                    },
                    Message::TaskCreated,
                );
            }
            Message::TaskCreated(result) => {
                if let Some(ref mut gs) = self.generation {
                    match result {
                        Ok(task_id) => {
                            info!("Meshy task created: {task_id}");
                            gs.task_id = Some(task_id);
                            gs.status = GenerationStatus::Pending;
                        }
                        Err(e) => {
                            error!("Failed to create task: {e}");
                            gs.status = GenerationStatus::Failed;
                            gs.error = Some(e);
                        }
                    }
                }
            }
            Message::PollStatus => {
                if let Some(ref gs) = self.generation
                    && let Some(ref task_id) = gs.task_id
                {
                    let api_key = self.config.meshy_api_key.clone().unwrap();
                    let task_id = task_id.clone();
                    return iced::Task::perform(
                        async move {
                            let client = MeshyClient::new(api_key);
                            client
                                .get_task_status(&task_id)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        Message::StatusReceived,
                    );
                }
            }
            Message::StatusReceived(result) => {
                if let Some(ref mut gs) = self.generation {
                    match result {
                        Ok(status) => {
                            gs.progress = status.progress.unwrap_or(0);

                            match status.status.as_str() {
                                "PENDING" => gs.status = GenerationStatus::Pending,
                                "IN_PROGRESS" => gs.status = GenerationStatus::InProgress,
                                "SUCCEEDED" => {
                                    info!("Generation succeeded, downloading assets");
                                    gs.status = GenerationStatus::Downloading;
                                    return self.start_download(&status);
                                }
                                "FAILED" | "EXPIRED" => {
                                    gs.status = GenerationStatus::Failed;
                                    gs.error = Some(format!("Task {}", status.status));
                                }
                                other => {
                                    info!("Unknown status: {other}");
                                }
                            }
                        }
                        Err(e) => {
                            error!("Poll failed: {e}");
                            gs.status = GenerationStatus::Failed;
                            gs.error = Some(e);
                        }
                    }
                }
            }
            Message::DownloadComplete(result) => {
                if let Some(ref mut gs) = self.generation {
                    match result {
                        Ok((model, thumbnail)) => {
                            info!("Download complete");
                            gs.model_data = Some(model);
                            gs.thumbnail_data = thumbnail;
                            gs.status = GenerationStatus::Succeeded;
                        }
                        Err(e) => {
                            error!("Download failed: {e}");
                            gs.status = GenerationStatus::Failed;
                            gs.error = Some(e);
                        }
                    }
                }
            }
            Message::SavePet { set_active } => {
                if let Some(ref gs) = self.generation
                    && let Some(ref model_data) = gs.model_data
                {
                    let pet_id = Uuid::new_v4();
                    match self.storage.save_model(&pet_id, model_data) {
                        Ok(model_path) => {
                            let mut pet = Pet::new(
                                self.pet_name.clone(),
                                self.pet_description.clone(),
                                model_path,
                            );
                            pet.id = pet_id;

                            if let Some(ref thumb) = gs.thumbnail_data {
                                match self.storage.save_thumbnail(&pet_id, thumb) {
                                    Ok(path) => pet.thumbnail_path = Some(path),
                                    Err(e) => error!("Failed to save thumbnail: {e}"),
                                }
                            }

                            self.config.add_pet(pet);
                            if set_active {
                                self.config.set_active_pet(pet_id);
                            }
                            if let Err(e) = self.config.save(&self.paths) {
                                error!("Failed to save config: {e}");
                                self.error_message = Some(format!("Failed to save: {e}"));
                            } else {
                                info!("Pet saved: {pet_id}");
                            }
                        }
                        Err(e) => {
                            error!("Failed to save model: {e}");
                            self.error_message = Some(format!("Failed to save model: {e}"));
                        }
                    }

                    self.generation = None;
                    self.current_view = View::PetList;
                }
            }
            Message::RetryGeneration => {
                self.generation = None;
                self.current_view = View::CreatePet;
            }
            Message::CloseRequested(id) => {
                if self.generation.as_ref().is_some_and(|g| g.is_active()) {
                    self.close_confirmation = Some(id);
                } else {
                    return iced::window::close(id);
                }
            }
            Message::ForceClose => {
                if let Some(id) = self.close_confirmation.take() {
                    return iced::window::close(id);
                }
            }
            Message::CancelClose => {
                self.close_confirmation = None;
            }
            Message::PreviewPet(id) => {
                self.config.set_active_pet(id);
                if let Err(e) = self.config.save(&self.paths) {
                    error!("Failed to save config: {e}");
                    self.error_message = Some(format!("Failed to save: {e}"));
                } else {
                    info!("Preview pet {id}: set active and saved config");
                }
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
            View::CreatePet => views::create_pet::view(
                &self.pet_name,
                &self.pet_description,
                self.config.meshy_api_key.is_some(),
            ),
            View::Generation => {
                let gs = self.generation.as_ref();
                views::generation::view(
                    &self.pet_name,
                    gs.map_or(&GenerationStatus::Submitting, |g| &g.status),
                    gs.map_or(0, |g| g.progress),
                    gs.and_then(|g| g.error.as_deref()),
                    self.close_confirmation.is_some(),
                )
            }
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

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![];

        if let Some(ref gs) = self.generation {
            let interval = match gs.status {
                GenerationStatus::Pending => Some(Duration::from_secs(5)),
                GenerationStatus::InProgress => Some(Duration::from_secs(3)),
                _ => None,
            };
            if let Some(interval) = interval {
                subs.push(iced::time::every(interval).map(|_| Message::PollStatus));
            }
        }

        subs.push(iced::window::close_requests().map(Message::CloseRequested));

        Subscription::batch(subs)
    }

    fn start_download(&self, status: &TaskStatusResponse) -> iced::Task<Message> {
        let api_key = self.config.meshy_api_key.clone().unwrap();
        let model_url = status
            .model_urls
            .as_ref()
            .and_then(|u| u.glb.clone())
            .unwrap_or_default();
        let thumbnail_url = status.thumbnail_url.clone();

        iced::Task::perform(
            async move {
                let client = MeshyClient::new(api_key);
                let model = client
                    .download_bytes(&model_url)
                    .await
                    .map_err(|e| e.to_string())?;
                let thumbnail = if let Some(url) = thumbnail_url {
                    Some(
                        client
                            .download_bytes(&url)
                            .await
                            .map_err(|e| e.to_string())?,
                    )
                } else {
                    None
                };
                Ok((model, thumbnail))
            },
            Message::DownloadComplete,
        )
    }
}
