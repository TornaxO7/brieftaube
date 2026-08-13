mod attachment_viewer;
mod markdown_viewer;
mod metadata_viewer;
mod text_viewer;

use super::Action;
use crate::{
    backend::{
        Backend,
        mails::types::{MailId, MailKeyword, MailUpdate},
    },
    task_manager::TaskManager,
    utils::ui::{
        ScreenOverlay, ScreenOverlayResult, ScreenState, keybindmanager::KeybindManager, palette,
    },
};
pub use attachment_viewer::AttachmentViewer;
pub use markdown_viewer::MarkdownViewer;
pub use metadata_viewer::MetadataViewer;
use std::{collections::HashMap, rc::Rc, sync::Arc};
pub use text_viewer::TextViewer;
use tracing::{debug, error};

#[derive(Debug, Clone)]
pub enum PaletteType {
    /// Palette is displaying commands
    Action(Action),
}

#[derive(Debug, Clone)]
pub enum InputType {}

#[derive(Debug, Clone, Copy)]
pub enum Viewer {
    Metadata,
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

pub struct Model {
    overlay: Option<ScreenOverlay<PaletteType, InputType>>,
    keybindings: KeybindManager<Action>,

    pub id: MailId,
    pub backend: Arc<Backend>,
    pub task_manager: Rc<TaskManager>,

    pub selected_viewer: Viewer,
    pub metadata_viewer: MetadataViewer,
    pub text_viewer: TextViewer,
    pub markdown_viewer: MarkdownViewer,
    pub attachment_viewer: AttachmentViewer,

    /// Contains the scrolling action for the current, selected viewer.
    /// Since we don't know the height and width of the area where each viewer
    /// gets rendered to, we have to apply the scroll action _later_ during the rendering...
    pub scroll_action: Option<ScrollAction>,
}

impl Model {
    pub fn new(id: MailId, backend: Arc<Backend>, task_manager: Rc<TaskManager>) -> Self {
        let selected_viewer = match backend.config().mail_viewer.default_tab {
            crate::config::DefaultTab::Metadata => Viewer::Metadata,
            crate::config::DefaultTab::Attachments => Viewer::Attachments,
            crate::config::DefaultTab::Text => Viewer::Text,
            crate::config::DefaultTab::Markdown => Viewer::Markdown,
        };

        let id2 = id.clone();
        let backend2 = backend.clone();
        task_manager.spawn(async move {
            match backend2
                .update_mails(vec![MailUpdate {
                    id: id2,
                    patch_keywords: Some(vec![(MailKeyword::Seen, true)]),
                    ..Default::default()
                }])
                .await
            {
                Ok(()) => {}
                Err(err) => {
                    error!("Couldn't mark mail as \"seen\":\n{err}");
                }
            }
        });

        Self {
            id,
            backend,
            task_manager,
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

            metadata_viewer: MetadataViewer::default(),
            text_viewer: TextViewer::default(),
            markdown_viewer: MarkdownViewer::default(),
            attachment_viewer: AttachmentViewer::default(),
        }
    }
}

impl<'a> ScreenState<'a, Action, PaletteType, InputType> for Model {
    fn apply_user_action(&mut self, action: Action) -> Option<crate::Action> {
        debug!("Action: {}", action);
        match action {
            Action::Quit => return Some(crate::Action::Quit),
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

            Action::OpenMetadataTab => self.set_viewer(Viewer::Metadata),
            Action::OpenTextTab => self.set_viewer(Viewer::Text),
            Action::OpenMarkdownTab => self.set_viewer(Viewer::Markdown),
            Action::OpenAttachmentsTab => self.set_viewer(Viewer::Attachments),

            Action::OpenLogs => return Some(crate::Action::OpenLogViewer),
            Action::OpenMailInBrowser => self.open_html_mail_in_browser(),

            Action::Back => {
                return Some(crate::Action::Back);
            }
        };

        None
    }

    fn keybinding_manager(&mut self) -> &mut KeybindManager<Action> {
        &mut self.keybindings
    }

    fn overlay(&mut self) -> Option<&mut ScreenOverlay<PaletteType, InputType>> {
        self.overlay.as_mut()
    }

    fn handle_overlay_result(
        &mut self,
        result: ScreenOverlayResult<PaletteType, InputType>,
    ) -> Option<crate::Action> {
        self.overlay = None;

        match result {
            ScreenOverlayResult::Cancel => None,
            ScreenOverlayResult::Palette(value) => match value {
                PaletteType::Action(action) => self.apply_user_action(action),
            },
            ScreenOverlayResult::Input { value: _, typ: _ } => unreachable!(),
        }
    }

    // fn render_data(&'a mut self) -> RenderData<'a> {
    //     tracing::debug!("Selected viewer: {:?}", self.selected_viewer);
    //     let viewer_state = match self.selected_viewer {
    //         Viewer::Metadata => ViewerState::from(&mut self.metadata_viewer),
    //         Viewer::Text => {
    //             self.backend
    //                 .prefetch_mail_body(&self.id, MailBodyType::Text);

    //             ViewerState::from(&mut self.text_viewer)
    //         }
    //         Viewer::Markdown => {
    //             self.backend
    //                 .prefetch_mail_body(&self.id, MailBodyType::Html);

    //             ViewerState::from(&mut self.markdown_viewer)
    //         }
    //         Viewer::Attachments => {
    //             self.backend.prefetch_mail_attachments(&self.id);
    //             ViewerState::from(&mut self.attachment_viewer)
    //         }
    //     };

    //     let mail = self.backend.get_mail(&self.id).unwrap();

    //     RenderData {
    //         viewer_state,
    //         mail: MailDisplay::from(mail),
    //         scroll_queue: &mut self.scroll_action,
    //     }
    // }
}

impl Model {
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

    // TODO: If in attachment tab: Download the selected attachment and open the directory in it?
    //       Maybe create an action which downloads all attachments and then opens the directory?
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

    fn set_viewer(&mut self, viewer: Viewer) {
        self.selected_viewer = viewer;
    }

    fn open_html_mail_in_browser(&self) {
        todo!()
    }
}
