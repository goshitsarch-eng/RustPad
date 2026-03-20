use std::path::PathBuf;
use std::sync::Arc;

use iced::advanced::text::Wrapping;
use iced::keyboard;
use iced::keyboard::Key;
use iced::keyboard::key::Named;
use iced::widget::{
    Space, button, center, checkbox, column, container, mouse_area, opaque, radio, row, scrollable,
    stack, text, text_editor, text_input,
};
use iced::window;
use iced::{Element, Fill, Font, Length, Padding, Point, Subscription, Task, Theme, font};

use crate::file_ops;
use crate::menu::{self, ContextMenuState, MenuState};
use crate::message::{
    DialogKind, FindDirection, FontChoice, FontStyleChoice, MenuId, Message, PendingAction,
};
use crate::settings;
use crate::theme as t;

pub struct RustPad {
    content: text_editor::Content,
    file_path: Option<PathBuf>,
    is_dirty: bool,

    // Single-level undo (matches Win98 Notepad)
    undo_snapshot: Option<String>,

    // Settings
    word_wrap: bool,
    dark_mode: bool,
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
    context_menu_position: Option<Point>,
    editor_pointer_position: Point,

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
        let (dark_mode, alert_message) = match settings::load() {
            Ok(settings) => (settings.dark_mode, None),
            Err(error) => (false, Some(error.to_string())),
        };

