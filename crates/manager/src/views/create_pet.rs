use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length};

use crate::app::Message;

pub fn view<'a>(
    pet_name: &'a str,
    pet_description: &'a str,
    has_api_key: bool,
) -> Element<'a, Message> {
    let header = row![
        button("< Back").on_press(Message::NavigateTo(crate::views::View::PetList)),
        text("Create New Pet").size(24),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let name_input = column![
        text("Name").size(14),
        text_input("Enter pet name", pet_name).on_input(Message::UpdatePetName),
    ]
    .spacing(4);

    let description_input = column![
        text("Description (used as AI generation prompt)").size(14),
        text_input("Describe your pet's appearance", pet_description)
            .on_input(Message::UpdatePetDescription),
    ]
    .spacing(4);

    let api_status = if has_api_key {
        text("Meshy API Key: configured").size(12)
    } else {
        text("Meshy API Key: not configured (set meshy_api_key in config.toml)").size(12)
    };

    let can_generate = !pet_name.is_empty() && !pet_description.is_empty() && has_api_key;

    let mut generate_btn = button("Generate 3D Model").padding(8);
    if can_generate {
        generate_btn = generate_btn.on_press(Message::StartGeneration);
    }

    let form =
        container(column![name_input, description_input, api_status, generate_btn].spacing(16))
            .padding(16)
            .width(Length::Fill)
            .style(container::rounded_box);

    column![header, form].spacing(15).into()
}
