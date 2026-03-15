use std::path::PathBuf;
use std::sync::Arc;

use iced::keyboard;
use iced::keyboard::key::Named;
use iced::keyboard::Key;
use iced::advanced::text::Wrapping;
use iced::widget::{
    button, center, checkbox, column, container, mouse_area, opaque, radio, row, scrollable, stack,
    text, text_editor, text_input, Space,
};
use iced::window;
use iced::{font, Element, Fill, Font, Padding, Subscription, Task, Theme};

use crate::file_ops;
use crate::menu::{self, MenuState};
use crate::message::{
    DialogKind, FindDirection, FontChoice, FontStyleChoice, MenuId, Message, PendingAction,
};
use crate::theme as t;

pub struct RustPad {
    content: text_editor::Content,
    file_path: Option<PathBuf>,
    is_dirty: bool,

    // Single-level undo (matches Win98 Notepad)
    undo_snapshot: Option<String>,

    // Settings
    word_wrap: bool,
    font_size: f32,
    font_family: FontChoice,
    font_style: FontStyleChoice,

    // Font dialog transient state
    font_dialog_family: FontChoice,
    font_dialog_style: FontStyleChoice,
    font_dialog_size_text: String,
    font_dialog_family_filter: String,
    font_dialog_style_filter: String,

    // Menu
    active_menu: Option<MenuId>,

    // Dialogs
    dialog: Option<DialogKind>,

    // Find/Replace state
    find_text: String,
    replace_text: String,
    find_case_sensitive: bool,
    find_whole_word: bool,
    find_direction: FindDirection,
    goto_line_text: String,

    // Alert overlay (separate from dialog so Find can stay open)
    alert_message: Option<String>,

    // Status bar
    show_status_bar: bool,

    // Pending action for save prompt
    pending_action: Option<PendingAction>,
}

impl RustPad {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                content: text_editor::Content::new(),
                file_path: None,
                is_dirty: false,
                undo_snapshot: None,
                word_wrap: false,
                font_size: 16.0,
                font_family: FontChoice::Monospace,
                font_style: FontStyleChoice::Regular,
                font_dialog_family: FontChoice::Monospace,
                font_dialog_style: FontStyleChoice::Regular,
                font_dialog_size_text: String::from("16"),
                font_dialog_family_filter: String::from("Monospace"),
                font_dialog_style_filter: String::from("Regular"),
                active_menu: None,
                dialog: None,
                find_text: String::new(),
                replace_text: String::new(),
                find_case_sensitive: false,
                find_whole_word: false,
                find_direction: FindDirection::Down,
                goto_line_text: String::new(),
                alert_message: None,
                show_status_bar: true, // on when word_wrap is off
                pending_action: None,
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        let filename = self
            .file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled");

