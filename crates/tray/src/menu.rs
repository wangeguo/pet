use tray_icon::menu::{Menu, MenuId, MenuItem, PredefinedMenuItem};

pub fn build_menu(pet_visible: bool) -> Menu {
    let menu = Menu::new();

    let show_label = if pet_visible { "Hide Pet" } else { "Show Pet" };

    let show_pet = MenuItem::with_id(MenuId::new("show_pet"), show_label, true, None);
    let separator = PredefinedMenuItem::separator();
    let open_settings = MenuItem::with_id(MenuId::new("open_settings"), "Settings", true, None);
    let open_manager = MenuItem::with_id(MenuId::new("open_manager"), "Open Manager", true, None);
    let separator2 = PredefinedMenuItem::separator();
    let quit = MenuItem::with_id(MenuId::new("quit"), "Quit", true, None);

    menu.append(&show_pet).unwrap();
    menu.append(&separator).unwrap();
    menu.append(&open_settings).unwrap();
    menu.append(&open_manager).unwrap();
    menu.append(&separator2).unwrap();
    menu.append(&quit).unwrap();

    menu
}
