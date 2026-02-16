use iced::Element;
use iced::widget::{column, text};

pub fn view<'a, M: 'a>() -> Element<'a, M> {
    let version = env!("CARGO_PKG_VERSION");

    column![
        text("About Pet").size(24),
        text(format!("Version: {version}")),
        text("A desktop pet companion powered by AI"),
        text("License: Apache-2.0"),
    ]
    .spacing(12)
    .padding(20)
    .into()
}
