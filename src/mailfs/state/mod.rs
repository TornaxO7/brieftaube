mod column_state;
mod error;
mod input_type;
mod palette_value;
mod selection;

pub use column_state::{ColumnState, ColumnStateEntry};
pub use palette_value::PaletteValue;
pub use selection::{EntryId, SelectionType};

use super::Action;
use crate::{
    backend::{
        Backend,
        mailbox::types::{ParentMailboxId, TOP_PARENT_MAILBOX_ID},
        mails::types::{MailId, MailKeyword, MailUpdate},
        threads::types::ThreadId,
    },
    mailfs::widget::{ColumnDisplay, MailPreview, RenderData, RightColumn},
    utils::ui::{
        ScreenOverlay, ScreenOverlayResult, ScreenState, keybindmanager::KeybindManager, palette,
    },
};
use input_type::InputType;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    vec::Drain,
};
use tracing::{debug, instrument, warn};

pub struct State {
    app_actions: Vec<crate::Action>,
    keybindings: KeybindManager<Action>,
    overlay: Option<ScreenOverlay<PaletteValue, InputType>>,

    columns: HashMap<ParentMailboxId, ColumnState>,
    navigation_stack: Vec<ParentMailboxId>,

    /// stores which threads needs to be uncollapsed
    // if there needs to be more "task": Move it to an extra struct
    threads_to_uncollapse: HashMap<ParentMailboxId, HashSet<(MailId, ThreadId)>>,
    selection: HashMap<EntryId, SelectionType>,
    backend: Rc<Backend>,
}

impl State {
    pub fn new(backend: Rc<Backend>) -> Self {
        Self {
            backend,
            overlay: None,
            columns: HashMap::new(),
            selection: HashMap::new(),
            navigation_stack: vec![TOP_PARENT_MAILBOX_ID],
            app_actions: Vec::with_capacity(2),
            threads_to_uncollapse: HashMap::new(),
            keybindings: KeybindManager::new(HashMap::from([
                ("q", Action::Quit),
                ("j", Action::NavigateDown),
                ("l", Action::NavigateRight),
                ("<C-l>", Action::OpenLogs),
                ("h", Action::NavigateLeft),
                ("k", Action::NavigateUp),
                ("gg", Action::NavigateToTop),
                ("ge", Action::NavigateToBottom),
                (" ", Action::SelectEntryToggle),
                (":", Action::OpenCommandPalette),
            ])),
        }
    }
}

