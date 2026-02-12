use tray_icon::menu::{CheckMenuItem, Menu, MenuId, MenuItem, PredefinedMenuItem};

pub fn build_menu(pet_visible: bool, auto_start: bool) -> Menu {
    let menu = Menu::new();

    let show_label = if pet_visible { "Hide Pet" } else { "Show Pet" };

    let show_pet = MenuItem::with_id(MenuId::new("show_pet"), show_label, true, None);
    let auto_start_item = CheckMenuItem::with_id(
        MenuId::new("auto_start"),
        "Launch at Login",
        true,
        auto_start,
        None,
    );
    let separator = PredefinedMenuItem::separator();
    let open_manager = MenuItem::with_id(MenuId::new("open_manager"), "Open Manager", true, None);
    let separator2 = PredefinedMenuItem::separator();
    let quit = MenuItem::with_id(MenuId::new("quit"), "Quit", true, None);

    menu.append(&show_pet).unwrap();
    menu.append(&auto_start_item).unwrap();
    menu.append(&separator).unwrap();
    menu.append(&open_manager).unwrap();
    menu.append(&separator2).unwrap();
    menu.append(&quit).unwrap();

    menu
}
