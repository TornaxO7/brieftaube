use crate::Screen;

pub struct State {
    pub(super) screen_name: &'static str,
}

impl State {
    pub fn new(init_screen: &Screen) -> Self {
        Self {
            screen_name: get_screen_name(init_screen),
        }
    }

    pub fn set_screen(&mut self, screen: &Screen) {
        self.screen_name = get_screen_name(screen);
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