        (
            Self {
                content: text_editor::Content::new(),
                file_path: None,
                is_dirty: false,
                undo_snapshot: None,
                word_wrap: false,
                dark_mode,
                font_size: 16.0,
                font_family: FontChoice::Monospace,
                font_style: FontStyleChoice::Regular,
                font_dialog_family: FontChoice::Monospace,
                font_dialog_style: FontStyleChoice::Regular,
                font_dialog_size_text: String::from("16"),
                font_dialog_family_filter: String::from("Monospace"),
                font_dialog_style_filter: String::from("Regular"),
                active_menu: None,
                context_menu_position: None,
                editor_pointer_position: Point::ORIGIN,
                dialog: None,
                find_text: String::new(),
                replace_text: String::new(),
                find_case_sensitive: false,
                find_whole_word: false,
                find_direction: FindDirection::Down,
                goto_line_text: String::new(),
                alert_message,
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
        if self.dark_mode {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let keys = keyboard::listen().map(|event| match event {
            keyboard::Event::KeyPressed { key, modifiers, .. } => {
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

        Subscription::batch([keys, close]).map(|msg| msg.unwrap_or(Message::Ignored))
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Ignored => Task::none(),
            Message::EditorAction(action) => {
                // Snapshot for undo on first edit
                if action.is_edit() {
                    self.capture_undo_snapshot();
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
                    self.close_menus();
                    return Task::none();
                }
                self.do_new_file();
                Task::none()
            }
            Message::OpenFile => {
                if self.is_dirty {
                    self.pending_action = Some(PendingAction::Open);
                    self.dialog = Some(DialogKind::SavePrompt);
                    self.close_menus();
                    return Task::none();
                }
                self.close_menus();
                Task::perform(file_ops::open_file(), Message::FileOpened)
            }
            Message::FileOpened(result) => {
                match result {
                    Ok((path, contents)) => {
                        self.content = text_editor::Content::with_text(&contents);
                        self.file_path = Some(path);
                        self.is_dirty = false;
                        self.undo_snapshot = None;
                        self.alert_message = None;

                        // .LOG feature: auto-insert timestamp
                        if contents.starts_with(".LOG") {
                            self.capture_undo_snapshot();
                            let timestamp = chrono::Local::now()
                                .format("\n%I:%M %p %m/%d/%Y\n")
                                .to_string();
                            self.content.perform(text_editor::Action::Move(
                                text_editor::Motion::DocumentEnd,
                            ));
                            self.content.perform(text_editor::Action::Edit(
                                text_editor::Edit::Paste(Arc::new(timestamp)),
                            ));
                            self.is_dirty = true;
                        }
                    }
                    Err(file_ops::FileError::DialogClosed) => {}
                    Err(error) => {
                        self.pending_action = None;
                        self.show_alert(error.to_string());
                    }
                }
                Task::none()
            }
            Message::SaveFile => {
                self.close_menus();
                let path = self.file_path.clone();
                let contents = self.content.text();
                Task::perform(file_ops::save_file(path, contents), Message::FileSaved)
            }
            Message::SaveFileAs => {
                self.close_menus();
                let contents = self.content.text();
                Task::perform(file_ops::save_file(None, contents), Message::FileSaved)
            }
            Message::FileSaved(result) => {
                match result {
                    Ok(path) => {
                        self.file_path = Some(path);
                        self.is_dirty = false;
                        self.undo_snapshot = None;
                        self.alert_message = None;

                        // Execute pending action after save
                        if let Some(action) = self.pending_action.take() {
                            return self.execute_pending_action(action);
                        }
                    }
                    Err(file_ops::FileError::DialogClosed) => {
                        self.pending_action = None;
                    }
                    Err(error) => {
                        self.pending_action = None;
                        self.show_alert(error.to_string());
                    }
                }
                Task::none()
            }
            Message::Exit | Message::CloseRequested => {
                if self.is_dirty {
                    self.pending_action = Some(PendingAction::Exit);
                    self.dialog = Some(DialogKind::SavePrompt);
                    self.close_menus();
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
                self.close_menus();
                Task::none()
            }
            Message::Cut => {
                if let Some(selection) = self.content.selection() {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(&selection);
                    }
                    self.capture_undo_snapshot();
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
                        self.capture_undo_snapshot();
                        self.content
                            .perform(text_editor::Action::Edit(text_editor::Edit::Paste(
                                Arc::new(text),
                            )));
                        self.is_dirty = true;
                    }
                }
                self.close_menus();
                Task::none()
            }
            Message::Delete => {
                self.capture_undo_snapshot();
                self.content
                    .perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                self.is_dirty = true;
                self.close_menus();
                Task::none()
            }
            Message::SelectAll => {
                self.content.perform(text_editor::Action::SelectAll);
                self.close_menus();
                Task::none()
            }
            Message::InsertTimeDate => {
                let timestamp = chrono::Local::now().format("%I:%M %p %m/%d/%Y").to_string();
                self.capture_undo_snapshot();
                self.content
                    .perform(text_editor::Action::Edit(text_editor::Edit::Paste(
                        Arc::new(timestamp),
                    )));
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
            Message::ToggleDarkMode => {
                self.dark_mode = !self.dark_mode;
                if let Err(error) = self.persist_settings() {
                    self.show_alert(error.to_string());
                }
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
                if self.do_goto_line() {
                    self.dialog = None;
                }
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
            Message::EditorPointerMoved(position) => {
                self.editor_pointer_position = position;
                Task::none()
            }
            Message::OpenEditorContextMenu => {
                self.active_menu = None;
                self.context_menu_position = Some(self.editor_pointer_position);
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
                if self.dialog == Some(DialogKind::SavePrompt) {
                    self.pending_action = None;
                }
                self.dialog = None;
                Task::none()
            }

            // -- Menu --
            Message::MenuClicked(id) => {
                self.context_menu_position = None;
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
                Task::perform(file_ops::save_file(path, contents), Message::FileSaved)
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
                self.font_dialog_family_filter.clear();
                self.font_dialog_style_filter.clear();
                self.dialog = Some(DialogKind::Font);
                self.close_menus();
                Task::none()
            }
            Message::FontFamilyChanged(choice) => {
                self.font_dialog_family = choice;
                Task::none()
            }
            Message::FontFamilyFilterChanged(val) => {
                self.font_dialog_family_filter = val;
                Task::none()
            }
            Message::FontStyleChanged(choice) => {
                self.font_dialog_style = choice;
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
            Message::PrintResult(result) => {
                if let Err(error) = result {
                    self.show_alert(error.to_string());
                }
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

        let editor_with_border = mouse_area(
            container(editor)
                .style(t::win98_sunken_editor_style)
                .width(Fill)
                .height(Fill),
        )
        .on_move(Message::EditorPointerMoved)
        .on_right_press(Message::OpenEditorContextMenu);

        let status_bar: Element<'_, Message> = if self.show_status_bar {
            let (line, col) = self.cursor_position();
            container(
                row![
                    Space::new().width(Fill),
                    container(text(format!("Ln {}, Col {}", line, col)).size(12),)
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

        let below_menu: Element<'_, Message> = if let Some(position) = self.context_menu_position {
            let context_menu_state = ContextMenuState {
                has_undo: self.undo_snapshot.is_some(),
                has_selection: self.content.selection().is_some(),
            };
            let context_menu = menu::view_context_menu(&context_menu_state);

            stack![
                column![editor_with_border, status_bar],
                mouse_area(container(Space::new()).width(Fill).height(Fill))
                    .on_press(Message::CloseMenus),
                container(opaque(context_menu)).padding(Padding {
                    top: position.y,
                    right: 0.0,
                    bottom: 0.0,
                    left: position.x,
                })
            ]
            .into()
        } else {
            column![editor_with_border, status_bar].into()
        };

        // Layer dropdown menu if active — menu_bar stays ABOVE the overlay
        // so hover-to-switch between menu buttons keeps working.
        let with_menu = if let Some(menu_id) = self.active_menu {
            let menu_state = MenuState {
                has_undo: self.undo_snapshot.is_some(),
                has_selection: self.content.selection().is_some(),
                word_wrap: self.word_wrap,
                dark_mode: self.dark_mode,
            };
            let dropdown = menu::view_dropdown(menu_id, &menu_state);

            let x_offset = menu::menu_x_offset(menu_id);

            column![
                menu_bar,
                stack![
                    below_menu,
                    // Full-area click catcher (closes menu on click outside)
                    mouse_area(container(Space::new()).width(Fill).height(Fill))
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
                mouse_area(container(opaque(center(alert))).width(Fill).height(Fill),)
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
                text(title).size(12).color(iced::Color::WHITE).width(Fill),
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
                                radio(
                                    "Up",
                                    FindDirection::Up,
                                    Some(self.find_direction),
                                    Message::FindDirectionChanged
                                )
                                .size(14)
                                .text_size(13),
                                radio(
                                    "Down",
                                    FindDirection::Down,
                                    Some(self.find_direction),
                                    Message::FindDirectionChanged
                                )
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
        .padding(0)
        .width(Length::Fixed(420.0));

        mouse_area(opaque(content))
            .on_press(Message::Ignored)
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
        .padding(0)
        .width(Length::Fixed(420.0));

        mouse_area(opaque(content))
            .on_press(Message::Ignored)
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
        .padding(0)
        .width(Length::Fixed(260.0));

        mouse_area(opaque(content))
            .on_press(Message::Ignored)
            .into()
    }

    fn view_about_dialog(&self) -> Element<'_, Message> {
        let content = container(
            column![
                Self::dialog_title_bar("About RustPad"),
                column![
                    text("RustPad")
                        .size(20)
                        .width(Fill)
                        .align_x(iced::alignment::Horizontal::Center),
                    text("A Windows 98 Notepad clone")
                        .size(13)
                        .width(Fill)
                        .align_x(iced::alignment::Horizontal::Center),
                    text("Built with Rust and Iced")
                        .size(13)
                        .width(Fill)
                        .align_x(iced::alignment::Horizontal::Center),
                    text(format!("Version {}", env!("CARGO_PKG_VERSION")))
                        .size(12)
                        .width(Fill)
                        .align_x(iced::alignment::Horizontal::Center),
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
        .padding(0)
        .width(Length::Fixed(280.0));

        mouse_area(opaque(content))
            .on_press(Message::Ignored)
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
                    text(format!("Do you want to save changes to {filename}?")).size(14),
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
        .padding(0)
        .width(Length::Fixed(360.0));

        mouse_area(opaque(content))
            .on_press(Message::Ignored)
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
            container(scrollable(column(font_items).spacing(0).width(Fill)).height(100),)
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
            container(scrollable(column(style_items).spacing(0).width(Fill)).height(100),)
                .style(t::win98_sunken_container_style)
                .width(Fill),
        ]
        .spacing(2)
        .width(120);

        // Size scrollable list
        let sizes = [
            "8", "9", "10", "11", "12", "14", "16", "18", "20", "22", "24", "26", "28", "36", "48",
            "72",
        ];
        let size_items: Vec<Element<'_, Message>> = sizes
            .iter()
            .filter(|s| {
                self.font_dialog_size_text.is_empty() || s.starts_with(&self.font_dialog_size_text)
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
            container(scrollable(column(size_items).spacing(0).width(Fill)).height(100),)
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
                        container(text("AaBbYyZz").font(preview_font).size(preview_size),)
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
        .padding(0)
        .width(Length::Fixed(420.0));

        mouse_area(opaque(content))
            .on_press(Message::Ignored)
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
        .padding(0)
        .width(Length::Fixed(320.0));

        mouse_area(opaque(content))
            .on_press(Message::Ignored)
            .into()
    }

    // -- Helpers --

    fn close_menus(&mut self) {
        self.active_menu = None;
        self.context_menu_position = None;
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
            PendingAction::Open => Task::perform(file_ops::open_file(), Message::FileOpened),
            PendingAction::Exit => iced::exit(),
        }
    }

    fn cursor_position(&self) -> (usize, usize) {
        let cursor = self.content.cursor();
        (cursor.position.line + 1, cursor.position.column + 1)
    }

    fn capture_undo_snapshot(&mut self) {
        if self.undo_snapshot.is_none() || !self.is_dirty {
            self.undo_snapshot = Some(self.content.text());
        }
    }

    fn show_alert(&mut self, message: String) {
        self.alert_message = Some(message);
    }

    fn persist_settings(&self) -> Result<(), settings::SettingsError> {
        settings::save(&settings::Settings {
            dark_mode: self.dark_mode,
        })
    }

    fn do_find_next(&mut self) {
        if self.find_text.is_empty() {
            return;
        }

        let full_text = self.content.text();
        if let Some((start, len)) = self.find_match(
            &full_text,
            cursor_char_offset(&self.content, &full_text),
            true,
            true,
        ) {
            self.select_range(start, len, &full_text);
        } else {
            self.show_alert(format!("Cannot find \"{}\"", self.find_text));
        }
    }

    fn do_find_previous(&mut self) {
        if self.find_text.is_empty() {
            return;
        }

        let full_text = self.content.text();
        if let Some((start, len)) = self.find_match(
            &full_text,
            cursor_char_offset(&self.content, &full_text),
            false,
            true,
        ) {
            self.select_range(start, len, &full_text);
        } else {
            self.show_alert(format!("Cannot find \"{}\"", self.find_text));
        }
    }

    fn do_replace(&mut self) {
        if let Some(selection) = self.content.selection() {
            let full_text = self.content.text();
            let selection_matches =
                strings_match_case(&selection, &self.find_text, self.find_case_sensitive);
            let is_whole_word_selection = if self.find_whole_word {
                self.selection_matches_whole_word(&full_text)
            } else {
                true
            };

            if selection_matches && is_whole_word_selection {
                self.capture_undo_snapshot();
                self.content
                    .perform(text_editor::Action::Edit(text_editor::Edit::Paste(
                        Arc::new(self.replace_text.clone()),
                    )));
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
        let boundaries = char_boundaries(&full_text);
        let mut replacements = Vec::new();
        let mut cursor = 0;

        while let Some((start, len)) = self.find_match(&full_text, cursor, true, false) {
            replacements.push((start, len));
            cursor = start + len.max(1);
        }

        if replacements.is_empty() {
            self.show_alert(format!("Cannot find \"{}\"", self.find_text));
            return;
        }

        let mut rebuilt = String::new();
        let mut current_char = 0;

        for (start, len) in replacements {
            rebuilt.push_str(&full_text[boundaries[current_char]..boundaries[start]]);
            rebuilt.push_str(&self.replace_text);
            current_char = start + len;
        }

        rebuilt.push_str(&full_text[boundaries[current_char]..]);

        self.undo_snapshot = Some(full_text);
        self.content = text_editor::Content::with_text(&rebuilt);
        self.is_dirty = true;
    }

    fn do_goto_line(&mut self) -> bool {
        let Ok(line_num) = self.goto_line_text.trim().parse::<usize>() else {
            self.show_alert("Enter a valid line number.".to_owned());
            return false;
        };

        if line_num == 0 {
            self.show_alert("Line numbers start at 1.".to_owned());
            return false;
        }

        let total_lines = line_count(&self.content.text());
        if line_num > total_lines {
            self.show_alert(format!("The document only has {total_lines} line(s)."));
            return false;
        }

        let target_line = line_num - 1;

        self.content.perform(text_editor::Action::Move(
            text_editor::Motion::DocumentStart,
        ));
        for _ in 0..target_line {
            self.content
                .perform(text_editor::Action::Move(text_editor::Motion::Down));
        }
        self.content
            .perform(text_editor::Action::Move(text_editor::Motion::Home));

        true
    }

    fn select_range(&mut self, start_char: usize, len_chars: usize, full_text: &str) {
        let (target_line, target_col) = line_col_for_char_index(full_text, start_char);

        self.content.perform(text_editor::Action::Move(
            text_editor::Motion::DocumentStart,
        ));
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

        for _ in 0..len_chars {
            self.content
                .perform(text_editor::Action::Select(text_editor::Motion::Right));
        }
    }

    fn find_match(
        &self,
        full_text: &str,
        start_char: usize,
        forward: bool,
        wrap: bool,
    ) -> Option<(usize, usize)> {
        if self.find_text.is_empty() {
            return None;
        }

        if self.find_case_sensitive {
            let chars: Vec<char> = full_text.chars().collect();
            let boundaries = char_boundaries(full_text);

            if forward {
                find_case_sensitive_forward(
                    full_text,
                    &self.find_text,
                    &chars,
                    &boundaries,
                    start_char,
                    self.find_whole_word,
                )
                .or_else(|| {
                    if wrap {
                        find_case_sensitive_forward(
                            full_text,
                            &self.find_text,
                            &chars,
                            &boundaries,
                            0,
                            self.find_whole_word,
                        )
                    } else {
                        None
                    }
                })
            } else {
                find_case_sensitive_backward(
                    full_text,
                    &self.find_text,
                    &chars,
                    &boundaries,
                    start_char,
                    self.find_whole_word,
                )
                .or_else(|| {
                    if wrap {
                        find_case_sensitive_backward(
                            full_text,
                            &self.find_text,
                            &chars,
                            &boundaries,
                            chars.len(),
                            self.find_whole_word,
                        )
                    } else {
                        None
                    }
                })
            }
        } else {
            let chars: Vec<char> = full_text.chars().collect();
            let folded = FoldedText::new(full_text);
            let folded_needle = fold_case(&self.find_text);

            if forward {
                find_folded_forward(
                    &folded,
                    &folded_needle,
                    &chars,
                    start_char,
                    self.find_whole_word,
                )
                .or_else(|| {
                    if wrap {
                        find_folded_forward(
                            &folded,
                            &folded_needle,
                            &chars,
                            0,
                            self.find_whole_word,
                        )
                    } else {
                        None
                    }
                })
            } else {
                find_folded_backward(
                    &folded,
                    &folded_needle,
                    &chars,
                    start_char,
                    self.find_whole_word,
                )
                .or_else(|| {
                    if wrap {
                        find_folded_backward(
                            &folded,
                            &folded_needle,
                            &chars,
                            chars.len(),
                            self.find_whole_word,
                        )
                    } else {
                        None
                    }
                })
            }
        }
    }

    fn selection_matches_whole_word(&self, full_text: &str) -> bool {
        let Some(selection) = self.content.selection() else {
            return false;
        };

        if selection.is_empty() {
            return false;
        }

        let cursor_offset = cursor_char_offset(&self.content, full_text);
        let selection_len = selection.chars().count();
        let start = cursor_offset.saturating_sub(selection_len);
        let chars: Vec<char> = full_text.chars().collect();

        is_whole_word_match(&chars, start, selection_len)
    }
}

struct FoldedText {
    text: String,
    boundaries: Vec<usize>,
}

impl FoldedText {
    fn new(source: &str) -> Self {
        let mut text = String::new();
        let mut boundaries = Vec::with_capacity(source.chars().count() + 1);
        boundaries.push(0);

        for ch in source.chars() {
            text.extend(ch.to_lowercase());
            boundaries.push(text.len());
        }

        Self { text, boundaries }
    }
}

fn cursor_char_offset(content: &text_editor::Content, full_text: &str) -> usize {
    let cursor = content.cursor();
    let mut offset = 0;

    for (line_index, line) in full_text.split('\n').enumerate() {
        if line_index == cursor.position.line {
            return offset + cursor.position.column.min(line.chars().count());
        }

        offset += line.chars().count() + 1;
    }

    full_text.chars().count()
}

fn line_count(text: &str) -> usize {
    text.split('\n').count().max(1)
}

fn line_col_for_char_index(text: &str, target: usize) -> (usize, usize) {
    let mut line = 0;
    let mut col = 0;

    for (index, ch) in text.chars().enumerate() {
        if index == target {
            break;
        }

        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    (line, col)
}

fn char_boundaries(text: &str) -> Vec<usize> {
    let mut boundaries = text.char_indices().map(|(idx, _)| idx).collect::<Vec<_>>();
    boundaries.push(text.len());
    boundaries
}

fn fold_case(text: &str) -> String {
    text.chars().flat_map(|ch| ch.to_lowercase()).collect()
}

fn strings_match_case(left: &str, right: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        left == right
    } else {
        fold_case(left) == fold_case(right)
    }
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn is_whole_word_match(chars: &[char], start: usize, len: usize) -> bool {
    let before_ok = start == 0 || !is_word_char(chars[start - 1]);
    let after_index = start + len;
    let after_ok = after_index >= chars.len() || !is_word_char(chars[after_index]);
    before_ok && after_ok
}

fn next_boundary_after(boundaries: &[usize], value: usize) -> Option<usize> {
    boundaries
        .iter()
        .copied()
        .find(|boundary| *boundary > value)
}

fn find_case_sensitive_forward(
    text: &str,
    needle: &str,
    chars: &[char],
    boundaries: &[usize],
    start_char: usize,
    whole_word: bool,
) -> Option<(usize, usize)> {
    let mut search_from = boundaries[start_char.min(chars.len())];

    while search_from <= text.len() {
        let found = text[search_from..].find(needle)?;
        let start_byte = search_from + found;
        let end_byte = start_byte + needle.len();

        let Ok(start) = boundaries.binary_search(&start_byte) else {
            search_from = next_boundary_after(boundaries, start_byte)?;
            continue;
        };
        let Ok(end) = boundaries.binary_search(&end_byte) else {
            search_from = next_boundary_after(boundaries, start_byte)?;
            continue;
        };

        let len = end - start;
        if !whole_word || is_whole_word_match(chars, start, len) {
            return Some((start, len));
        }

        search_from = boundaries[(start + 1).min(chars.len())];
    }

    None
}

fn find_case_sensitive_backward(
    text: &str,
    needle: &str,
    chars: &[char],
    boundaries: &[usize],
    end_char: usize,
    whole_word: bool,
) -> Option<(usize, usize)> {
    let search_end = boundaries[end_char.min(chars.len())];
    let mut last_valid = None;

    for (start_byte, _) in text[..search_end].match_indices(needle) {
        let end_byte = start_byte + needle.len();
        let Ok(start) = boundaries.binary_search(&start_byte) else {
            continue;
        };
        let Ok(end) = boundaries.binary_search(&end_byte) else {
            continue;
        };

        let len = end - start;
        if !whole_word || is_whole_word_match(chars, start, len) {
            last_valid = Some((start, len));
        }
    }

    last_valid
}

fn find_folded_forward(
    folded: &FoldedText,
    needle: &str,
    chars: &[char],
    start_char: usize,
    whole_word: bool,
) -> Option<(usize, usize)> {
    let mut search_from = folded.boundaries[start_char.min(chars.len())];

    while search_from <= folded.text.len() {
        let found = folded.text[search_from..].find(needle)?;
        let start_byte = search_from + found;
        let end_byte = start_byte + needle.len();

        let Ok(start) = folded.boundaries.binary_search(&start_byte) else {
            search_from = next_boundary_after(&folded.boundaries, start_byte)?;
            continue;
        };
        let Ok(end) = folded.boundaries.binary_search(&end_byte) else {
            search_from = next_boundary_after(&folded.boundaries, start_byte)?;
            continue;
        };

        let len = end - start;
        if !whole_word || is_whole_word_match(chars, start, len) {
            return Some((start, len));
        }

        search_from = folded.boundaries[(start + 1).min(chars.len())];
    }

    None
}

fn find_folded_backward(
    folded: &FoldedText,
    needle: &str,
    chars: &[char],
    end_char: usize,
    whole_word: bool,
) -> Option<(usize, usize)> {
    let search_end = folded.boundaries[end_char.min(chars.len())];
    let mut last_valid = None;

    for (start_byte, _) in folded.text[..search_end].match_indices(needle) {
        let end_byte = start_byte + needle.len();
        let Ok(start) = folded.boundaries.binary_search(&start_byte) else {
            continue;
        };
        let Ok(end) = folded.boundaries.binary_search(&end_byte) else {
            continue;
        };

        let len = end - start;
        if !whole_word || is_whole_word_match(chars, start, len) {
            last_valid = Some((start, len));
        }
    }

    last_valid
}

#[cfg(test)]
mod tests {
    use super::{
        FoldedText, char_boundaries, find_case_sensitive_forward, find_folded_forward, fold_case,
        is_whole_word_match, strings_match_case,
    };

    #[test]
    fn folded_search_respects_unicode_boundaries() {
        let text = "Before İ after";
        let chars: Vec<char> = text.chars().collect();
        let folded = FoldedText::new(text);
        let needle = fold_case("İ");

        let found = find_folded_forward(&folded, &needle, &chars, 0, false);

        assert_eq!(found, Some((7, 1)));
    }

    #[test]
    fn case_sensitive_search_uses_character_indices() {
        let text = "aébc";
        let chars: Vec<char> = text.chars().collect();
        let boundaries = char_boundaries(text);

        let found = find_case_sensitive_forward(text, "éb", &chars, &boundaries, 0, false);

        assert_eq!(found, Some((1, 2)));
    }

    #[test]
    fn whole_word_check_respects_word_boundaries() {
        let chars: Vec<char> = "foo bar_baz".chars().collect();

        assert!(is_whole_word_match(&chars, 0, 3));
        assert!(!is_whole_word_match(&chars, 4, 3));
    }

    #[test]
    fn folded_string_comparison_matches_unicode_case() {
        assert!(strings_match_case("İ", "i\u{307}", false));
        assert!(!strings_match_case("Rust", "Dust", false));
    }
}