impl<'a> ScreenState<'a, Action, PaletteValue, InputType, RenderData<'a>> for State {
    #[instrument(skip(self))]
    fn apply_action(&mut self, action: Action) {
        debug!("{:?}", action);
        match action {
            Action::Quit => self.quit(),
            Action::OpenCommandPalette => self.open_command_palette(),
            Action::NavigateDown => self.navigate_down(),
            Action::NavigateUp => self.navigate_up(),
            Action::NavigateToTop => self.navigate_to_top(),
            Action::NavigateToBottom => self.navigate_to_bottom(),
            Action::NavigateRight => self.navigate_right(),
            Action::NavigateLeft => self.navigate_left(),
            Action::NavigateToParent => self.navigate_to_parent(),
            Action::OpenLogs => self.open_logs(),

            Action::SelectEntryToggle => self.select_entry(),
            Action::CutSelectedEntries => self.cut_selected_entries(),
            Action::PasteSelectedEntries => self.paste_selected_entries(),

            Action::MarkMailAsUnseen => self.mail_patch_keywords(&[(MailKeyword::Seen, false)]),
            Action::MarkMailAsSeen => self.mail_patch_keywords(&[(MailKeyword::Seen, true)]),
        }
    }

    fn get_app_actions(&mut self) -> Drain<'_, crate::Action> {
        self.app_actions.drain(..)
    }

    fn keybinding_manager(&mut self) -> &mut KeybindManager<Action> {
        &mut self.keybindings
    }

    fn overlay(&mut self) -> Option<&mut ScreenOverlay<PaletteValue, InputType>> {
        self.overlay.as_mut()
    }

    fn handle_overlay_result(&mut self, result: ScreenOverlayResult<PaletteValue, InputType>) {
        match result {
            ScreenOverlayResult::Palette(value) => match value {
                PaletteValue::Action(action) => {
                    self.apply_action(action);
                    self.overlay = None;
                }
            },
            ScreenOverlayResult::Cancel => self.overlay = None,
            ScreenOverlayResult::Input { .. } => unreachable!(),
        }
    }

    fn render_data(&'a mut self) -> RenderData<'a> {
        self.update_columns();
        let backend = self.backend.clone();

        let mailbox_path = self
            .navigation_stack
            .iter()
            .map(|id| match id {
                Some(id) => {
                    let mailbox = backend.mailbox_get_data(id).unwrap();
                    format!("{}/", mailbox.name)
                }
                None => String::from("/"),
            })
            .collect::<String>();

        let right_preview = self
            .get_center_column()
            .and_then(|center_column| center_column.selected_entry())
            .and_then(|selected_entry| match selected_entry {
                ColumnStateEntry::Mailbox(_) => None,
                ColumnStateEntry::SingleMail(id)
                | ColumnStateEntry::CollapsedThread(id, _)
                | ColumnStateEntry::ThreadStart { mail_id: id, .. }
                | ColumnStateEntry::ThreadChild(id, _)
                | ColumnStateEntry::ThreadEnd(id, _) => Some(id),
            })
            .and_then(|mail_id| backend.get_mail(mail_id))
            .map(MailPreview::from)
            .map(RightColumn::MailPreview);

        let (left, center, right_column) = {
            let center_mailbox = self.get_center_column_mailbox();
            match (
                self.get_left_column_mailbox(),
                self.get_right_column_mailbox(),
            ) {
                (Some(left_mailbox), Some(right_mailbox)) => {
                    debug!("{:?}, {:?}", left_mailbox, right_mailbox);
                    let [left, center, right] = self.columns.get_disjoint_mut([
                        &left_mailbox,
                        &center_mailbox,
                        &right_mailbox,
                    ]);

                    (
                        ColumnDisplay::new(left, &self.selection, backend.clone()).ok(),
                        ColumnDisplay::new(center, &self.selection, backend.clone()).ok(),
                        ColumnDisplay::new(right, &self.selection, backend.clone()).ok(),
                    )
                }
                (Some(left_mailbox), None) => {
                    let [left, center] = self
                        .columns
                        .get_disjoint_mut([&left_mailbox, &center_mailbox]);

                    (
                        ColumnDisplay::new(left, &self.selection, backend.clone()).ok(),
                        ColumnDisplay::new(center, &self.selection, backend.clone()).ok(),
                        None,
                    )
                }
                (None, Some(right_mailbox)) => {
                    let [center, right] = self
                        .columns
                        .get_disjoint_mut([&center_mailbox, &right_mailbox]);

                    (
                        None,
                        ColumnDisplay::new(center, &self.selection, backend.clone()).ok(),
                        ColumnDisplay::new(right, &self.selection, backend.clone()).ok(),
                    )
                }
                (None, None) => {
                    let center_column = self.columns.get_mut(self.navigation_stack.last().unwrap());
                    (
                        None,
                        ColumnDisplay::new(center_column, &self.selection, backend.clone()).ok(),
                        None,
                    )
                }
            }
        };

        let right = right_preview.or(right_column.map(|column| RightColumn::ColumnData(column)));

        RenderData {
            mailbox_path,
            left,
            center,
            right,
        }
    }
}

/// Helper functions
impl State {
    fn get_left_column_mailbox(&self) -> Option<ParentMailboxId> {
        (self.navigation_stack.len().checked_sub(2))
            .map(|idx| &self.navigation_stack[idx])
            .cloned()
    }

    fn get_center_column_mailbox(&self) -> ParentMailboxId {
        self.navigation_stack.last().cloned().unwrap()
    }

    fn get_right_column_mailbox(&self) -> Option<ParentMailboxId> {
        self.get_center_column()
            .and_then(|center| center.selected_entry())
            .cloned()
            .and_then(|selected| {
                if let ColumnStateEntry::Mailbox(id) = selected.clone() {
                    Some(Some(id))
                } else {
                    None
                }
            })
    }

    fn get_center_column(&self) -> Option<&ColumnState> {
        self.columns.get(self.navigation_stack.last().unwrap())
    }

    fn get_center_column_mut(&mut self) -> Option<&mut ColumnState> {
        self.columns.get_mut(self.navigation_stack.last().unwrap())
    }
}

/// Action implementations
impl State {
    fn quit(&mut self) {
        self.app_actions.push(crate::Action::Quit);
    }

    fn open_command_palette(&mut self) {
        self.overlay = Some(ScreenOverlay::Palette(palette::State::new(
            Action::palette_options(),
        )));
    }

    fn navigate_down(&mut self) {
        if let Some(column) = self.get_center_column_mut() {
            let pos = column.state.selected();
            let new_pos = pos.map(|old_pos| (old_pos + 1).min(column.entries().len() - 1));
            column.state.select(new_pos);
        }
    }

    fn navigate_up(&mut self) {
        if let Some(column) = self.get_center_column_mut() {
            column.state.select_previous();
        }
    }

    fn navigate_to_top(&mut self) {
        if let Some(column) = self.get_center_column_mut() {
            column.state.select_first();
        }
    }

