use iced::widget::{button, checkbox, container, text_editor, text_input};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

pub const MENU_BAR_HEIGHT: f32 = 22.0;
pub const STATUS_BAR_HEIGHT: f32 = 20.0;

const WIN98_GRAY: Color = Color::from_rgb(0.753, 0.753, 0.753);
const WIN98_DARK_GRAY: Color = Color::from_rgb(0.502, 0.502, 0.502);
const WIN98_WHITE: Color = Color::WHITE;
const WIN98_BLACK: Color = Color::BLACK;
const WIN98_NAVY: Color = Color::from_rgb(0.0, 0.0, 0.502);

const DARK_SURFACE: Color = Color::from_rgb(0.148, 0.164, 0.188);
const DARK_PANEL: Color = Color::from_rgb(0.200, 0.219, 0.250);
const DARK_BORDER_LIGHT: Color = Color::from_rgb(0.380, 0.407, 0.454);
const DARK_BORDER_DARK: Color = Color::from_rgb(0.055, 0.063, 0.078);
const DARK_TEXT: Color = Color::from_rgb(0.925, 0.937, 0.956);
const DARK_TEXT_MUTED: Color = Color::from_rgb(0.600, 0.647, 0.714);
const DARK_ACCENT: Color = Color::from_rgb(0.231, 0.478, 0.894);
const DARK_ACCENT_STRONG: Color = Color::from_rgb(0.152, 0.392, 0.792);

#[derive(Clone, Copy)]
struct Palette {
    surface: Color,
    panel: Color,
    border_light: Color,
    border_dark: Color,
    text: Color,
    text_muted: Color,
    accent: Color,
    accent_strong: Color,
}

fn palette(theme: &Theme) -> Palette {
    if matches!(theme, Theme::Dark) {
        Palette {
            surface: DARK_SURFACE,
            panel: DARK_PANEL,
            border_light: DARK_BORDER_LIGHT,
            border_dark: DARK_BORDER_DARK,
            text: DARK_TEXT,
            text_muted: DARK_TEXT_MUTED,
            accent: DARK_ACCENT,
            accent_strong: DARK_ACCENT_STRONG,
        }
    } else {
        Palette {
            surface: WIN98_GRAY,
            panel: WIN98_WHITE,
            border_light: WIN98_WHITE,
            border_dark: WIN98_DARK_GRAY,
            text: WIN98_BLACK,
            text_muted: WIN98_DARK_GRAY,
            accent: WIN98_NAVY,
            accent_strong: WIN98_NAVY,
        }
    }
}

pub fn menu_bar_style(theme: &Theme) -> container::Style {
    let palette = palette(theme);

    container::Style {
        background: Some(Background::Color(palette.surface)),
        ..Default::default()
    }
}

pub fn menu_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = palette(theme);

    match status {
        button::Status::Hovered | button::Status::Pressed => button::Style {
            background: Some(Background::Color(palette.accent)),
            text_color: WIN98_WHITE,
            border: Border::default(),
            ..Default::default()
        },
        _ => button::Style {
            background: Some(Background::Color(palette.surface)),
            text_color: palette.text,
            border: Border::default(),
            ..Default::default()
        },
    }
}

pub fn menu_button_active_style(theme: &Theme, _status: button::Status) -> button::Style {
    let palette = palette(theme);

    button::Style {
        background: Some(Background::Color(palette.accent)),
        text_color: WIN98_WHITE,
        border: Border::default(),
        ..Default::default()
    }
}

pub fn menu_item_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = palette(theme);

    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(palette.accent)),
            text_color: WIN98_WHITE,
            border: Border::default(),
            ..Default::default()
        },
        _ => button::Style {
            background: Some(Background::Color(palette.surface)),
            text_color: palette.text,
            border: Border::default(),
            ..Default::default()
        },
    }
}

pub fn dropdown_style(theme: &Theme) -> container::Style {
    let palette = palette(theme);

    container::Style {
        background: Some(Background::Color(palette.surface)),
        border: Border {
            color: palette.border_light,
            width: 2.0,
            radius: 0.0.into(),
        },
        shadow: Shadow {
            color: palette.border_dark,
            offset: Vector::new(1.0, 1.0),
            blur_radius: 0.0,
        },
        ..Default::default()
    }
}

