use iced::widget::{button, checkbox, container, text_editor, text_input};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

// Win98 color palette
pub const WIN98_GRAY: Color = Color::from_rgb(0.753, 0.753, 0.753);
pub const WIN98_DARK_GRAY: Color = Color::from_rgb(0.502, 0.502, 0.502);
pub const WIN98_WHITE: Color = Color::WHITE;
pub const WIN98_BLACK: Color = Color::BLACK;
pub const WIN98_NAVY: Color = Color::from_rgb(0.0, 0.0, 0.502);

pub const MENU_BAR_HEIGHT: f32 = 22.0;
pub const STATUS_BAR_HEIGHT: f32 = 20.0;

pub fn menu_bar_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(WIN98_GRAY)),
        ..Default::default()
    }
}

pub fn menu_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Hovered | button::Status::Pressed => button::Style {
            background: Some(Background::Color(WIN98_NAVY)),
            text_color: WIN98_WHITE,
            border: Border::default(),
            ..Default::default()
        },
        _ => button::Style {
            background: Some(Background::Color(WIN98_GRAY)),
            text_color: WIN98_BLACK,
            border: Border::default(),
            ..Default::default()
        },
    }
}

pub fn menu_button_active_style(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(WIN98_NAVY)),
        text_color: WIN98_WHITE,
        border: Border::default(),
        ..Default::default()
    }
}

pub fn menu_item_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(WIN98_NAVY)),
            text_color: WIN98_WHITE,
            border: Border::default(),
            ..Default::default()
        },
        _ => button::Style {
            background: Some(Background::Color(WIN98_GRAY)),
            text_color: WIN98_BLACK,
            border: Border::default(),
            ..Default::default()
        },
    }
}

pub fn dropdown_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(WIN98_GRAY)),
        border: Border {
            color: WIN98_WHITE,
            width: 2.0,
            radius: 0.0.into(),
        },
        shadow: Shadow {
            color: WIN98_BLACK,
            offset: Vector::new(1.0, 1.0),
            blur_radius: 0.0,
        },
        ..Default::default()
    }
}

pub fn status_bar_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(WIN98_GRAY)),
        border: Border {
            color: WIN98_DARK_GRAY,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn editor_style(_theme: &Theme, status: text_editor::Status) -> text_editor::Style {
    let _ = status;
    text_editor::Style {
        background: Background::Color(WIN98_WHITE),
        border: Border {
            color: WIN98_DARK_GRAY,
            width: 0.0,
            radius: 0.0.into(),
        },
        placeholder: WIN98_DARK_GRAY,
        value: WIN98_BLACK,
        selection: Color::from_rgb(0.0, 0.0, 0.502),
    }
}

pub fn dialog_container_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(WIN98_GRAY)),
        border: Border {
            color: WIN98_WHITE,
            width: 2.0,
            radius: 0.0.into(),
        },
        shadow: Shadow {
            color: WIN98_BLACK,
            offset: Vector::new(1.0, 1.0),
            blur_radius: 0.0,
        },
        ..Default::default()
    }
}

pub fn win98_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(WIN98_GRAY)),
            text_color: WIN98_BLACK,
            border: Border {
                color: WIN98_DARK_GRAY,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        },
        _ => button::Style {
            background: Some(Background::Color(WIN98_GRAY)),
            text_color: WIN98_BLACK,
            border: Border {
                color: WIN98_WHITE,
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: Shadow {
                color: WIN98_DARK_GRAY,
                offset: Vector::new(1.0, 1.0),
                blur_radius: 0.0,
            },
            ..Default::default()
        },
    }
}

pub fn win98_text_input_style(_theme: &Theme, _status: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: Background::Color(WIN98_WHITE),
        border: Border {
            color: WIN98_DARK_GRAY,
            width: 1.0,
            radius: 0.0.into(),
        },
        icon: WIN98_BLACK,
        placeholder: WIN98_DARK_GRAY,
        value: WIN98_BLACK,
        selection: WIN98_NAVY,
    }
}

pub fn win98_checkbox_style(_theme: &Theme, _status: checkbox::Status) -> checkbox::Style {
    checkbox::Style {
        background: Background::Color(WIN98_WHITE),
        icon_color: WIN98_BLACK,
        border: Border {
            color: WIN98_DARK_GRAY,
            width: 1.0,
            radius: 0.0.into(),
        },
        text_color: Some(WIN98_BLACK),
    }
}

pub fn dialog_title_bar_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(WIN98_NAVY)),
        ..Default::default()
    }
}

pub fn dialog_title_bar_close_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Hovered | button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.6, 0.0, 0.0))),
            text_color: WIN98_WHITE,
            border: Border {
                color: WIN98_WHITE,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        },
        _ => button::Style {
            background: Some(Background::Color(WIN98_NAVY)),
            text_color: WIN98_WHITE,
            border: Border::default(),
            ..Default::default()
        },
    }
}

pub fn menu_item_disabled_style(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(WIN98_GRAY)),
        text_color: WIN98_DARK_GRAY,
        border: Border::default(),
        ..Default::default()
    }
}

pub fn win98_sunken_editor_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(WIN98_WHITE)),
        border: Border {
            color: WIN98_DARK_GRAY,
            width: 1.0,
            radius: 0.0.into(),
        },
        shadow: Shadow {
            color: WIN98_WHITE,
            offset: Vector::new(-1.0, -1.0),
            blur_radius: 0.0,
        },
        ..Default::default()
    }
}

pub fn win98_sunken_container_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(WIN98_WHITE)),
        border: Border {
            color: WIN98_DARK_GRAY,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn win98_list_item_style(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(WIN98_NAVY)),
            text_color: WIN98_WHITE,
            border: Border::default(),
            ..Default::default()
        },
        _ => button::Style {
            background: Some(Background::Color(WIN98_WHITE)),
            text_color: WIN98_BLACK,
            border: Border::default(),
            ..Default::default()
        },
    }
}

pub fn win98_list_item_selected_style(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(WIN98_NAVY)),
        text_color: WIN98_WHITE,
        border: Border::default(),
        ..Default::default()
    }
}

