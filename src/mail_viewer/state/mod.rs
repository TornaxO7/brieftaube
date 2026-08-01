mod attachment_viewer;
mod headers_viewer;
mod markdown_viewer;
mod text_viewer;

use super::Action;
use crate::{
    backend::{
        Backend,
        mails::types::{MailBodyType, MailId, MailKeyword, MailUpdate},
    },
    mail_viewer::{
        types::FullMailDisplay,
        widget::{RenderData, ViewerState},
    },
    utils::ui::{
        ScreenOverlay, ScreenOverlayResult, ScreenState, keybindmanager::KeybindManager, palette,
    },
};
pub use attachment_viewer::AttachmentViewer;
pub use headers_viewer::HeadersViewer;
pub use markdown_viewer::MarkdownViewer;
use std::{collections::HashMap, rc::Rc};
pub use text_viewer::TextViewer;
use tracing::debug;

#[derive(Debug, Clone)]
pub enum PaletteType {
    /// Palette is displaying commands
    Action(Action),
}

#[derive(Debug, Clone)]
pub enum InputType {}

#[derive(Debug, Clone, Copy)]
pub enum Viewer {
    Headers,
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
    ScrollLeft(usize),
    ScrollRight(usize),
    SetTop,
    SetBottom,
}

pub struct State {
    app_actions: Vec<crate::Action>,
    overlay: Option<ScreenOverlay<PaletteType, InputType>>,
    keybindings: KeybindManager<Action>,

    id: MailId,
    backend: Rc<Backend>,

    selected_viewer: Viewer,
    headers_viewer: HeadersViewer,
    text_viewer: TextViewer,
    markdown_viewer: MarkdownViewer,
    attachment_viewer: AttachmentViewer,

    /// Contains the scrolling action for the current, selected viewer.
    /// Since we don't know the height and width of the area where each viewer
    /// gets rendered to, we have to apply the scroll action _later_ during the rendering...
    scroll_action: Option<ScrollAction>,
}

impl State {
    pub fn new(id: MailId, backend: Rc<Backend>) -> Self {
        let selected_viewer = match backend.config().mail_viewer.default_tab {
            crate::config::DefaultTab::Headers => Viewer::Headers,
            crate::config::DefaultTab::Attachments => Viewer::Attachments,
            crate::config::DefaultTab::Text => {
                backend.mail_request_body_type(&id, MailBodyType::Text);
                Viewer::Text
            }
            crate::config::DefaultTab::Markdown => {
                backend.mail_request_body_type(&id, MailBodyType::Html);
                Viewer::Markdown
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
            selected_viewer,

            headers_viewer: HeadersViewer::default(),
            text_viewer: TextViewer::default(),
            markdown_viewer: MarkdownViewer::default(),
            attachment_viewer: AttachmentViewer::default(),
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

            Action::OpenTextTab => self.set_viewer(Viewer::Text),
            Action::OpenMarkdownTab => self.set_viewer(Viewer::Markdown),
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
        tracing::debug!("Selected viewer: {:?}", self.selected_viewer);
        let viewer_state = match self.selected_viewer {
            Viewer::Headers => ViewerState::from(&mut self.headers_viewer),
            Viewer::Text => {
                self.backend
                    .mail_request_body_type(&self.id, MailBodyType::Text);

                ViewerState::from(&mut self.text_viewer)
            }
            Viewer::Markdown => {
                self.backend
                    .mail_request_body_type(&self.id, MailBodyType::Html);

                ViewerState::from(&mut self.markdown_viewer)
            }
            Viewer::Attachments => {
                self.backend.mail_request_attachments(&self.id);
                ViewerState::from(&mut self.attachment_viewer)
            }
        };

        let mail = self.backend.mail_get_data(&self.id).unwrap();

        RenderData {
            viewer_state,
            mail: FullMailDisplay::from(mail),
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
        let action = match self.keybindings.flush_int_prefix() {
            Some(num) => ScrollAction::ScrollLeft(num),
            None => ScrollAction::ScrollLeft(1),
        };

        self.scroll_action = Some(action);
    }

    fn scroll_right(&mut self) {
        let action = match self.keybindings.flush_int_prefix() {
            Some(num) => ScrollAction::ScrollRight(num),
            None => ScrollAction::ScrollRight(1),
        };

        self.scroll_action = Some(action);
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

    fn set_viewer(&mut self, variant: Viewer) {
        self.selected_viewer = variant;
    }

    fn open_html_mail_in_browser(&self) {
        todo!()
    }
}