pub fn status_bar_style(theme: &Theme) -> container::Style {
    let palette = palette(theme);

    container::Style {
        background: Some(Background::Color(palette.surface)),
        border: Border {
            color: palette.border_dark,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn editor_style(theme: &Theme, status: text_editor::Status) -> text_editor::Style {
    let _ = status;
    let palette = palette(theme);

    text_editor::Style {
        background: Background::Color(palette.panel),
        border: Border {
            color: palette.border_dark,
            width: 0.0,
            radius: 0.0.into(),
        },
        placeholder: palette.text_muted,
        value: palette.text,
        selection: palette.accent_strong,
    }
}

pub fn dialog_container_style(theme: &Theme) -> container::Style {
    let palette = palette(theme);

    container::Style {
        background: Some(Background::Color(palette.surface)),
        border: Border {
            color: palette.border_light,
            width: 2.0,
            radius: 0.0.into(),
        },
        shadow: Shadow {
            color: palette.border_dark,
            offset: Vector::new(1.0, 1.0),
            blur_radius: 0.0,
        },
        ..Default::default()
    }
}

pub fn win98_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = palette(theme);

    match status {
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(palette.surface)),
            text_color: palette.text,
            border: Border {
                color: palette.border_dark,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        },
        _ => button::Style {
            background: Some(Background::Color(palette.surface)),
            text_color: palette.text,
            border: Border {
                color: palette.border_light,
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: Shadow {
                color: palette.border_dark,
                offset: Vector::new(1.0, 1.0),
                blur_radius: 0.0,
            },
            ..Default::default()
        },
    }
}

pub fn win98_text_input_style(theme: &Theme, _status: text_input::Status) -> text_input::Style {
    let palette = palette(theme);

    text_input::Style {
        background: Background::Color(palette.panel),
        border: Border {
            color: palette.border_dark,
            width: 1.0,
            radius: 0.0.into(),
        },
        icon: palette.text,
        placeholder: palette.text_muted,
        value: palette.text,
        selection: palette.accent,
    }
}

pub fn win98_checkbox_style(theme: &Theme, _status: checkbox::Status) -> checkbox::Style {
    let palette = palette(theme);

    checkbox::Style {
        background: Background::Color(palette.panel),
        icon_color: palette.text,
        border: Border {
            color: palette.border_dark,
            width: 1.0,
            radius: 0.0.into(),
        },
        text_color: Some(palette.text),
    }
}

pub fn dialog_title_bar_style(theme: &Theme) -> container::Style {
    let palette = palette(theme);

    container::Style {
        background: Some(Background::Color(palette.accent)),
        ..Default::default()
    }
}

pub fn dialog_title_bar_close_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = palette(theme);

    match status {
        button::Status::Hovered | button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.6, 0.0, 0.0))),
            text_color: WIN98_WHITE,
            border: Border {
                color: palette.border_light,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        },
        _ => button::Style {
            background: Some(Background::Color(palette.accent)),
            text_color: WIN98_WHITE,
            border: Border::default(),
            ..Default::default()
        },
    }
}

pub fn menu_item_disabled_style(theme: &Theme, _status: button::Status) -> button::Style {
    let palette = palette(theme);

    button::Style {
        background: Some(Background::Color(palette.surface)),
        text_color: palette.text_muted,
        border: Border::default(),
        ..Default::default()
    }
}

pub fn win98_sunken_editor_style(theme: &Theme) -> container::Style {
    let palette = palette(theme);

    container::Style {
        background: Some(Background::Color(palette.panel)),
        border: Border {
            color: palette.border_dark,
            width: 1.0,
            radius: 0.0.into(),
        },
        shadow: Shadow {
            color: palette.border_light,
            offset: Vector::new(-1.0, -1.0),
            blur_radius: 0.0,
        },
        ..Default::default()
    }
}

pub fn win98_sunken_container_style(theme: &Theme) -> container::Style {
    let palette = palette(theme);

    container::Style {
        background: Some(Background::Color(palette.panel)),
        border: Border {
            color: palette.border_dark,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn win98_list_item_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = palette(theme);

    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(palette.accent)),
            text_color: WIN98_WHITE,
            border: Border::default(),
            ..Default::default()
        },
        _ => button::Style {
            background: Some(Background::Color(palette.panel)),
            text_color: palette.text,
            border: Border::default(),
            ..Default::default()
        },
    }
}

pub fn win98_list_item_selected_style(theme: &Theme, _status: button::Status) -> button::Style {
    let palette = palette(theme);

    button::Style {
        background: Some(Background::Color(palette.accent)),
        text_color: WIN98_WHITE,
        border: Border::default(),
        ..Default::default()
    }
}