    fn navigate_to_bottom(&mut self) {
        if let Some(column) = self.get_center_column_mut() {
            if column.entries().is_empty() {
                column.state.select(None);
            } else {
                let len = column.entries().len();
                column.state.select(Some(len - 1));
            }
        }
    }

    fn navigate_right(&mut self) {
        let center_column = self.columns.get(self.navigation_stack.last().unwrap());

        if let Some(column) = center_column {
            if let Some(entry) = column.selected_entry().cloned() {
                match entry.clone() {
                    ColumnStateEntry::Mailbox(id) => {
                        self.navigation_stack.push(Some(id));
                    }
                    ColumnStateEntry::ThreadStart { mail_id, .. }
                    | ColumnStateEntry::ThreadChild(mail_id, _)
                    | ColumnStateEntry::ThreadEnd(mail_id, _)
                    | ColumnStateEntry::SingleMail(mail_id) => {
                        self.app_actions
                            .push(crate::Action::OpenMailViewer(mail_id));
                    }
                    ColumnStateEntry::CollapsedThread(mail_id, thread_id) => {
                        // we need to create a "task" because we may need to wait for the server...
                        let list = self
                            .threads_to_uncollapse
                            .get_mut(column.mailbox())
                            .unwrap();
                        list.insert((mail_id, thread_id));
                    }
                }
            }
        }
    }

