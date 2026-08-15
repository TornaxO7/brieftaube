pub mod attachments;
pub mod markdown;
pub mod metadata;
pub mod text;

use super::Action;
use crate::utils::layer::LayerModelDefaultHandleEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::Text,
    widgets::ScrollbarState,
};

#[derive(Debug, Clone, Copy)]
enum ScrollAction {
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

trait MailViewerSubModel {
    fn quit(&self) -> Option<Action> {
        Some(Action::Quit)
    }

    fn back(&self) -> Option<Action> {
        Some(Action::Back)
    }

    fn open_metadata_tab(&self) -> Option<Action> {
        Some(Action::OpenMetadataTab)
    }

    fn open_text_tab(&self) -> Option<Action> {
        Some(Action::OpenTextTab)
    }

    fn open_markdown_tab(&self) -> Option<Action> {
        Some(Action::OpenMarkdownTab)
    }

    fn open_attachments_tab(&self) -> Option<Action> {
        Some(Action::OpenAttachmentsTab)
    }

    fn open_next_tab(&self) -> Option<Action> {
        Some(Action::OpenNextTab)
    }

    fn open_previous_tab(&self) -> Option<Action> {
        Some(Action::OpenPreviousTab)
    }

    fn open_logs(&self) -> Option<Action> {
        Some(Action::OpenLogs)
    }
}

trait MailViewerPager<SubmoduleAction>: LayerModelDefaultHandleEvent<SubmoduleAction, Action>
where
    SubmoduleAction: Clone,
{
    fn set_scroll_action(&mut self, scroll: ScrollAction);

    fn scroll_down(&mut self) -> Option<Action> {
        let keybindings = self.keybinding_manager();

        let offset = keybindings.flush_int_prefix().unwrap_or(1);
        self.set_scroll_action(ScrollAction::ScrollDown(offset));

        None
    }

    fn scroll_up(&mut self) -> Option<Action> {
        let keybindings = self.keybinding_manager();

        let offset = keybindings.flush_int_prefix().unwrap_or(1);
        self.set_scroll_action(ScrollAction::ScrollUp(offset));

        None
    }

    fn scroll_left(&mut self) -> Option<Action> {
        let keybindings = self.keybinding_manager();

        let offset = keybindings.flush_int_prefix().unwrap_or(1);
        self.set_scroll_action(ScrollAction::ScrollLeft(offset));

        None
    }

    fn scroll_right(&mut self) -> Option<Action> {
        let keybindings = self.keybinding_manager();

        let offset = keybindings.flush_int_prefix().unwrap_or(1);
        self.set_scroll_action(ScrollAction::ScrollRight(offset));

        None
    }

    fn scroll_to_top(&mut self) -> Option<Action> {
        self.set_scroll_action(ScrollAction::SetTop);
        None
    }

    fn scroll_to_bottom(&mut self) -> Option<Action> {
        self.set_scroll_action(ScrollAction::SetBottom);
        None
    }

    fn scroll_half_page_down(&mut self) -> Option<Action> {
        self.set_scroll_action(ScrollAction::ScrollHalfPageDown);
        None
    }

    fn scroll_half_page_up(&mut self) -> Option<Action> {
        self.set_scroll_action(ScrollAction::ScrollHalfPageUp);
        None
    }

    fn scroll_half_page_right(&mut self) -> Option<Action> {
        self.set_scroll_action(ScrollAction::ScrollHalfPageRight);
        None
    }

    fn scroll_half_page_left(&mut self) -> Option<Action> {
        self.set_scroll_action(ScrollAction::ScrollHalfPageLeft);
        None
    }
}

fn new_pos(pos: u16, inner_max: u16, area_max: u16, offset: u16, inc: bool) -> u16 {
    let unseen_lines_or_columns = inner_max.saturating_sub(area_max);

    if inc {
        (pos + offset).min(unseen_lines_or_columns)
    } else {
        pos.saturating_sub(offset).min(unseen_lines_or_columns)
    }
}

fn adjust_scrollbars(
    text: &Text,
    area: Rect,
    vertical: &mut ScrollbarState,
    horizontal: &mut ScrollbarState,
    scroll_action: Option<ScrollAction>,
) -> (Rect, Option<Rect>, Option<Rect>) {
    let amount_unseen_lines = text.height().saturating_sub(area.height as usize);
    let amount_unseen_columns = text.width().saturating_sub(area.width as usize);

    if let Some(action) = scroll_action {
        match action {
            ScrollAction::ScrollUp(amount) => {
                let pos = vertical.get_position();
                *vertical = vertical.position(pos.saturating_sub(amount));
            }
            ScrollAction::ScrollDown(amount) => {
                let pos = vertical.get_position();
                *vertical = vertical.position((pos + amount).min(amount_unseen_lines));
            }
            ScrollAction::ScrollHalfPageDown => {
                let prev_pos = vertical.get_position();
                let new_pos = prev_pos + area.height as usize / 2;
                *vertical = vertical.position(new_pos.min(amount_unseen_lines));
            }
            ScrollAction::ScrollHalfPageUp => {
                let prev_pos = vertical.get_position();
                *vertical = vertical.position(prev_pos.saturating_sub(area.height as usize / 2));
            }
            ScrollAction::SetTop => vertical.first(),
            ScrollAction::SetBottom => vertical.last(),
            ScrollAction::ScrollHalfPageRight => {
                let prev_pos = horizontal.get_position();
                let new_pos = prev_pos + area.width as usize / 2;
                *horizontal = horizontal.position(new_pos.min(amount_unseen_columns));
            }
            ScrollAction::ScrollRight(amount) => {
                let prev_pos = horizontal.get_position();
                let new_pos = prev_pos + amount;
                *horizontal = horizontal.position(new_pos.min(amount_unseen_columns));
            }
            ScrollAction::ScrollHalfPageLeft => {
                let prev_pos = horizontal.get_position();
                *horizontal = horizontal.position(prev_pos.saturating_sub(area.width as usize / 2));
            }
            ScrollAction::ScrollLeft(amount) => {
                let prev_pos = horizontal.get_position();
                *horizontal = horizontal.position(prev_pos.saturating_sub(amount));
            }
        }
    }

    // restrict height
    *vertical = vertical.content_length(amount_unseen_lines);
    let (rest, vertical_scrollbar_area) = {
        let scrollbar_is_visible = amount_unseen_lines > 0;
        if scrollbar_is_visible {
            let [rest, scrollbar_area] =
                Layout::horizontal([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

            (rest, Some(scrollbar_area))
        } else {
            (area, None)
        }
    };

    // restrict width
    *horizontal = horizontal.content_length(amount_unseen_columns);
    let (mail_content_area, horizontal_scrollbar_area) = {
        let scrollbar_is_visible = amount_unseen_columns > 0;
        if scrollbar_is_visible {
            let [mail_content_area, scrollbar_area] =
                Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(rest);

            (mail_content_area, Some(scrollbar_area))
        } else {
            (rest, None)
        }
    };

    (
        mail_content_area,
        vertical_scrollbar_area,
        horizontal_scrollbar_area,
    )
}
