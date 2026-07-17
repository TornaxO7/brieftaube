use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Style;

use crate::Screen;

pub struct State {
    pub(super) screen_name: &'static str,
    pub(super) errors: usize,
    pub(super) warnings: usize,
    pub(super) info: usize,

    pub(super) error_style: Style,
    pub(super) warning_style: Style,
    pub(super) info_style: Style,

    pub(super) keypresses: String,
}

impl State {
    pub fn new(init_screen: &Screen) -> Self {
        Self {
            screen_name: get_screen_name(init_screen),
            errors: 0,
            warnings: 0,
            info: 0,

            error_style: Style::default().red(),
            warning_style: Style::default().yellow(),
            info_style: Style::default().green(),

            keypresses: String::new(),
        }
    }

    pub fn set_screen(&mut self, screen: &Screen) {
        self.screen_name = get_screen_name(screen);
    }

    pub fn push_key_press(&mut self, event: KeyEvent) {
        let code = match event.code {
            KeyCode::Char(c) => c,
            _ => '?',
        };

        let s = match event.modifiers {
            KeyModifiers::ALT => format!("<A-{}>", code),
            KeyModifiers::CONTROL => format!("<C-{}>", code),
            _ => code.to_string(),
        };

        self.keypresses.push_str(&s);
    }

    pub fn reset_key_press(&mut self) {
        self.keypresses.clear();
    }
}

fn get_screen_name(screen: &Screen) -> &'static str {
    match screen {
        Screen::Mailboxes(_) => "Mailboxes",
        Screen::MailList(_) => "Mail list",
        Screen::Composer(_) => "Composer",
        Screen::MailViewer(_) => "Mail Viewer",
        Screen::LogViewer(_) => "Log Viewer",
        Screen::ThreadMails(_) => "Thread Viewer",
    }
}
