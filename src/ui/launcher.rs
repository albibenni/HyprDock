use gtk4::prelude::*;
use gtk4::{Button, Popover, Box, SearchEntry, ScrolledWindow, ListBox, Orientation, ListBoxRow, Image, Label};
use crate::ui::constants::*;
use crate::ui::utils::create_base_button;
use crate::launcher::{self, AppItem};

pub fn create_launcher() -> (Button, Popover) {
    let button = create_base_button(Some(LAUNCHER_ICON_SIZE), Some("open-menu-symbolic"));
    button.add_css_class(CLASS_LAUNCHER_BUTTON);

    let popover = Popover::builder().position(gtk4::PositionType::Top).autohide(true).build();
    popover.add_css_class(CLASS_LAUNCHER_POPOVER);
    popover.set_parent(&button);

    let launcher_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .width_request(300)
        .height_request(400)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .build();

    let search_entry = SearchEntry::builder().placeholder_text("Search applications...").build();
    launcher_box.append(&search_entry);

    let scrolled = ScrolledWindow::builder().propagate_natural_height(true).build();
    launcher_box.append(&scrolled);

    let list_box = ListBox::builder().selection_mode(gtk4::SelectionMode::None).build();
    list_box.add_css_class(CLASS_LAUNCHER_LIST);
    scrolled.set_child(Some(&list_box));

    let lb_clone = list_box.clone();
    let p_clone = popover.clone();
    search_entry.connect_search_changed(move |entry| {
        let query = entry.text().to_lowercase();
        let apps: Vec<AppItem> = launcher::get_all_apps().into_iter()
            .filter(|a| a.name.to_lowercase().contains(&query)).collect();
        update_launcher_list(&lb_clone, &apps, &p_clone);
    });

    let lb_init = list_box.clone();
    let p_init = popover.clone();
    popover.connect_show(move |_| {
        update_launcher_list(&lb_init, &launcher::get_all_apps(), &p_init);
    });

    popover.set_child(Some(&launcher_box));
    
    let p_click = popover.clone();
    button.connect_clicked(move |_| p_click.popup());

    (button, popover)
}

fn update_launcher_list(list_box: &ListBox, apps: &[AppItem], popover: &Popover) {
    while let Some(child) = list_box.first_child() { list_box.remove(&child); }
    for app in apps {
        let row = ListBoxRow::new();
        row.add_css_class(CLASS_LAUNCHER_ITEM);
        let item_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .margin_top(6)
            .margin_bottom(6)
            .margin_start(6)
            .margin_end(6)
            .build();
        
        let icon = Image::builder().pixel_size(LAUNCHER_ICON_SIZE).build();
        if let Some(gicon) = &app.icon { icon.set_from_gicon(gicon); }
        else { icon.set_icon_name(Some(FALLBACK_ICON_NAME)); }

        let label = Label::new(Some(&app.name));
        label.set_halign(gtk4::Align::Start);

        item_box.append(&icon);
        item_box.append(&label);
        row.set_child(Some(&item_box));

        let app_name = app.name.clone();
        let p_clone = popover.clone();
        row.connect_activate(move |_| {
            launcher::launch_app(&app_name);
            p_clone.popdown();
        });
        list_box.append(&row);
    }
}
