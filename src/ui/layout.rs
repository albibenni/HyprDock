use gtk4::prelude::*;
use gtk4::{Box, Orientation, Separator, Widget};
use crate::ui::constants::*;

pub fn create_section(class: &str, spacing: i32, children: &[&impl IsA<Widget>]) -> Box {
    let section = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(spacing)
        .build();
    section.add_css_class(class);
    for child in children {
        section.append(*child);
    }
    section
}

pub fn create_separator() -> Separator {
    let sep = Separator::new(Orientation::Vertical);
    sep.add_css_class(CLASS_DOCK_SEPARATOR);
    sep
}
