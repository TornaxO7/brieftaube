use super::Action;
use crate::{
    backend::{
        Backend,
        mails::types::{MailBodyType, MailId, MailKeyword, MailUpdate},
    },
    mail_viewer::{types::FullMailDisplay, widget::RenderData},
    utils::ui::{
        ScreenOverlay, ScreenOverlayResult, ScreenState, keybindmanager::KeybindManager, palette,
    },
};
use ratatui::widgets::ScrollbarState;
use std::{collections::HashMap, rc::Rc};
use tracing::{debug, error, warn};

#[derive(Debug, Clone)]
pub enum PaletteType {
    /// Palette is displaying commands
    Action(Action),
}

#[derive(Debug, Clone)]
pub enum InputType {}

#[derive(Debug, Clone, Copy)]
pub enum ViewVariant {
    Text,
    Markdown,
    Attachments,
}

#[derive(Debug, Clone, Copy)]
pub enum ScrollAction {
    ScrollDown(usize),
    ScrollUp(usize),
    ScrollHalfPageDown,
    ScrollHalfPageUp,
    ScrollHalfPageRight,
    ScrollHalfPageLeft,
    SetTop,
    SetBottom,
}

pub struct State {
    app_actions: Vec<crate::Action>,
    overlay: Option<ScreenOverlay<PaletteType, InputType>>,
    keybindings: KeybindManager<Action>,

    id: MailId,
    backend: Rc<Backend>,
    variant: ViewVariant,
    vertical: ScrollbarState,
    horizontal: ScrollbarState,
    scroll_action: Option<ScrollAction>,
}

impl State {
    pub fn new(id: MailId, backend: Rc<Backend>) -> Self {
        let variant = match backend.config().mail_viewer.default_tab {
            crate::config::DefaultTab::Headers => {
                todo!()
            }
            crate::config::DefaultTab::Attachments => ViewVariant::Attachments,
            crate::config::DefaultTab::Text => {
                backend.mail_request_body_type(&id, MailBodyType::Text);
                ViewVariant::Text
            }
            crate::config::DefaultTab::Markdown => {
                backend.mail_request_body_type(&id, MailBodyType::Html);
                ViewVariant::Markdown
            }
        };

        backend.mails_update(vec![MailUpdate {
            id: id.clone(),
            patch_keywords: Some(vec![(MailKeyword::Seen, true)]),
            ..Default::default()
        }]);

        Self {
            id,
            backend,
            app_actions: vec![],
            scroll_action: None,
            overlay: None,
            keybindings: KeybindManager::new(HashMap::from([
                ("j", Action::ScrollDown),
                ("k", Action::ScrollUp),
                ("h", Action::Back),
                ("<C-d>", Action::ScrollHalfPageDown),
                ("<C-u>", Action::ScrollHalfPageUp),
                ("zH", Action::ScrollHalfPageLeft),
                ("zL", Action::ScrollHalfPageRight),
                ("zh", Action::ScrollLeft),
                ("zl", Action::ScrollRight),
                ("q", Action::Quit),
                (":", Action::OpenCommandPalette),
                ("gg", Action::ScrollToTop),
                ("ge", Action::ScrollToBottom),
                ("<BS>", Action::Back),
                ("<C-l>", Action::OpenLogs),
            ])),
            variant,
            vertical: ScrollbarState::default(),
            horizontal: ScrollbarState::default(),
        }
    }
}