    fn navigate_left(&mut self) {
        if let Some(column) = self.get_center_column_mut() {
            if let Some(entry) = column.selected_entry() {
                match &entry {
                    ColumnStateEntry::Mailbox(_)
                    | ColumnStateEntry::SingleMail(_)
                    | ColumnStateEntry::CollapsedThread(_, _) => {
                        self.navigate_to_parent();
                    }
                    ColumnStateEntry::ThreadStart { thread_id, .. }
                    | ColumnStateEntry::ThreadChild(_, thread_id)
                    | ColumnStateEntry::ThreadEnd(_, thread_id) => {
                        let (start_pos, new_entry) = column
                            .entries()
                            .iter()
                            .cloned()
                            .enumerate()
                            .find_map(|(idx, entry)| {
                                if let ColumnStateEntry::ThreadStart {
                                    thread_id: entry_thread_id,
                                    collapsed_mail_id,
                                    ..
                                } = entry
                                {
                                    if &entry_thread_id == thread_id {
                                        Some((
                                            idx,
                                            ColumnStateEntry::CollapsedThread(
                                                collapsed_mail_id,
                                                entry_thread_id,
                                            ),
                                        ))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                            .expect(
                                "Well... we are looking for the entry. It can't just disappear o.O",
                            );

                        let end_pos = column.entries().iter().position(|entry| matches!(entry, ColumnStateEntry::ThreadEnd(_, entry_thread_id) if entry_thread_id == thread_id))
                            .expect("Same as in the previous `.expect`.");

                        column
                            .entries_mut()
                            .splice(start_pos..=end_pos, [new_entry]);
                    }
                };
                return;
            }
        }

        // fallback
        self.navigate_to_parent();
    }

    fn navigate_to_parent(&mut self) {
        if self.navigation_stack.len() > 1 {
            self.navigation_stack.pop();
        }
    }

    fn open_logs(&mut self) {
        self.app_actions.push(crate::Action::OpenLogViewer);
    }

    fn select_entry(&mut self) {
        if let Some(column) = self.get_center_column() {
            if let Some(entry) = column.selected_entry() {
                let id = EntryId::from(entry);

                if self.selection.remove(&id).is_none() {
                    self.selection.insert(id, SelectionType::Selected);
                }

                self.navigate_down();
            }
        }
    }

    fn cut_selected_entries(&mut self) {
        if self.selection.is_empty() {
            if let Some(column) = self.get_center_column() {
                if let Some(entry) = column.selected_entry() {
                    let id = EntryId::from(entry);

                    self.selection.insert(id, SelectionType::Cut);
                }
            }
        } else {
            for (_id, selection) in self.selection.iter_mut() {
                *selection = SelectionType::Cut;
            }
        }
    }

    fn paste_selected_entries(&mut self) {
        for (entry_id, selection) in self.selection.drain() {
            match selection {
                SelectionType::Selected => {}
                SelectionType::Cut => match entry_id {
                    EntryId::Mail(id) => {}
                    EntryId::Mailbox(id) => {}
                },
            }
        }
    }

    fn mail_patch_keywords(&mut self, patch: &[(MailKeyword, bool)]) {
        if !self.selection.is_empty() {
            let mut updates = Vec::with_capacity(self.selection.len());

            for (id, ty) in self.selection.drain() {
                if ty == SelectionType::Selected {
                    match id {
                        EntryId::Mail(id) => updates.push(MailUpdate {
                            id,
                            patch_keywords: Some(patch.to_vec()),
                            ..Default::default()
                        }),
                        EntryId::Mailbox(_) => {}
                    }
                }
            }

            self.backend.mails_update(updates);
            return;
        }

        // use current selected entry
        if let Some(column) = self.get_center_column() {
            if let Some(entry) = column.selected_entry() {
                match &entry {
                    ColumnStateEntry::Mailbox(_) => {}
                    ColumnStateEntry::SingleMail(mail_id)
                    | ColumnStateEntry::CollapsedThread(mail_id, _)
                    | ColumnStateEntry::ThreadStart { mail_id, .. }
                    | ColumnStateEntry::ThreadChild(mail_id, _)
                    | ColumnStateEntry::ThreadEnd(mail_id, _) => {
                        self.backend.mails_update(vec![MailUpdate {
                            id: mail_id.clone(),
                            patch_keywords: Some(patch.to_vec()),
                            ..Default::default()
                        }])
                    }
                }
            }
        }
    }
}

/// Methods for loading the columns
impl<'a> State {
    fn update_columns(&mut self) {
        let current_column = self.navigation_stack.last().cloned().unwrap();
        self.update_column(&current_column);

        let preview_mail_id = {
            self.get_center_column()
                .and_then(|column| column.selected_entry())
                .and_then(|entry| match entry {
                    ColumnStateEntry::Mailbox(_) => None,
                    ColumnStateEntry::SingleMail(mail_id)
                    | ColumnStateEntry::CollapsedThread(mail_id, _)
                    | ColumnStateEntry::ThreadStart { mail_id, .. }
                    | ColumnStateEntry::ThreadChild(mail_id, _)
                    | ColumnStateEntry::ThreadEnd(mail_id, _) => Some(mail_id),
                })
        };
        if let Some(id) = preview_mail_id {
            self.backend.request_mails_attachments(&id);
        }

        if let Some(right_mailbox) = self.get_right_column_mailbox() {
            self.update_column(&right_mailbox);
        }
    }

    fn update_column(&mut self, id: &ParentMailboxId) {
        match self.columns.get_mut(id) {
            None => {
                if let Ok(entries) =
                    ColumnStateEntry::create_entries(id.clone(), self.backend.clone())
                {
                    let state = ColumnState::new(id.clone(), entries);
                    self.columns.insert(id.clone(), state);
                    self.threads_to_uncollapse
                        .insert(id.clone(), HashSet::new());
                }
            }
            Some(column) => {
                // uncollapse threads if needed
                for (collapsed_mail_id, thread_id) in
                    self.threads_to_uncollapse.get(id).cloned().unwrap()
                {
                    if let Some(mut thread_mails) =
                        self.backend.get_or_request_thread_mails(&thread_id)
                    {
                        debug_assert!(
                            thread_mails.len() >= 2,
                            "Uncollapseable threads must have at least 2 mails <.<"
                        );

                        // according to the jmap specs: The thread saves the mails from oldest to latest,
                        // but we want the newest mail to be first: So reverse it
                        thread_mails.reverse();

                        let new_entries = {
                            let (first, rest) = thread_mails.split_first().unwrap();
                            let (last, inner) = rest.split_last().unwrap();

                            let mut new_entries = vec![ColumnStateEntry::ThreadStart {
                                mail_id: first.id.clone(),
                                thread_id: thread_id.clone(),
                                collapsed_mail_id: collapsed_mail_id.clone(),
                            }];

                            new_entries.extend(inner.iter().map(|mail| {
                                ColumnStateEntry::ThreadChild(mail.id.clone(), thread_id.clone())
                            }));

                            new_entries.push(ColumnStateEntry::ThreadEnd(
                                last.id.clone(),
                                thread_id.clone(),
                            ));

                            new_entries
                        };

                        match column
                            .entries()
                            .iter()
                            .position(|entry| matches!(entry, ColumnStateEntry::CollapsedThread(_, entry_thread_id) if entry_thread_id == &thread_id))
                        {
                            Some(thread_idx) => {
                                column
                                    .entries_mut()
                                    .splice(thread_idx..(thread_idx + 1), new_entries);
                            },
                            None => {
                                warn!("Eh, the thread with the id '{}' seems to be disappeared so it can't be uncollapsed now.... weird. Welp, it won't get uncollapsed then.", thread_id.0);
                            }
                        };

                        let threads_to_uncollapse = self.threads_to_uncollapse.get_mut(id).unwrap();
                        threads_to_uncollapse.remove(&(collapsed_mail_id, thread_id));
                    }
                }

                // TODO: Update column by comparing current entries with entries from cache/backend.
                //
                // Steps:
                // 1. Add new mailboxes
                // 2. Update mailboxes
                // 3. Remove removed mailboxes
                //
                // 4. Add new mails
                // 5. Update mails
                // 6. Remove removed mails from backend
            }
        }
    }
}
