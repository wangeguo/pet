use common::config::AppConfig;
use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Element, Length};
use uuid::Uuid;

use crate::app::Message;

pub fn view<'a>(config: &'a AppConfig, delete_confirmation: Option<Uuid>) -> Element<'a, Message> {
    let header = row![
        text("Pet Manager").size(24),
        Space::new().width(Length::Fill),
        button("+ Create").on_press(Message::NavigateToCreate),
    ]
    .spacing(10);

    if config.pets.is_empty() {
        return column![
            header,
            Space::new().height(Length::Fixed(40.0)),
            text("No pets yet. Click \"+ Create\" to get started!")
                .width(Length::Fill)
                .center(),
        ]
        .spacing(15)
        .into();
    }

    let mut list = column![].spacing(10);

    for pet in &config.pets {
        let is_active = config.active_pet == Some(pet.id);

        let status = if is_active { "[Active] " } else { "" };

        let info = column![
            text(format!("{}{}", status, pet.name)).size(16),
            text(&pet.description).size(12),
            text(format!("Created: {}", pet.created_at)).size(10),
        ]
        .spacing(4);

        let mut actions = row![].spacing(8);

        if !is_active {
            actions = actions.push(
                button("Set Active")
                    .on_press(Message::SwitchPet(pet.id))
                    .padding(4),
            );
        }

        if delete_confirmation == Some(pet.id) {
            actions = actions.push(
                button("Confirm Delete")
                    .on_press(Message::ConfirmDelete(pet.id))
                    .padding(4),
            );
            actions = actions.push(button("Cancel").on_press(Message::CancelDelete).padding(4));
        } else {
            actions = actions.push(
                button("Delete")
                    .on_press(Message::DeletePet(pet.id))
                    .padding(4),
            );
        }

        let card = container(
            row![info, Space::new().width(Length::Fill), actions]
                .spacing(10)
                .align_y(iced::Alignment::Center),
        )
        .padding(12)
        .width(Length::Fill)
        .style(container::rounded_box);

        list = list.push(card);
    }

    column![header, scrollable(list).height(Length::Fill)]
        .spacing(15)
        .into()
}
