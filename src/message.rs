use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use iced::font;
use iced::widget::text_editor;

use crate::file_ops::FileError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontChoice {
    FixedSys,
    Courier,
    CourierNew,
    Consolas,
    LucidaConsole,
    Monospace,
    SansSerif,
    Serif,
}

impl FontChoice {
    pub const ALL: &[FontChoice] = &[
        FontChoice::FixedSys,
        FontChoice::Courier,
        FontChoice::CourierNew,
        FontChoice::Consolas,
        FontChoice::LucidaConsole,
        FontChoice::Monospace,
        FontChoice::SansSerif,
        FontChoice::Serif,
    ];

    pub fn to_iced_family(self) -> font::Family {
        match self {
            FontChoice::FixedSys => font::Family::Name("Fixedsys"),
            FontChoice::Courier => font::Family::Name("Courier"),
            FontChoice::CourierNew => font::Family::Name("Courier New"),
            FontChoice::Consolas => font::Family::Name("Consolas"),
            FontChoice::LucidaConsole => font::Family::Name("Lucida Console"),
            FontChoice::Monospace => font::Family::Monospace,
            FontChoice::SansSerif => font::Family::SansSerif,
            FontChoice::Serif => font::Family::Serif,
        }
    }
}

impl fmt::Display for FontChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FontChoice::FixedSys => write!(f, "FixedSys"),
            FontChoice::Courier => write!(f, "Courier"),
            FontChoice::CourierNew => write!(f, "Courier New"),
            FontChoice::Consolas => write!(f, "Consolas"),
            FontChoice::LucidaConsole => write!(f, "Lucida Console"),
            FontChoice::Monospace => write!(f, "Monospace"),
            FontChoice::SansSerif => write!(f, "Sans Serif"),
            FontChoice::Serif => write!(f, "Serif"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyleChoice {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}

impl FontStyleChoice {
    pub const ALL: &[FontStyleChoice] = &[
        FontStyleChoice::Regular,
        FontStyleChoice::Bold,
        FontStyleChoice::Italic,
        FontStyleChoice::BoldItalic,
    ];
}

impl fmt::Display for FontStyleChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FontStyleChoice::Regular => write!(f, "Regular"),
            FontStyleChoice::Bold => write!(f, "Bold"),
            FontStyleChoice::Italic => write!(f, "Italic"),
            FontStyleChoice::BoldItalic => write!(f, "Bold Italic"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    // Text editor
    EditorAction(text_editor::Action),

    // File operations
    NewFile,
    OpenFile,
    FileOpened(Result<(PathBuf, Arc<String>), FileError>),
    SaveFile,
    SaveFileAs,
    FileSaved(Result<PathBuf, FileError>),
    Exit,

    // Edit operations
    Undo,
    Cut,
    Copy,
    Paste,
    Delete,
    SelectAll,
    InsertTimeDate,
    ToggleWordWrap,

    // Search operations
    OpenFindDialog,
    OpenReplaceDialog,
    OpenGoToDialog,
    FindNext,
    DoReplace,
    DoReplaceAll,
    GoToLine,

    // Font dialog
    OpenFontDialog,
    FontFamilyChanged(FontChoice),
    FontFamilyFilterChanged(String),
    FontStyleChanged(FontStyleChoice),
    FontStyleFilterChanged(String),
    FontSizeChanged(String),
    FontSizeSelected(String),
    ApplyFont,

    // Dialog field updates
    FindTextChanged(String),
    ReplaceTextChanged(String),
    GoToLineChanged(String),
    ToggleCaseSensitive(bool),
    ToggleWholeWord(bool),
    FindDirectionChanged(FindDirection),
    DismissAlert,
    CloseDialog,

    // Menu bar
    MenuClicked(MenuId),
    CloseMenus,

    // Help
    ShowAbout,

    // Save prompt
    SavePromptSave,
    SavePromptDontSave,
    SavePromptCancel,

    // Window events
    CloseRequested,

    // Print
    PageSetup,
    Print,
    PrintResult(Result<(), FileError>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuId {
    File,
    Edit,
    Format,
    Search,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogKind {
    Find,
    Replace,
    GoTo,
    About,
    SavePrompt,
    Font,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingAction {
    New,
    Open,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindDirection {
    Up,
    Down,
}
