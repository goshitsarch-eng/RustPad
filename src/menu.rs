use iced::widget::{button, column, container, mouse_area, opaque, row, rule, text};
use iced::{Element, Fill, Length, Padding};

use crate::message::{MenuId, Message};
use crate::theme;

pub struct MenuState {
    pub has_undo: bool,
    pub has_selection: bool,
    pub word_wrap: bool,
}

pub fn view_menu_bar(active_menu: Option<MenuId>) -> Element<'static, Message> {
    let file_btn = menu_top_button("File", MenuId::File, active_menu);
    let edit_btn = menu_top_button("Edit", MenuId::Edit, active_menu);
    let format_btn = menu_top_button("Format", MenuId::Format, active_menu);
    let search_btn = menu_top_button("Search", MenuId::Search, active_menu);
    let help_btn = menu_top_button("Help", MenuId::Help, active_menu);

    container(
        row![file_btn, edit_btn, format_btn, search_btn, help_btn]
            .spacing(0)
            .height(theme::MENU_BAR_HEIGHT),
    )
    .style(theme::menu_bar_style)
    .width(Fill)
    .height(theme::MENU_BAR_HEIGHT)
    .into()
}

fn menu_top_button(
    label: &'static str,
    id: MenuId,
    active: Option<MenuId>,
) -> Element<'static, Message> {
    let is_active = active == Some(id);
    let style = if is_active {
        theme::menu_button_active_style
    } else {
        theme::menu_button_style
    };

    let btn = button(text(label).size(13))
        .on_press(Message::MenuClicked(id))
        .style(style)
        .padding(Padding::from([2, 8]));

    if active.is_some() && !is_active {
        // When a menu is open, hovering another button should switch to it
        mouse_area(btn)
            .on_enter(Message::MenuClicked(id))
            .into()
    } else {
        btn.into()
    }
}

pub fn view_dropdown<'a>(menu_id: MenuId, state: &MenuState) -> Element<'a, Message> {
    let items: Vec<Element<'a, Message>> = match menu_id {
        MenuId::File => vec![
            menu_item("New", Some("Ctrl+N"), Message::NewFile),
            menu_item("Open...", Some("Ctrl+O"), Message::OpenFile),
            menu_item("Save", Some("Ctrl+S"), Message::SaveFile),
            menu_item("Save As...", None, Message::SaveFileAs),
            separator(),
            menu_item("Page Setup...", None, Message::PageSetup),
            menu_item("Print...", Some("Ctrl+P"), Message::Print),
            separator(),
            menu_item("Exit", None, Message::Exit),
        ],
        MenuId::Edit => {
            let undo_label = if state.has_undo { "Undo" } else { "Can't Undo" };
            let undo_msg = if state.has_undo { Some(Message::Undo) } else { None };
            let sel_cut = if state.has_selection { Some(Message::Cut) } else { None };
            let sel_copy = if state.has_selection { Some(Message::Copy) } else { None };
            let sel_delete = if state.has_selection { Some(Message::Delete) } else { None };

            vec![
                menu_item_maybe(undo_label, Some("Ctrl+Z"), undo_msg),
                separator(),
                menu_item_maybe("Cut", Some("Ctrl+X"), sel_cut),
                menu_item_maybe("Copy", Some("Ctrl+C"), sel_copy),
                menu_item("Paste", Some("Ctrl+V"), Message::Paste),
                menu_item_maybe("Delete", Some("Del"), sel_delete),
                separator(),
                menu_item("Select All", Some("Ctrl+A"), Message::SelectAll),
                menu_item("Time/Date", Some("F5"), Message::InsertTimeDate),
            ]
        }
        MenuId::Format => vec![
            menu_item_checked("Word Wrap", None, Message::ToggleWordWrap, state.word_wrap),
            menu_item("Font...", None, Message::OpenFontDialog),
        ],
        MenuId::Search => {
            let goto_msg = if !state.word_wrap { Some(Message::OpenGoToDialog) } else { None };
            vec![
                menu_item("Find...", Some("Ctrl+F"), Message::OpenFindDialog),
                menu_item("Find Next", Some("F3"), Message::FindNext),
                menu_item("Replace...", Some("Ctrl+H"), Message::OpenReplaceDialog),
                separator(),
                menu_item_maybe("Go To...", Some("Ctrl+G"), goto_msg),
            ]
        }
        MenuId::Help => vec![
            menu_item("About RustPad", None, Message::ShowAbout),
        ],
    };

    let dropdown = container(column(items).spacing(0))
        .style(theme::dropdown_style)
        .padding(2)
        .width(Length::Fixed(200.0));

    opaque(dropdown).into()
}

fn menu_item(
    label: &'static str,
    shortcut: Option<&'static str>,
    msg: Message,
) -> Element<'static, Message> {
    let content = if let Some(sc) = shortcut {
        row![
            text(label).size(13).width(Length::Fill),
            text(sc).size(12),
        ]
        .spacing(12)
        .width(160)
    } else {
        row![text(label).size(13).width(Length::Fill),].width(160)
    };

    button(content)
        .on_press(msg)
        .style(theme::menu_item_style)
        .padding(Padding::from([3, 16]))
        .into()
}

fn menu_item_maybe<'a>(
    label: &'a str,
    shortcut: Option<&'a str>,
    msg: Option<Message>,
) -> Element<'a, Message> {
    let content = if let Some(sc) = shortcut {
        row![
            text(label).size(13).width(Length::Fill),
            text(sc).size(12),
        ]
        .spacing(12)
        .width(160)
    } else {
        row![text(label).size(13).width(Length::Fill),].width(160)
    };

    let style = if msg.is_some() {
        theme::menu_item_style as fn(&iced::Theme, button::Status) -> button::Style
    } else {
        theme::menu_item_disabled_style
    };

    button(content)
        .on_press_maybe(msg)
        .style(style)
        .padding(Padding::from([3, 16]))
        .into()
}

fn menu_item_checked(
    label: &'static str,
    shortcut: Option<&'static str>,
    msg: Message,
    checked: bool,
) -> Element<'static, Message> {
    let display_label = if checked {
        format!("✓ {label}")
    } else {
        format!("   {label}")
    };

    let content = if let Some(sc) = shortcut {
        row![
            text(display_label).size(13).width(Length::Fill),
            text(sc).size(12),
        ]
        .spacing(12)
        .width(160)
    } else {
        row![text(display_label).size(13).width(Length::Fill),].width(160)
    };

    button(content)
        .on_press(msg)
        .style(theme::menu_item_style)
        .padding(Padding::from([3, 16]))
        .into()
}

fn separator() -> Element<'static, Message> {
    rule::horizontal(1).into()
}