impl<'a> ScreenState<'a, Action, PaletteType, InputType, RenderData<'a>> for State {
    fn apply_action(&mut self, action: Action) {
        debug!("Action: {}", action);
        match action {
            Action::Quit => self.app_actions.push(crate::Action::Quit),
            Action::OpenCommandPalette => {
                self.overlay = Some(ScreenOverlay::Palette(palette::State::new(
                    super::action::palette_options(),
                )));
            }

            Action::ScrollUp => self.scroll_up(),
            Action::ScrollDown => self.scroll_down(),
            Action::ScrollLeft => self.scroll_left(),
            Action::ScrollRight => self.scroll_right(),
            Action::ScrollToTop => self.scroll_to_top(),
            Action::ScrollToBottom => self.scroll_to_bottom(),
            Action::ScrollHalfPageDown => self.scroll_half_page_down(),
            Action::ScrollHalfPageUp => self.scroll_half_page_up(),
            Action::ScrollHalfPageLeft => self.scroll_half_page_left(),
            Action::ScrollHalfPageRight => self.scroll_half_page_right(),

            Action::OpenTextTab => self.set_variant(ViewVariant::Text),
            Action::OpenMarkdownTab => self.set_variant(ViewVariant::Markdown),
            Action::OpenLogs => self.app_actions.push(crate::Action::OpenLogViewer),
            Action::OpenMailInBrowser => self.open_html_mail_in_browser(),

            Action::Back => {
                self.app_actions.push(crate::Action::Back);
            }
        }
    }

    fn get_app_actions(&mut self) -> std::vec::Drain<'_, crate::Action> {
        self.app_actions.drain(..)
    }

    fn keybinding_manager(&mut self) -> &mut KeybindManager<Action> {
        &mut self.keybindings
    }

    fn overlay(&mut self) -> Option<&mut ScreenOverlay<PaletteType, InputType>> {
        self.overlay.as_mut()
    }

    fn handle_overlay_result(&mut self, result: ScreenOverlayResult<PaletteType, InputType>) {
        self.overlay = None;

        match result {
            ScreenOverlayResult::Cancel => {}
            ScreenOverlayResult::Palette(value) => match value {
                PaletteType::Action(action) => self.apply_action(action),
            },
            ScreenOverlayResult::Input { value: _, typ: _ } => unreachable!(),
        }
    }

    fn render_data(&'a mut self) -> RenderData<'a> {
        let mail = self.backend.mail_get_data(&self.id).unwrap();

        match self.variant {
            ViewVariant::Text => {
                if mail.rest.text_body.is_none() {
                    self.backend
                        .mail_request_body_type(&mail.id, MailBodyType::Text);
                }
            }
            ViewVariant::Markdown => {
                if mail.rest.html_body.is_none() {
                    self.backend
                        .mail_request_body_type(&mail.id, MailBodyType::Html);
                }
            }
            ViewVariant::Attachments => {
                todo!()
            }
        }

        RenderData {
            variant: self.variant,
            mail: FullMailDisplay::from(&mail),
            horizontal: &mut self.horizontal,
            vertical: &mut self.vertical,
            scroll_queue: &mut self.scroll_action,
        }
    }
}

impl State {
    fn scroll_down(&mut self) {
        let action = match self.keybindings.flush_int_prefix() {
            Some(num) => ScrollAction::ScrollDown(num),
            None => ScrollAction::ScrollDown(1),
        };

        self.scroll_action = Some(action);
    }

    fn scroll_up(&mut self) {
        let action = match self.keybindings.flush_int_prefix() {
            Some(num) => ScrollAction::ScrollUp(num),
            None => ScrollAction::ScrollUp(1),
        };

        self.scroll_action = Some(action);
    }

    fn scroll_left(&mut self) {
        match self.variant {
            ViewVariant::Text | ViewVariant::Markdown => self.horizontal.prev(),
            ViewVariant::Attachments => todo!(),
        }
    }

    fn scroll_right(&mut self) {
        match self.variant {
            ViewVariant::Text | ViewVariant::Markdown => self.horizontal.next(),
            ViewVariant::Attachments => todo!(),
        }
    }

    fn scroll_to_top(&mut self) {
        self.scroll_action = Some(ScrollAction::SetTop);
    }

    fn scroll_to_bottom(&mut self) {
        self.scroll_action = Some(ScrollAction::SetBottom);
    }

    fn scroll_half_page_up(&mut self) {
        self.scroll_action = Some(ScrollAction::ScrollHalfPageUp);
    }

    fn scroll_half_page_down(&mut self) {
        self.scroll_action = Some(ScrollAction::ScrollHalfPageDown);
    }

    fn scroll_half_page_left(&mut self) {
        self.scroll_action = Some(ScrollAction::ScrollHalfPageLeft);
    }

    fn scroll_half_page_right(&mut self) {
        self.scroll_action = Some(ScrollAction::ScrollHalfPageRight);
    }

    fn set_variant(&mut self, variant: ViewVariant) {
        self.variant = variant;
    }

    fn open_html_mail_in_browser(&self) {
        todo!()
    }
}