        let modified = if self.is_dirty { "*" } else { "" };
        format!("{modified}{filename} - RustPad")
    }

    pub fn theme(&self) -> Theme {
        Theme::Light
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let keys = keyboard::listen().map(|event| match event {
            keyboard::Event::KeyPressed {
                key, modifiers, ..
            } => {
                if modifiers.command() {
                    match key.as_ref() {
                        Key::Character("n") => Some(Message::NewFile),
                        Key::Character("o") => Some(Message::OpenFile),
                        Key::Character("s") => Some(Message::SaveFile),
                        Key::Character("p") => Some(Message::Print),
                        Key::Character("f") => Some(Message::OpenFindDialog),
                        Key::Character("h") => Some(Message::OpenReplaceDialog),
                        Key::Character("g") => Some(Message::OpenGoToDialog),
                        _ => None,
                    }
                } else {
                    match key.as_ref() {
                        Key::Named(Named::Escape) => Some(Message::CloseMenus),
                        Key::Named(Named::F3) => Some(Message::FindNext),
                        Key::Named(Named::F5) => Some(Message::InsertTimeDate),
                        _ => None,
                    }
                }
            }
            _ => None,
        });

        let close = window::close_requests().map(|_id| Some(Message::CloseRequested));

        Subscription::batch([keys, close]).map(|msg| msg.unwrap_or(Message::CloseMenus))
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::EditorAction(action) => {
                // Snapshot for undo on first edit
                if action.is_edit() {
                    if self.undo_snapshot.is_none() || !self.is_dirty {
                        self.undo_snapshot = Some(self.content.text());
                    }
                    self.is_dirty = true;
                }
                self.content.perform(action);
                Task::none()
            }

            // -- File operations --
            Message::NewFile => {
                if self.is_dirty {
                    self.pending_action = Some(PendingAction::New);
                    self.dialog = Some(DialogKind::SavePrompt);
                    return Task::none();
                }
                self.do_new_file();
                Task::none()
            }
            Message::OpenFile => {
                if self.is_dirty {
                    self.pending_action = Some(PendingAction::Open);
                    self.dialog = Some(DialogKind::SavePrompt);
                    return Task::none();
                }
                self.close_menus();
                Task::perform(file_ops::open_file(), Message::FileOpened)
            }
            Message::FileOpened(result) => {
                if let Ok((path, contents)) = result {
                    self.content = text_editor::Content::with_text(&contents);
                    self.file_path = Some(path);
                    self.is_dirty = false;
                    self.undo_snapshot = None;

                    // .LOG feature: auto-insert timestamp
                    if contents.starts_with(".LOG") {
                        let timestamp =
                            chrono::Local::now().format("\n%I:%M %p %m/%d/%Y\n").to_string();
                        self.content.perform(text_editor::Action::Move(
                            text_editor::Motion::DocumentEnd,
                        ));
                        self.content.perform(text_editor::Action::Edit(
                            text_editor::Edit::Paste(Arc::new(timestamp)),
                        ));
                        self.is_dirty = true;
                    }
                }
                Task::none()
            }
            Message::SaveFile => {
                self.close_menus();
                let path = self.file_path.clone();
                let contents = self.content.text();
                Task::perform(
                    file_ops::save_file(path, contents),
                    Message::FileSaved,
                )
            }
            Message::SaveFileAs => {
                self.close_menus();
                let contents = self.content.text();
                Task::perform(
                    file_ops::save_file(None, contents),
                    Message::FileSaved,
                )
            }
            Message::FileSaved(result) => {
                if let Ok(path) = result {
                    self.file_path = Some(path);
                    self.is_dirty = false;
                    self.undo_snapshot = None;

                    // Execute pending action after save
                    if let Some(action) = self.pending_action.take() {
                        return self.execute_pending_action(action);
                    }
                }
                Task::none()
            }
            Message::Exit | Message::CloseRequested => {
                if self.is_dirty {
                    self.pending_action = Some(PendingAction::Exit);
                    self.dialog = Some(DialogKind::SavePrompt);
                    Task::none()
                } else {
                    iced::exit()
                }
            }

            // -- Edit operations --
            Message::Undo => {
                if let Some(snapshot) = self.undo_snapshot.take() {
                    let current = self.content.text();
                    self.content = text_editor::Content::with_text(&snapshot);
                    self.undo_snapshot = Some(current);
                    self.is_dirty = true;
                }
                Task::none()
            }
            Message::Cut => {
                if let Some(selection) = self.content.selection() {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(&selection);
                    }
                    self.content
                        .perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                    self.is_dirty = true;
                }
                self.close_menus();
                Task::none()
            }
            Message::Copy => {
                if let Some(selection) = self.content.selection() {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(&selection);
                    }
                }
                self.close_menus();
                Task::none()
            }
            Message::Paste => {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    if let Ok(text) = clipboard.get_text() {
                        self.content.perform(text_editor::Action::Edit(
                            text_editor::Edit::Paste(Arc::new(text)),
                        ));
                        self.is_dirty = true;
                    }
                }
                self.close_menus();
                Task::none()
            }
            Message::Delete => {
                self.content
                    .perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                self.is_dirty = true;
                self.close_menus();
                Task::none()
            }
            Message::SelectAll => {
                self.content
                    .perform(text_editor::Action::SelectAll);
                self.close_menus();
                Task::none()
            }
            Message::InsertTimeDate => {
                let timestamp = chrono::Local::now()
                    .format("%I:%M %p %m/%d/%Y")
                    .to_string();
                self.content.perform(text_editor::Action::Edit(
                    text_editor::Edit::Paste(Arc::new(timestamp)),
                ));
                self.is_dirty = true;
                self.close_menus();
                Task::none()
            }
            Message::ToggleWordWrap => {
                self.word_wrap = !self.word_wrap;
                // Win98: status bar disabled when word wrap is on
                self.show_status_bar = !self.word_wrap;
                self.close_menus();
                Task::none()
            }

            // -- Search operations (Phase 4) --
            Message::OpenFindDialog => {
                self.dialog = Some(DialogKind::Find);
                self.close_menus();
                Task::none()
            }
            Message::OpenReplaceDialog => {
                self.dialog = Some(DialogKind::Replace);
                self.close_menus();
                Task::none()
            }
            Message::OpenGoToDialog => {
                if self.word_wrap {
                    return Task::none();
                }
                self.dialog = Some(DialogKind::GoTo);
                self.close_menus();
                Task::none()
            }
            Message::FindNext => {
                match self.find_direction {
                    FindDirection::Down => self.do_find_next(),
                    FindDirection::Up => self.do_find_previous(),
                }
                Task::none()
            }
            Message::DoReplace => {
                self.do_replace();
                Task::none()
            }
            Message::DoReplaceAll => {
                self.do_replace_all();
                Task::none()
            }
            Message::GoToLine => {
                self.do_goto_line();
                self.dialog = None;
                Task::none()
            }

            // Dialog field updates
            Message::FindTextChanged(val) => {
                self.find_text = val;
                Task::none()
            }
            Message::ReplaceTextChanged(val) => {
                self.replace_text = val;
                Task::none()
            }
            Message::GoToLineChanged(val) => {
                self.goto_line_text = val;
                Task::none()
            }
            Message::ToggleCaseSensitive(val) => {
                self.find_case_sensitive = val;
                Task::none()
            }
            Message::ToggleWholeWord(val) => {
                self.find_whole_word = val;
                Task::none()
            }
            Message::FindDirectionChanged(dir) => {
                self.find_direction = dir;
                Task::none()
            }
            Message::DismissAlert => {
                self.alert_message = None;
                Task::none()
            }
            Message::CloseDialog => {
                self.dialog = None;
                Task::none()
            }

            // -- Menu --
            Message::MenuClicked(id) => {
                if self.active_menu == Some(id) {
                    self.active_menu = None;
                } else {
                    self.active_menu = Some(id);
                }
                Task::none()
            }
            Message::CloseMenus => {
                self.close_menus();
                Task::none()
            }

            // -- Help --
            Message::ShowAbout => {
                self.dialog = Some(DialogKind::About);
                self.close_menus();
                Task::none()
            }

            // -- Save prompt --
            Message::SavePromptSave => {
                self.dialog = None;
                let path = self.file_path.clone();
                let contents = self.content.text();
                Task::perform(
                    file_ops::save_file(path, contents),
                    Message::FileSaved,
                )
            }
            Message::SavePromptDontSave => {
                self.dialog = None;
                if let Some(action) = self.pending_action.take() {
                    return self.execute_pending_action(action);
                }
                Task::none()
            }
            Message::SavePromptCancel => {
                self.dialog = None;
                self.pending_action = None;
                Task::none()
            }

            // -- Font dialog --
            Message::OpenFontDialog => {
                self.font_dialog_family = self.font_family;
                self.font_dialog_style = self.font_style;
                self.font_dialog_size_text = self.font_size.to_string();
                self.font_dialog_family_filter = self.font_family.to_string();
                self.font_dialog_style_filter = self.font_style.to_string();
                self.dialog = Some(DialogKind::Font);
                self.close_menus();
                Task::none()
            }
            Message::FontFamilyChanged(choice) => {
                self.font_dialog_family = choice;
                self.font_dialog_family_filter = choice.to_string();
                Task::none()
            }
            Message::FontFamilyFilterChanged(val) => {
                self.font_dialog_family_filter = val;
                Task::none()
            }
            Message::FontStyleChanged(choice) => {
                self.font_dialog_style = choice;
                self.font_dialog_style_filter = choice.to_string();
                Task::none()
            }
            Message::FontStyleFilterChanged(val) => {
                self.font_dialog_style_filter = val;
                Task::none()
            }
            Message::FontSizeChanged(val) => {
                self.font_dialog_size_text = val;
                Task::none()
            }
            Message::FontSizeSelected(val) => {
                self.font_dialog_size_text = val;
                Task::none()
            }
            Message::ApplyFont => {
                self.font_family = self.font_dialog_family;
                self.font_style = self.font_dialog_style;
                if let Ok(size) = self.font_dialog_size_text.parse::<f32>() {
                    self.font_size = size.clamp(8.0, 72.0);
                }
                self.dialog = None;
                Task::none()
            }

            Message::Print => {
                self.close_menus();
                let contents = self.content.text();
                Task::perform(file_ops::print_file(contents), Message::PrintResult)
            }
            Message::PrintResult(_) => Task::none(),

            // Stubs
            Message::PageSetup => {
                self.close_menus();
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let menu_bar = menu::view_menu_bar(self.active_menu);

        let editor = {
            let editor_font = Font {
                family: self.font_family.to_iced_family(),
                weight: match self.font_style {
                    FontStyleChoice::Bold | FontStyleChoice::BoldItalic => font::Weight::Bold,
                    _ => font::Weight::Normal,
                },
                style: match self.font_style {
                    FontStyleChoice::Italic | FontStyleChoice::BoldItalic => font::Style::Italic,
                    _ => font::Style::Normal,
                },
                ..Font::default()
            };

            let mut ed = text_editor(&self.content)
                .on_action(Message::EditorAction)
                .font(editor_font)
                .size(self.font_size)
                .style(t::editor_style)
                .padding(Padding::from([1, 1]))
                .height(Fill);

            if self.word_wrap {
                ed = ed.wrapping(Wrapping::Word);
            } else {
                ed = ed.wrapping(Wrapping::None);
            }

            ed
        };

        let editor_with_border = container(editor)
            .style(t::win98_sunken_editor_style)
            .width(Fill)
            .height(Fill);

        let status_bar: Element<'_, Message> = if self.show_status_bar {
            let (line, col) = self.cursor_position();
            container(
                row![
                    Space::new().width(Fill),
                    container(
                        text(format!("Ln {}, Col {}", line, col)).size(12),
                    )
                    .style(t::win98_sunken_container_style)
                    .padding(Padding::from([1, 8])),
                ]
                .padding(Padding::from([2, 4])),
            )
            .style(t::status_bar_style)
            .width(Fill)
            .height(t::STATUS_BAR_HEIGHT)
            .into()
        } else {
            Space::new().into()
        };

        let below_menu = column![editor_with_border, status_bar];

        // Layer dropdown menu if active — menu_bar stays ABOVE the overlay
        // so hover-to-switch between menu buttons keeps working.
        let with_menu = if let Some(menu_id) = self.active_menu {
            let menu_state = MenuState {
                has_undo: self.undo_snapshot.is_some(),
                has_selection: self.content.selection().is_some(),
                word_wrap: self.word_wrap,
            };
            let dropdown = menu::view_dropdown(menu_id, &menu_state);

            let x_offset = match menu_id {
                MenuId::File => 0.0,
                MenuId::Edit => 40.0,
                MenuId::Format => 76.0,
                MenuId::Search => 130.0,
                MenuId::Help => 182.0,
            };

            column![
                menu_bar,
                stack![
                    below_menu,
                    // Full-area click catcher (closes menu on click outside)
                    mouse_area(
                        container(Space::new()).width(Fill).height(Fill)
                    )
                    .on_press(Message::CloseMenus),
                    // Dropdown: opaque so clicks on it don't close the menu
                    container(opaque(dropdown)).padding(Padding {
                        top: 0.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: x_offset,
                    })
                ]
            ]
            .into()
        } else {
            column![menu_bar, below_menu].into()
        };

        // Layer dialog if active
        let with_dialog = if let Some(dialog_kind) = &self.dialog {
            let dialog_content = match dialog_kind {
                DialogKind::Find => self.view_find_dialog(),
                DialogKind::Replace => self.view_replace_dialog(),
                DialogKind::GoTo => self.view_goto_dialog(),
                DialogKind::About => self.view_about_dialog(),
                DialogKind::SavePrompt => self.view_save_prompt(),
                DialogKind::Font => self.view_font_dialog(),
            };

            stack![
                with_menu,
                mouse_area(
                    container(opaque(center(dialog_content)))
                        .width(Fill)
                        .height(Fill),
                )
                .on_press(Message::CloseDialog)
            ]
            .into()
        } else {
            with_menu
        };

        // Layer alert on top of everything (so Find dialog stays open behind it)
        if let Some(ref msg) = self.alert_message {
            let alert = self.view_alert_dialog(msg);
            stack![
                with_dialog,
                mouse_area(
                    container(opaque(center(alert)))
                        .width(Fill)
                        .height(Fill),
                )
                .on_press(Message::DismissAlert)
            ]
            .into()
        } else {
            with_dialog
        }
    }

    // -- Dialog views --

    fn dialog_title_bar(title: &str) -> Element<'_, Message> {
        container(
            row![
                text(title)
                    .size(12)
                    .color(iced::Color::WHITE)
                    .width(Fill),
                button(text("X").size(10).color(iced::Color::WHITE))
                    .on_press(Message::CloseDialog)
                    .style(t::dialog_title_bar_close_style)
                    .padding(Padding::from([1, 4])),
            ]
            .align_y(iced::Alignment::Center)
            .padding(Padding::from([2, 4])),
        )
        .style(t::dialog_title_bar_style)
        .width(Fill)
        .into()
    }

    fn view_find_dialog(&self) -> Element<'_, Message> {
        let content = container(
            column![
                Self::dialog_title_bar("Find"),
                column![
                    row![
                        text("Find what:").size(13).width(70),
                        text_input("", &self.find_text)
                            .on_input(Message::FindTextChanged)
                            .on_submit(Message::FindNext)
                            .size(13)
                            .width(200)
                            .style(t::win98_text_input_style),
                        button(text("Find Next").size(13))
                            .on_press(Message::FindNext)
                            .padding(Padding::from([4, 12]))
                            .style(t::win98_button_style),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                    row![
                        column![
                            checkbox(self.find_whole_word)
                                .label("Match whole word only")
                                .on_toggle(Message::ToggleWholeWord)
                                .size(14)
                                .text_size(13)
                                .style(t::win98_checkbox_style),
                            checkbox(self.find_case_sensitive)
                                .label("Match case")
                                .on_toggle(Message::ToggleCaseSensitive)
                                .size(14)
                                .text_size(13)
                                .style(t::win98_checkbox_style),
                        ]
                        .spacing(4),
                        Space::new().width(Fill),
                        column![
                            text("Direction").size(12),
                            row![
                                radio("Up", FindDirection::Up, Some(self.find_direction), Message::FindDirectionChanged)
                                    .size(14)
                                    .text_size(13),
                                radio("Down", FindDirection::Down, Some(self.find_direction), Message::FindDirectionChanged)
                                    .size(14)
                                    .text_size(13),
                            ]
                            .spacing(8),
                        ]
                        .spacing(4),
                        button(text("Cancel").size(13))
                            .on_press(Message::CloseDialog)
                            .padding(Padding::from([4, 12]))
                            .style(t::win98_button_style),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::End),
                ]
                .spacing(8)
                .padding(12),
            ]
            .spacing(0),
        )
        .style(t::dialog_container_style)
        .padding(0);

        mouse_area(opaque(content))
            .on_press(Message::FindTextChanged(self.find_text.clone())) // absorb click
            .into()
    }

    fn view_replace_dialog(&self) -> Element<'_, Message> {
        let content = container(
            column![
                Self::dialog_title_bar("Replace"),
                column![
                    row![
                        text("Find what:").size(13).width(80),
                        text_input("", &self.find_text)
                            .on_input(Message::FindTextChanged)
                            .size(13)
                            .width(200)
                            .style(t::win98_text_input_style),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                    row![
                        text("Replace with:").size(13).width(80),
                        text_input("", &self.replace_text)
                            .on_input(Message::ReplaceTextChanged)
                            .size(13)
                            .width(200)
                            .style(t::win98_text_input_style),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                    row![
                        button(text("Find Next").size(13))
                            .on_press(Message::FindNext)
                            .padding(Padding::from([4, 12]))
                            .style(t::win98_button_style),
                        button(text("Replace").size(13))
                            .on_press(Message::DoReplace)
                            .padding(Padding::from([4, 12]))
                            .style(t::win98_button_style),
                        button(text("Replace All").size(13))
                            .on_press(Message::DoReplaceAll)
                            .padding(Padding::from([4, 12]))
                            .style(t::win98_button_style),
                        button(text("Cancel").size(13))
                            .on_press(Message::CloseDialog)
                            .padding(Padding::from([4, 12]))
                            .style(t::win98_button_style),
                    ]
                    .spacing(8),
                    row![
                        checkbox(self.find_whole_word)
                            .label("Match whole word only")
                            .on_toggle(Message::ToggleWholeWord)
                            .size(14)
                            .text_size(13)
                            .style(t::win98_checkbox_style),
                        checkbox(self.find_case_sensitive)
                            .label("Match case")
                            .on_toggle(Message::ToggleCaseSensitive)
                            .size(14)
                            .text_size(13)
                            .style(t::win98_checkbox_style),
                    ]
                    .spacing(16),
                ]
                .spacing(8)
                .padding(12),
            ]
            .spacing(0),
        )
        .style(t::dialog_container_style)
        .padding(0);

        mouse_area(opaque(content))
            .on_press(Message::ReplaceTextChanged(self.replace_text.clone()))
            .into()
    }

    fn view_goto_dialog(&self) -> Element<'_, Message> {
        let content = container(
            column![
                Self::dialog_title_bar("Go To Line"),
                column![
                    text_input("Line number", &self.goto_line_text)
                        .on_input(Message::GoToLineChanged)
                        .on_submit(Message::GoToLine)
                        .size(13)
                        .width(200)
                        .style(t::win98_text_input_style),
                    row![
                        button(text("Go To").size(13))
                            .on_press(Message::GoToLine)
                            .padding(Padding::from([4, 12]))
                            .style(t::win98_button_style),
                        button(text("Cancel").size(13))
                            .on_press(Message::CloseDialog)
                            .padding(Padding::from([4, 12]))
                            .style(t::win98_button_style),
                    ]
                    .spacing(8),
                ]
                .spacing(8)
                .padding(12),
            ]
            .spacing(0),
        )
        .style(t::dialog_container_style)
        .padding(0);

        mouse_area(opaque(content))
            .on_press(Message::GoToLineChanged(self.goto_line_text.clone()))
            .into()
    }

    fn view_about_dialog(&self) -> Element<'_, Message> {
        let content = container(
            column![
                Self::dialog_title_bar("About RustPad"),
                column![
                    text("RustPad").size(20),
                    text("A Windows 98 Notepad clone").size(13),
                    text("Built with Rust and Iced").size(13),
                    text("Version 0.1.0").size(12),
                    button(text("OK").size(13))
                        .on_press(Message::CloseDialog)
                        .padding(Padding::from([4, 20]))
                        .style(t::win98_button_style),
                ]
                .spacing(8)
                .align_x(iced::Alignment::Center)
                .padding(16),
            ]
            .spacing(0),
        )
        .style(t::dialog_container_style)
        .padding(0);

        mouse_area(opaque(content))
            .on_press(Message::ShowAbout) // absorb click
            .into()
    }

    fn view_save_prompt(&self) -> Element<'_, Message> {
        let filename = self
            .file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled");

        let content = container(
            column![
                Self::dialog_title_bar("RustPad"),
                column![
                    text(format!(
                        "Do you want to save changes to {filename}?"
                    ))
                    .size(14),
                    row![
                        button(text("Save").size(13))
                            .on_press(Message::SavePromptSave)
                            .padding(Padding::from([4, 16]))
                            .style(t::win98_button_style),
                        button(text("Don't Save").size(13))
                            .on_press(Message::SavePromptDontSave)
                            .padding(Padding::from([4, 16]))
                            .style(t::win98_button_style),
                        button(text("Cancel").size(13))
                            .on_press(Message::SavePromptCancel)
                            .padding(Padding::from([4, 16]))
                            .style(t::win98_button_style),
                    ]
                    .spacing(8),
                ]
                .spacing(12)
                .align_x(iced::Alignment::Center)
                .padding(16),
            ]
            .spacing(0),
        )
        .style(t::dialog_container_style)
        .padding(0);

        mouse_area(opaque(content))
            .on_press(Message::SavePromptCancel) // absorb click, treat as cancel
            .into()
    }

    fn view_font_dialog(&self) -> Element<'_, Message> {
        let preview_font = Font {
            family: self.font_dialog_family.to_iced_family(),
            weight: match self.font_dialog_style {
                FontStyleChoice::Bold | FontStyleChoice::BoldItalic => font::Weight::Bold,
                _ => font::Weight::Normal,
            },
            style: match self.font_dialog_style {
                FontStyleChoice::Italic | FontStyleChoice::BoldItalic => font::Style::Italic,
                _ => font::Style::Normal,
            },
            ..Font::default()
        };

        let preview_size = self
            .font_dialog_size_text
            .parse::<f32>()
            .unwrap_or(16.0)
            .clamp(8.0, 72.0);

        // Font family scrollable list
        let filter_lower = self.font_dialog_family_filter.to_lowercase();
        let font_items: Vec<Element<'_, Message>> = FontChoice::ALL
            .iter()
            .filter(|f| {
                filter_lower.is_empty() || f.to_string().to_lowercase().contains(&filter_lower)
            })
            .map(|&f| {
                let is_selected = f == self.font_dialog_family;
                let style = if is_selected {
                    t::win98_list_item_selected_style
                        as fn(&iced::Theme, button::Status) -> button::Style
                } else {
                    t::win98_list_item_style
                };
                button(text(f.to_string()).size(13))
                    .on_press(Message::FontFamilyChanged(f))
                    .style(style)
                    .width(Fill)
                    .padding(Padding::from([2, 4]))
                    .into()
            })
            .collect();

        let font_list = column![
            text("Font:").size(12),
            text_input("", &self.font_dialog_family_filter)
                .on_input(Message::FontFamilyFilterChanged)
                .size(13)
                .width(Fill)
                .style(t::win98_text_input_style),
            container(
                scrollable(column(font_items).spacing(0).width(Fill))
                    .height(100),
            )
            .style(t::win98_sunken_container_style)
            .width(Fill),
        ]
        .spacing(2)
        .width(160);

        // Font style scrollable list
        let style_filter_lower = self.font_dialog_style_filter.to_lowercase();
        let style_items: Vec<Element<'_, Message>> = FontStyleChoice::ALL
            .iter()
            .filter(|s| {
                style_filter_lower.is_empty()
                    || s.to_string().to_lowercase().contains(&style_filter_lower)
            })
            .map(|&s| {
                let is_selected = s == self.font_dialog_style;
                let style = if is_selected {
                    t::win98_list_item_selected_style
                        as fn(&iced::Theme, button::Status) -> button::Style
                } else {
                    t::win98_list_item_style
                };
                button(text(s.to_string()).size(13))
                    .on_press(Message::FontStyleChanged(s))
                    .style(style)
                    .width(Fill)
                    .padding(Padding::from([2, 4]))
                    .into()
            })
            .collect();

        let style_list = column![
            text("Font Style:").size(12),
            text_input("", &self.font_dialog_style_filter)
                .on_input(Message::FontStyleFilterChanged)
                .size(13)
                .width(Fill)
                .style(t::win98_text_input_style),
            container(
                scrollable(column(style_items).spacing(0).width(Fill))
                    .height(100),
            )
            .style(t::win98_sunken_container_style)
            .width(Fill),
        ]
        .spacing(2)
        .width(120);

        // Size scrollable list
        let sizes = [
            "8", "9", "10", "11", "12", "14", "16", "18", "20", "22", "24", "26", "28", "36",
            "48", "72",
        ];
        let size_items: Vec<Element<'_, Message>> = sizes
            .iter()
            .filter(|s| {
                self.font_dialog_size_text.is_empty()
                    || s.starts_with(&self.font_dialog_size_text)
            })
            .map(|&s| {
                let is_selected = s == self.font_dialog_size_text;
                let style = if is_selected {
                    t::win98_list_item_selected_style
                        as fn(&iced::Theme, button::Status) -> button::Style
                } else {
                    t::win98_list_item_style
                };
                button(text(s).size(13))
                    .on_press(Message::FontSizeSelected(s.to_string()))
                    .style(style)
                    .width(Fill)
                    .padding(Padding::from([2, 4]))
                    .into()
            })
            .collect();

        let size_list = column![
            text("Size:").size(12),
            text_input("", &self.font_dialog_size_text)
                .on_input(Message::FontSizeChanged)
                .size(13)
                .width(Fill)
                .style(t::win98_text_input_style),
            container(
                scrollable(column(size_items).spacing(0).width(Fill))
                    .height(100),
            )
            .style(t::win98_sunken_container_style)
            .width(Fill),
        ]
        .spacing(2)
        .width(70);

        let content = container(
            column![
                Self::dialog_title_bar("Font"),
                column![
                    row![font_list, style_list, size_list,].spacing(12),
                    column![
                        text("Sample").size(11),
                        container(
                            text("AaBbYyZz").font(preview_font).size(preview_size),
                        )
                        .style(t::win98_sunken_container_style)
                        .padding(12)
                        .width(Fill),
                    ]
                    .spacing(4),
                    row![
                        button(text("OK").size(13))
                            .on_press(Message::ApplyFont)
                            .padding(Padding::from([4, 20]))
                            .style(t::win98_button_style),
                        button(text("Cancel").size(13))
                            .on_press(Message::CloseDialog)
                            .padding(Padding::from([4, 20]))
                            .style(t::win98_button_style),
                    ]
                    .spacing(8),
                ]
                .spacing(12)
                .padding(12),
            ]
            .spacing(0),
        )
        .style(t::dialog_container_style)
        .padding(0);

        mouse_area(opaque(content))
            .on_press(Message::FontSizeChanged(self.font_dialog_size_text.clone()))
            .into()
    }

    fn view_alert_dialog<'a>(&'a self, msg: &'a str) -> Element<'a, Message> {
        let content = container(
            column![
                Self::dialog_title_bar("RustPad"),
                column![
                    text(msg).size(13),
                    button(text("OK").size(13))
                        .on_press(Message::DismissAlert)
                        .padding(Padding::from([4, 20]))
                        .style(t::win98_button_style),
                ]
                .spacing(12)
                .align_x(iced::Alignment::Center)
                .padding(16),
            ]
            .spacing(0),
        )
        .style(t::dialog_container_style)
        .padding(0);

        mouse_area(opaque(content))
            .on_press(Message::DismissAlert)
            .into()
    }

    // -- Helpers --

    fn close_menus(&mut self) {
        self.active_menu = None;
    }

    fn do_new_file(&mut self) {
        self.content = text_editor::Content::new();
        self.file_path = None;
        self.is_dirty = false;
        self.undo_snapshot = None;
    }

    fn execute_pending_action(&mut self, action: PendingAction) -> Task<Message> {
        match action {
            PendingAction::New => {
                self.do_new_file();
                Task::none()
            }
            PendingAction::Open => {
                Task::perform(file_ops::open_file(), Message::FileOpened)
            }
            PendingAction::Exit => iced::exit(),
        }
    }

    fn cursor_position(&self) -> (usize, usize) {
        let cursor = self.content.cursor();
        (cursor.position.line + 1, cursor.position.column + 1)
    }

    fn is_whole_word_match(&self, text: &str, pos: usize, len: usize) -> bool {
        if !self.find_whole_word {
            return true;
        }
        let before_ok = pos == 0
            || !text[..pos]
                .chars()
                .next_back()
                .map_or(false, |c| c.is_alphanumeric() || c == '_');
        let after_ok = pos + len >= text.len()
            || !text[pos + len..]
                .chars()
                .next()
                .map_or(false, |c| c.is_alphanumeric() || c == '_');
        before_ok && after_ok
    }

    fn do_find_next(&mut self) {
        if self.find_text.is_empty() {
            return;
        }

        let full_text = self.content.text();
        let cursor = self.content.cursor();
        let cur_line = cursor.position.line;
        let cur_col = cursor.position.column;

        // Calculate byte offset from line/col
        let mut offset = 0;
        for (i, line) in full_text.lines().enumerate() {
            if i == cur_line {
                offset += cur_col;
                break;
            }
            offset += line.len() + 1; // +1 for newline
        }

        // Search from cursor position
        let search_text = if self.find_case_sensitive {
            full_text.clone()
        } else {
            full_text.to_lowercase()
        };
        let needle = if self.find_case_sensitive {
            self.find_text.clone()
        } else {
            self.find_text.to_lowercase()
        };

        // Try from cursor, then wrap around
        let found_offset = self
            .find_forward(&search_text, &needle, offset)
            .or_else(|| self.find_forward(&search_text, &needle, 0));

        if let Some(byte_pos) = found_offset {
            self.select_range(byte_pos, needle.len(), &full_text);
        } else {
            self.alert_message = Some(format!("Cannot find \"{}\"", self.find_text));
        }
    }

    fn find_forward(&self, haystack: &str, needle: &str, start: usize) -> Option<usize> {
        let mut pos = start;
        while pos < haystack.len() {
            if let Some(found) = haystack[pos..].find(needle) {
                let abs = pos + found;
                if self.is_whole_word_match(haystack, abs, needle.len()) {
                    return Some(abs);
                }
                pos = abs + 1;
            } else {
                break;
            }
        }
        None
    }

    fn do_find_previous(&mut self) {
        if self.find_text.is_empty() {
            return;
        }

        let full_text = self.content.text();
        let cursor = self.content.cursor();
        let cur_line = cursor.position.line;
        let cur_col = cursor.position.column;

        let mut offset = 0;
        for (i, line) in full_text.lines().enumerate() {
            if i == cur_line {
                offset += cur_col;
                break;
            }
            offset += line.len() + 1;
        }

        let search_text = if self.find_case_sensitive {
            full_text.clone()
        } else {
            full_text.to_lowercase()
        };
        let needle = if self.find_case_sensitive {
            self.find_text.clone()
        } else {
            self.find_text.to_lowercase()
        };

        let found_offset = self
            .find_backward(&search_text, &needle, offset)
            .or_else(|| self.find_backward(&search_text, &needle, search_text.len()));

        if let Some(byte_pos) = found_offset {
            self.select_range(byte_pos, needle.len(), &full_text);
        } else {
            self.alert_message = Some(format!("Cannot find \"{}\"", self.find_text));
        }
    }

    fn find_backward(&self, haystack: &str, needle: &str, end: usize) -> Option<usize> {
        let search_area = &haystack[..end.min(haystack.len())];
        let mut last_valid = None;
        let mut pos = 0;
        while pos < search_area.len() {
            if let Some(found) = search_area[pos..].find(needle) {
                let abs = pos + found;
                if self.is_whole_word_match(haystack, abs, needle.len()) {
                    last_valid = Some(abs);
                }
                pos = abs + 1;
            } else {
                break;
            }
        }
        last_valid
    }

    fn do_replace(&mut self) {
        // If text is selected and matches find_text, replace it, then find next
        if let Some(selection) = self.content.selection() {
            let matches = if self.find_case_sensitive {
                selection == self.find_text
            } else {
                selection.to_lowercase() == self.find_text.to_lowercase()
            };

            if matches {
                self.content.perform(text_editor::Action::Edit(
                    text_editor::Edit::Paste(Arc::new(self.replace_text.clone())),
                ));
                self.is_dirty = true;
            }
        }
        self.do_find_next();
    }

    fn do_replace_all(&mut self) {
        if self.find_text.is_empty() {
            return;
        }

        let full_text = self.content.text();
        let new_text = if self.find_case_sensitive {
            full_text.replace(&self.find_text, &self.replace_text)
        } else {
            // Case-insensitive replace
            let mut result = String::new();
            let lower = full_text.to_lowercase();
            let needle = self.find_text.to_lowercase();
            let mut last = 0;
            for (start, _) in lower.match_indices(&needle) {
                result.push_str(&full_text[last..start]);
                result.push_str(&self.replace_text);
                last = start + self.find_text.len();
            }
            result.push_str(&full_text[last..]);
            result
        };

        if new_text != full_text {
            self.undo_snapshot = Some(full_text);
            self.content = text_editor::Content::with_text(&new_text);
            self.is_dirty = true;
        }
    }

    fn do_goto_line(&mut self) {
        if let Ok(line_num) = self.goto_line_text.parse::<usize>() {
            if line_num > 0 {
                let target_line = line_num - 1;

                // Move cursor to start of target line
                self.content
                    .perform(text_editor::Action::Move(text_editor::Motion::DocumentStart));
                for _ in 0..target_line {
                    self.content
                        .perform(text_editor::Action::Move(text_editor::Motion::Down));
                }
                self.content
                    .perform(text_editor::Action::Move(text_editor::Motion::Home));
            }
        }
    }

    fn select_range(&mut self, byte_pos: usize, len: usize, full_text: &str) {
        // Convert byte position to line/col
        let mut target_line = 0;
        let mut target_col = 0;
        let mut counted = 0;

        for (i, line) in full_text.lines().enumerate() {
            if counted + line.len() >= byte_pos {
                target_line = i;
                target_col = byte_pos - counted;
                break;
            }
            counted += line.len() + 1; // +1 for newline
        }

        // Move cursor to start of match
        self.content
            .perform(text_editor::Action::Move(text_editor::Motion::DocumentStart));
        for _ in 0..target_line {
            self.content
                .perform(text_editor::Action::Move(text_editor::Motion::Down));
        }
        self.content
            .perform(text_editor::Action::Move(text_editor::Motion::Home));
        for _ in 0..target_col {
            self.content
                .perform(text_editor::Action::Move(text_editor::Motion::Right));
        }

        // Select the match
        for _ in 0..len {
            self.content
                .perform(text_editor::Action::Select(text_editor::Motion::Right));
        }
    }
}
