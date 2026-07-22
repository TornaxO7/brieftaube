use super::Action;
use crate::{
    backend::mails::{MailsBackend, types::MailId},
    mail_viewer::{types::FullMailDisplay, widget::RenderData},
    utils::ui::{
        ScreenOverlay, ScreenOverlayResult, ScreenState, keybindmanager::KeybindManager, palette,
    },
};
use ratatui::widgets::ScrollbarState;
use std::{collections::HashMap, rc::Rc};
use tracing::debug;

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
    ScrollDown,
    ScrollUp,
    ScrollHalfPageDown,
    ScrollHalfPageUp,
    SetTop,
    SetBottom,
}

pub struct State {
    app_actions: Vec<crate::Action>,
    overlay: Option<ScreenOverlay<PaletteType, InputType>>,
    keybindings: KeybindManager<Action>,

    id: MailId,
    backend: Rc<MailsBackend>,
    variant: ViewVariant,
    vertical: ScrollbarState,
    horizontal: ScrollbarState,
    scroll_action: Option<ScrollAction>,
}

impl State {
    pub fn new(id: MailId, backend: Rc<MailsBackend>) -> Self {
        backend.request_get_mails_rest(vec![id.clone()]);

        Self {
            id,
            backend,
            app_actions: vec![],
            scroll_action: None,
            overlay: None,
            keybindings: KeybindManager::new(HashMap::from([
                ("h", Action::ScrollLeft),
                ("j", Action::ScrollDown),
                ("k", Action::ScrollUp),
                ("l", Action::ScrollRight),
                ("<C-d>", Action::ScrollHalfPageDown),
                ("<C-u>", Action::ScrollHalfPageUp),
                ("q", Action::Quit),
                (":", Action::OpenCommandPalette),
                ("gg", Action::ScrollToTop),
                ("ge", Action::ScrollToBottom),
                ("<BS>", Action::Back),
                ("<C-l>", Action::OpenLogs),
            ])),
            variant: ViewVariant::Markdown,
            vertical: ScrollbarState::default(),
            horizontal: ScrollbarState::default(),
        }
    }
}

impl ScreenState<Action, PaletteType, InputType> for State {
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

            Action::OpenTextTab => self.set_variant(ViewVariant::Text),
            Action::OpenMarkdownTab => self.set_variant(ViewVariant::Markdown),
            Action::OpenLogs => self.app_actions.push(crate::Action::OpenLogViewer),

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
}

impl State {
    fn scroll_down(&mut self) {
        match self.variant {
            ViewVariant::Text | ViewVariant::Markdown => self.vertical.next(),
            ViewVariant::Attachments => todo!(),
        }
    }

    fn scroll_up(&mut self) {
        self.scroll_action = Some(ScrollAction::ScrollUp);
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

    fn set_variant(&mut self, variant: ViewVariant) {
        self.variant = variant;
    }
}

// for `widget`
impl State {
    pub fn get_render_data<'a>(&'a mut self) -> Option<RenderData<'a>> {
        let mail = self.backend.get_mail(&self.id)?;

        if mail.rest.is_none() {
            self.backend.request_get_mails_rest(vec![mail.id]);
            return None;
        }

        Some(RenderData {
            variant: self.variant,
            mail: FullMailDisplay::from(&mail),
            horizontal: &mut self.horizontal,
            vertical: &mut self.vertical,
            scroll_queue: &mut self.scroll_action,
        })
    }
}
