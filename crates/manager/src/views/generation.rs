use iced::widget::{button, column, container, progress_bar, row, text};
use iced::{Element, Length};

use crate::app::{GenerationStatus, Message};

pub fn view<'a>(
    pet_name: &'a str,
    status: &GenerationStatus,
    progress: u32,
    error: Option<&'a str>,
    close_confirmation: bool,
) -> Element<'a, Message> {
    let title = text(format!("Generating: {pet_name}")).size(24);

    let status_text = match status {
        GenerationStatus::Submitting => "Submitting to Meshy AI...".to_string(),
        GenerationStatus::Pending => format!("Pending... {progress}%"),
        GenerationStatus::InProgress => format!("Generating... {progress}%"),
        GenerationStatus::Downloading => "Downloading model...".to_string(),
        GenerationStatus::Succeeded => "Generation complete!".to_string(),
        GenerationStatus::Failed => "Generation failed.".to_string(),
    };

    let mut content = column![title, text(status_text).size(16)].spacing(12);

    if matches!(
        status,
        GenerationStatus::Pending | GenerationStatus::InProgress
    ) {
        content = content.push(progress_bar(0.0..=100.0, progress as f32));
    }

    if let Some(err) = error {
        content = content.push(text(err).size(12));
    }

    match status {
        GenerationStatus::Failed => {
            content = content.push(
                row![
                    button("Retry")
                        .on_press(Message::RetryGeneration)
                        .padding(8),
                    button("Back")
                        .on_press(Message::NavigateTo(crate::views::View::PetList))
                        .padding(8),
                ]
                .spacing(8),
            );
        }
        GenerationStatus::Succeeded => {
            content = content.push(
                row![
                    button("Save & Set Active")
                        .on_press(Message::SavePet { set_active: true })
                        .padding(8),
                    button("Save Only")
                        .on_press(Message::SavePet { set_active: false })
                        .padding(8),
                ]
                .spacing(8),
            );
        }
        _ => {}
    }

    if close_confirmation {
        content = content.push(
            container(
                column![
                    text("Generation is in progress. Close anyway?").size(14),
                    row![
                        button("Force Close")
                            .on_press(Message::ForceClose)
                            .padding(8),
                        button("Cancel").on_press(Message::CancelClose).padding(8),
                    ]
                    .spacing(8),
                ]
                .spacing(8),
            )
            .padding(12)
            .style(container::rounded_box),
        );
    }

    container(content).padding(16).width(Length::Fill).into()
}
