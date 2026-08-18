mod column_state;
mod selection;

use super::UserAction;
use crate::{
    backend::{
        Backend, MailId, MailboxData, MailboxId, MailboxNew, MailboxUpdate, ThreadId,
        mailbox::{
            RemoveMailboxOption,
            types::{ParentMailboxId, TOP_PARENT_MAILBOX_ID},
        },
        mails::types::{MailKeyword, MailUpdate},
    },
    task_manager::TaskManager,
    utils::{
        keybindmanager::KeybindManager,
        layer::{LayerCore, LayerModel, LayerModelDefaultHandleEvent, LayerOverlay},
    },
};
use std::{
    collections::HashMap,
    rc::Rc,
    str::FromStr,
    sync::{Arc, Mutex},
};
use throbber_widgets_tui::ThrobberState;
use tracing::{debug, error, warn};

pub use column_state::{ColumnState, ColumnStateEntry};
pub use selection::{EntryId, SelectionType};

const NORMALIZE_SORT_ORDER_SIZE: u32 = 32;

pub type Columns = Arc<Mutex<HashMap<ParentMailboxId, ColumnState>>>;

enum OverlayValue {
    Action,
    NewMailboxName,
}

pub enum RightColumn {
    Mailbox(MailboxId),
    MailPreview(MailId),
}

pub struct Model {
    keybindings: KeybindManager<UserAction>,
    task_manager: Rc<TaskManager>,
    overlay_value: Option<OverlayValue>,

    pub throbber: ThrobberState,
    pub backend: Arc<Backend>,
    pub selection: HashMap<EntryId, SelectionType>,
    pub navigation_stack: Vec<ParentMailboxId>,
    pub columns: Columns,
}

impl Model {
    pub fn new(backend: Arc<Backend>, task_manager: Rc<TaskManager>) -> Self {
        let columns = Arc::new(Mutex::new(HashMap::new()));
        let columns2 = columns.clone();
        let backend2 = backend.clone();

        task_manager.spawn(async move {
            init_mailfs(columns2, backend2).await;
        });

        Self {
            columns,
            overlay_value: None,
            throbber: ThrobberState::default(),
            navigation_stack: vec![TOP_PARENT_MAILBOX_ID],
            selection: HashMap::new(),
            task_manager,

            backend,
            keybindings: KeybindManager::new(HashMap::from([
                ("q", UserAction::Quit),
                ("j", UserAction::NavigateDown),
                ("l", UserAction::NavigateRight),
                ("<C-l>", UserAction::OpenLogs),
                ("h", UserAction::NavigateLeft),
                ("k", UserAction::NavigateUp),
                ("gg", UserAction::NavigateToTop),
                ("ge", UserAction::NavigateToBottom),
                (" ", UserAction::SelectEntryToggle),
                (":", UserAction::OpenCommandPalette),
            ])),
        }
    }
}

impl LayerCore for Model {
    fn handle_event(
        &mut self,
        event: crossterm::event::Event,
        statusbar: &mut crate::statusbar::Model,
    ) -> Option<crate::Action> {
        <Self as LayerModelDefaultHandleEvent<UserAction>>::handle_event(self, event, statusbar)
    }
}

impl LayerModel<UserAction> for Model {
    fn apply_action(&mut self, action: UserAction) -> Option<crate::Action> {
        debug!("{:?}", action);

        match action {
            UserAction::Quit => self.quit(),
            UserAction::OpenCommandPalette => self.open_command_palette(),
            UserAction::NavigateDown => self.navigate_down(),
            UserAction::NavigateUp => self.navigate_up(),
            UserAction::NavigateToTop => self.navigate_to_top(),
            UserAction::NavigateToBottom => self.navigate_to_bottom(),
            UserAction::NavigateRight => return self.navigate_right(),
            UserAction::NavigateLeft => self.navigate_left(),
            UserAction::NavigateToParent => self.navigate_to_parent(),
            UserAction::OpenLogs => self.open_logs(),

            UserAction::SelectEntryToggle => self.select_entry(),
            UserAction::CutSelectedEntries => self.cut_selected_entries(),
            UserAction::PasteSelectedEntries => self.paste_selected_entries(),

            UserAction::MoveMailboxUp => self.move_mailbox_up(),
            UserAction::MoveMailboxDown => self.move_mailbox_down(),

            UserAction::CreateMailbox => self.create_mailbox(),
            UserAction::RemoveMailbox => self.remove_mailbox(),

            UserAction::MarkMailAsUnseen => self.mail_patch_keywords(&[(MailKeyword::Seen, false)]),
            UserAction::MarkMailAsSeen => self.mail_patch_keywords(&[(MailKeyword::Seen, true)]),
        }
    }

    fn handle_overlay<O: LayerOverlay>(&mut self, overlay: O) -> Option<crate::Action> {
        let expected_type = self.overlay_value.take()?;
        let msg = overlay.into_message()?;

        match expected_type {
            OverlayValue::Action => {
                let action = UserAction::from_str(&msg).unwrap();
                self.apply_action(action)
            }
            OverlayValue::NewMailboxName => {
                let column_id = self.center_column_mailbox().clone();
                let columns = self.columns.clone();
                let backend = self.backend.clone();
                self.task_manager.spawn(async move {
                    create_new_mailbox(msg, column_id, columns, backend).await;
                });

                None
            }
        }
    }
}

impl LayerModelDefaultHandleEvent<UserAction> for Model {
    fn keybinding_manager(&mut self) -> &mut KeybindManager<UserAction> {
        &mut self.keybindings
    }
}

/// Helper functions
impl<'a> Model {
    pub fn left_column_mailbox(&self) -> Option<&ParentMailboxId> {
        (self.navigation_stack.len().checked_sub(2)).map(|idx| &self.navigation_stack[idx])
    }

    pub fn center_column_mailbox(&self) -> &ParentMailboxId {
        self.navigation_stack.last().unwrap()
    }

    pub fn right_column(
        &self,
        columns: &HashMap<ParentMailboxId, ColumnState>,
    ) -> Option<RightColumn> {
        let center = self.center_column_mailbox();

        columns
            .get(&center)
            .and_then(|center| center.selected_entry())
            .map(|selected| match selected.clone() {
                ColumnStateEntry::Mailbox(id) => RightColumn::Mailbox(id),
                ColumnStateEntry::SingleMail(mail_id)
                | ColumnStateEntry::CollapsedThread(mail_id, _)
                | ColumnStateEntry::ThreadStart { mail_id, .. }
                | ColumnStateEntry::ThreadChild(mail_id, _)
                | ColumnStateEntry::ThreadEnd(mail_id, _) => RightColumn::MailPreview(mail_id),
            })
    }

    fn load_right_column_for(&self, entry: ColumnStateEntry) {
        match entry {
            ColumnStateEntry::Mailbox(id) => {
                let right_column_not_loaded = {
                    let columns = self.columns.lock().unwrap();
                    !columns.contains_key(&Some(id.clone()))
                };

                if right_column_not_loaded {
                    let id = id.clone();
                    let columns = self.columns.clone();
                    let backend = self.backend.clone();

                    self.task_manager.spawn(async move {
                        match op_init_mailbox(Some(id), columns, backend).await {
                            Ok(()) => {}
                            Err(err) => {
                                error!("Couldn't initialize mailbox for right column:\n{err}");
                            }
                        }
                    });
                }
            }
            ColumnStateEntry::SingleMail(mail_id)
            | ColumnStateEntry::CollapsedThread(mail_id, _)
            | ColumnStateEntry::ThreadStart { mail_id, .. }
            | ColumnStateEntry::ThreadChild(mail_id, _)
            | ColumnStateEntry::ThreadEnd(mail_id, _) => {
                let misses_attachments = {
                    let mail = self.backend.get_mail(&mail_id).unwrap();
                    mail.attachments.loaded().is_none()
                };

                tracing::debug!("Hello: {}", misses_attachments);

                if misses_attachments {
                    let mail_id = mail_id.clone();
                    let backend = self.backend.clone();
                    self.task_manager.spawn(async move {
                        match backend.prefetch_mail_attachments(&mail_id).await {
                            Ok(()) => {}
                            Err(err) => {
                                error!("Couldn't fetch mail attachments:\n{err}");
                            }
                        }
                    });
                }
            }
        }
    }

    fn _normalize_mailbox_sort_order(&self) {
        let ids: Vec<MailboxId> = {
            let columns = self.columns.lock().unwrap();
            let Some(center) = columns.get(self.center_column_mailbox()) else {
                return;
            };

            center
                .entries()
                .iter()
                .map_while(|entry| {
                    if let ColumnStateEntry::Mailbox(id) = entry {
                        Some(id.clone())
                    } else {
                        None
                    }
                })
                .collect()
        };

        let backend = self.backend.clone();
        self.task_manager.spawn(async move {
            let updates: Vec<MailboxUpdate> = ids
                .into_iter()
                .enumerate()
                .map(|(idx, id)| MailboxUpdate {
                    id,
                    sort_order: Some((idx as u32 + 1) * NORMALIZE_SORT_ORDER_SIZE),
                    ..Default::default()
                })
                .collect();

            if let Err(err) = backend.update_mailboxes(updates).await {
                error!("Couldn't normalize sort order of mailboxes:\n{err}");
            }
        });
    }
}

/// Action implementations
impl Model {
    fn quit(&self) -> Option<crate::Action> {
        Some(crate::Action::Quit)
    }

    fn open_command_palette(&mut self) -> Option<crate::Action> {
        self.overlay_value = Some(OverlayValue::Action);
        let entries = UserAction::palette_options();
        Some(crate::Action::OpenPalette { entries })
    }

    fn navigate_down(&self) -> Option<crate::Action> {
        let selected_entry = {
            let mut columns = self.columns.lock().unwrap();
            columns
                .get_mut(&self.center_column_mailbox())
                .and_then(|center| {
                    let pos = center.state.selected();
                    let new_pos = pos.map(|old_pos| (old_pos + 1).min(center.entries().len() - 1));
                    center.state.select(new_pos);
                    center.selected_entry().cloned()
                })
        };

        if let Some(entry) = selected_entry {
            self.load_right_column_for(entry);
        }
        None
    }

    fn navigate_up(&self) -> Option<crate::Action> {
        let selected_entry = {
            let mut columns = self.columns.lock().unwrap();
            columns
                .get_mut(self.center_column_mailbox())
                .and_then(|center| {
                    center.state.select_previous();
                    center.selected_entry().cloned()
                })
        };

        if let Some(entry) = selected_entry {
            self.load_right_column_for(entry);
        }

        None
    }

    fn navigate_to_top(&mut self) -> Option<crate::Action> {
        let selected_entry = {
            let mut columns = self.columns.lock().unwrap();
            columns
                .get_mut(self.center_column_mailbox())
                .and_then(|center| {
                    center.state.select_first();
                    center.selected_entry().cloned()
                })
        };

        if let Some(entry) = selected_entry {
            self.load_right_column_for(entry);
        }

        None
    }

    fn navigate_to_bottom(&mut self) -> Option<crate::Action> {
        let selected_entry = {
            let mut columns = self.columns.lock().unwrap();
            columns
                .get_mut(self.center_column_mailbox())
                .and_then(|center| {
                    if center.entries().is_empty() {
                        center.state.select(None);
                    } else {
                        let len = center.entries().len();
                        center.state.select(Some(len - 1));
                    }

                    center.selected_entry().cloned()
                })
        };

        if let Some(entry) = selected_entry {
            self.load_right_column_for(entry);
        }

        None
    }

    fn navigate_right(&mut self) -> Option<crate::Action> {
        let selected_entry = {
            let columns = self.columns.lock().unwrap();
            columns
                .get(self.center_column_mailbox())
                .and_then(|column| column.selected_entry().cloned())
        };

        if let Some(entry) = selected_entry {
            match entry {
                ColumnStateEntry::Mailbox(id) => {
                    self.navigation_stack.push(Some(id));

                    let selected_entry = {
                        let columns = self.columns.lock().unwrap();
                        let column = columns.get(self.center_column_mailbox());

                        column.and_then(|column| column.selected_entry().cloned())
                    };

                    if let Some(entry) = selected_entry {
                        self.load_right_column_for(entry);
                    }
                }
                ColumnStateEntry::ThreadStart { mail_id, .. }
                | ColumnStateEntry::ThreadChild(mail_id, _)
                | ColumnStateEntry::ThreadEnd(mail_id, _)
                | ColumnStateEntry::SingleMail(mail_id) => {
                    return Some(crate::Action::OpenMailViewer(mail_id));
                }
                ColumnStateEntry::CollapsedThread(mail_id, thread_id) => {
                    let column_mailbox = self
                        .center_column_mailbox()
                        .clone()
                        .expect("Is not root mailbox");
                    let columns = self.columns.clone();
                    let backend = self.backend.clone();

                    self.task_manager.spawn(async move {
                        match op_uncollapse_thread(
                            column_mailbox,
                            mail_id,
                            thread_id,
                            columns,
                            backend,
                        )
                        .await
                        {
                            Ok(()) => {}
                            Err(err) => {
                                error!("Can't uncollapse thread:\n{err}");
                            }
                        }
                    });
                }
            }
        }

        None
    }

    fn navigate_left(&mut self) -> Option<crate::Action> {
        let selected_entry = {
            let mut columns = self.columns.lock().unwrap();
            columns
                .get_mut(self.center_column_mailbox())
                .and_then(|center| center.selected_entry().cloned())
        };

        if let Some(entry) = selected_entry {
            match &entry {
                ColumnStateEntry::Mailbox(_)
                | ColumnStateEntry::SingleMail(_)
                | ColumnStateEntry::CollapsedThread(_, _) => {
                    self.navigate_to_parent();
                }
                ColumnStateEntry::ThreadStart { thread_id, .. }
                | ColumnStateEntry::ThreadChild(_, thread_id)
                | ColumnStateEntry::ThreadEnd(_, thread_id) => {
                    let mut columns = self.columns.lock().unwrap();
                    let column = columns.get_mut(self.center_column_mailbox()).unwrap();

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

                    column.state.select(Some(start_pos));
                }
            };
            return None;
        }

        // fallback
        self.navigate_to_parent();

        None
    }

    fn navigate_to_parent(&mut self) -> Option<crate::Action> {
        if self.navigation_stack.len() > 1 {
            self.navigation_stack.pop();
        }

        None
    }

    fn open_logs(&mut self) -> Option<crate::Action> {
        Some(crate::Action::OpenLogViewer)
    }

    fn select_entry(&mut self) -> Option<crate::Action> {
        let selected_entry = {
            let columns = self.columns.lock().unwrap();
            columns
                .get(self.center_column_mailbox())
                .and_then(|center| center.selected_entry().cloned())
        };

        if let Some(entry) = selected_entry {
            let id = EntryId::from(entry);

            if self.selection.remove(&id).is_none() {
                self.selection.insert(id, SelectionType::Selected);
            }

            self.navigate_down();
        }

        None
    }

    fn cut_selected_entries(&mut self) -> Option<crate::Action> {
        if self.selection.is_empty() {
            let columns = self.columns.lock().unwrap();
            if let Some(column) = columns.get(self.center_column_mailbox()) {
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

        None
    }

    fn paste_selected_entries(&mut self) -> Option<crate::Action> {
        for (entry_id, selection) in self.selection.drain() {
            match selection {
                SelectionType::Selected => {}
                SelectionType::Cut => match entry_id {
                    EntryId::Mail(_id) => {
                        todo!()
                    }
                    EntryId::Mailbox(_id) => {
                        todo!()
                    }
                },
            }
        }

        None
    }

    fn move_mailbox_up(&mut self) -> Option<crate::Action> {
        move_mailbox(
            true,
            self.center_column_mailbox().clone(),
            self.backend.clone(),
            self.columns.clone(),
            self.task_manager.clone(),
        );
        None
    }

    fn move_mailbox_down(&mut self) -> Option<crate::Action> {
        move_mailbox(
            false,
            self.center_column_mailbox().clone(),
            self.backend.clone(),
            self.columns.clone(),
            self.task_manager.clone(),
        );
        None
    }

    fn create_mailbox(&mut self) -> Option<crate::Action> {
        self.overlay_value = Some(OverlayValue::NewMailboxName);
        Some(crate::Action::OpenPrompt {
            description: "Mailbox name:".to_string(),
        })
    }

    fn remove_mailbox(&mut self) -> Option<crate::Action> {
        let (selected_entry, current_mailbox) = {
            let columns = self.columns.lock().unwrap();
            let current_mailbox = self.center_column_mailbox().clone();

            let selected_entry = columns
                .get(&current_mailbox)
                .and_then(|center| center.selected_entry().cloned());

            (selected_entry, current_mailbox)
        };

        if let Some(entry) = selected_entry {
            let ColumnStateEntry::Mailbox(mailbox_id) = entry else {
                warn!("You can only remove a mailbox, if you've selected it.");
                return None;
            };

            let columns = self.columns.clone();
            let backend = self.backend.clone();
            self.task_manager.spawn(async move {
                if let Err(err) = backend
                    .remove_mailboxes(&[mailbox_id.clone()], RemoveMailboxOption::Empty)
                    .await
                {
                    error!("Couldn't remove mailbox:\n{err}");
                    return;
                }

                let mut columns = columns.lock().unwrap();
                let column = columns.get_mut(&current_mailbox).unwrap();
                let pos = column.entries()
                    .iter()
                    .position(|entry| matches!(entry, ColumnStateEntry::Mailbox(other) if other == &mailbox_id));

                if let Some(pos) = pos {
                    column.entries_mut()
                        .remove(pos);
                }
            });
        }

        None
    }

    fn mail_patch_keywords(&mut self, patch: &[(MailKeyword, bool)]) -> Option<crate::Action> {
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

            let backend = self.backend.clone();
            self.task_manager.spawn(async move {
                match backend.update_mails(updates).await {
                    Ok(()) => {}
                    Err(err) => {
                        error!("Couldn't apply mail keyword to mails:\n{err}");
                    }
                }
            });

            return None;
        }

        // use current selected entry
        let selected_entry = {
            let columns = self.columns.lock().unwrap();
            columns
                .get(self.center_column_mailbox())
                .and_then(|center| center.selected_entry().cloned())
        };

        if let Some(entry) = selected_entry {
            match entry {
                ColumnStateEntry::Mailbox(_) => {}
                ColumnStateEntry::SingleMail(mail_id)
                | ColumnStateEntry::CollapsedThread(mail_id, _)
                | ColumnStateEntry::ThreadStart { mail_id, .. }
                | ColumnStateEntry::ThreadChild(mail_id, _)
                | ColumnStateEntry::ThreadEnd(mail_id, _) => {
                    let backend = self.backend.clone();
                    let patch = patch.to_owned();

                    self.task_manager.spawn(async move {
                        match backend
                            .update_mails(vec![MailUpdate {
                                id: mail_id.clone(),
                                patch_keywords: Some(patch),
                                ..Default::default()
                            }])
                            .await
                        {
                            Ok(()) => {}
                            Err(err) => {
                                error!("Couldn't apply mail keyword to mail:\n{err}");
                            }
                        }
                    });
                }
            }
        }

        None
    }
}

async fn op_init_mailbox(
    id: ParentMailboxId,
    columns: Columns,
    backend: Arc<Backend>,
) -> Result<(), jmap_client::Error> {
    let mut entries: Vec<ColumnStateEntry> = Vec::new();

    // mailbox children
    {
        let mut mailboxes: Vec<MailboxData> = backend.get_mailbox_children(id.clone()).await?;

        mailboxes.sort_by_key(|mailbox| mailbox.sort_order);

        entries.extend(
            mailboxes
                .into_iter()
                .map(|mailbox| ColumnStateEntry::Mailbox(mailbox.id)),
        );
    }

    // the first mails from the mailbox
    if let Some(parent_mailbox_id) = id.as_ref() {
        let collapsed_mails = backend
            .get_or_request_mailbox_root_mails(parent_mailbox_id)
            .await?;

        entries.extend(collapsed_mails.into_iter().map(ColumnStateEntry::from));
    }

    let created_column = ColumnState::new(id.clone(), entries);
    if let Some(first_entry) = created_column.selected_entry() {
        match first_entry {
            ColumnStateEntry::Mailbox(_) => {}
            ColumnStateEntry::SingleMail(mail_id)
            | ColumnStateEntry::CollapsedThread(mail_id, _) => {
                match backend.prefetch_mail_attachments(mail_id).await {
                    Ok(()) => {}
                    Err(err) => {
                        error!("Couldn't prefetch mail attachments:\n{err}");
                    }
                }
            }
            ColumnStateEntry::ThreadStart { .. }
            | ColumnStateEntry::ThreadChild(_, _)
            | ColumnStateEntry::ThreadEnd(_, _) => unreachable!("All threads are collapsed"),
        }
    }

    let mut guard = columns.lock().unwrap();
    guard.insert(id.clone(), created_column);

    Ok(())
}

async fn op_uncollapse_thread(
    column_mailbox: MailboxId,
    collapsed_mail_id: MailId,
    thread_id: ThreadId,
    columns: Columns,
    backend: Arc<Backend>,
) -> Result<(), jmap_client::Error> {
    let mut thread_mails = backend.get_or_request_thread_mails(&thread_id).await?;

    let mut columns = columns.lock().unwrap();
    let column = columns
        .get_mut(&Some(column_mailbox))
        .expect("Column exists");

    debug_assert!(
        thread_mails.len() >= 2,
        "Uncollapseable threads must have at least 2 mails <.<"
    );

    // according to the jmap specs: The thread saves the mails from oldest to latest,
    // but we want the newest mail to be first: So reverse it
    thread_mails.reverse();

    let thread_children_entries = {
        let (first, rest) = thread_mails.split_first().unwrap();
        let (last, inner) = rest.split_last().unwrap();

        let mut new_entries = vec![ColumnStateEntry::ThreadStart {
            mail_id: first.id.clone(),
            thread_id: thread_id.clone(),
            collapsed_mail_id: collapsed_mail_id.clone(),
        }];

        new_entries.extend(
            inner
                .iter()
                .map(|mail| ColumnStateEntry::ThreadChild(mail.id.clone(), thread_id.clone())),
        );

        new_entries.push(ColumnStateEntry::ThreadEnd(
            last.id.clone(),
            thread_id.clone(),
        ));

        new_entries
    };

    let thread_idx = column
            .entries()
            .iter()
            .position(|entry| matches!(entry, ColumnStateEntry::CollapsedThread(_, entry_thread_id) if entry_thread_id == &thread_id))
            .expect("Thread still exists in the mailbox.");

    column
        .entries_mut()
        .splice(thread_idx..(thread_idx + 1), thread_children_entries);

    Ok(())
}

fn move_mailbox(
    up: bool,
    column_mailbox: ParentMailboxId,
    backend: Arc<Backend>,
    columns: Columns,
    task_manager: Rc<TaskManager>,
) {
    // TODO: Check `self.selection` so that the user can move multiple mailboxes
    let selected_entry = {
        let columns = columns.lock().unwrap();
        columns
            .get(&column_mailbox)
            .and_then(|center| center.selected_entry().cloned())
    };

    if let Some(entry) = selected_entry {
        match entry {
            ColumnStateEntry::SingleMail(_)
            | ColumnStateEntry::CollapsedThread(_, _)
            | ColumnStateEntry::ThreadStart { .. }
            | ColumnStateEntry::ThreadChild(_, _)
            | ColumnStateEntry::ThreadEnd(_, _) => {
                warn!("This action can be only applied to mailboxes.");
            }
            ColumnStateEntry::Mailbox(mailbox_id) => {
                let (idx, last_mailbox_idx) = {
                    let columns = columns.lock().unwrap();
                    let center = columns.get(&column_mailbox).unwrap();

                    let idx = center.selected_idx().unwrap();
                    let last_mailbox_idx = center
                        .entries()
                        .iter()
                        .position(|entry| !matches!(entry, ColumnStateEntry::Mailbox(_)))
                        .unwrap_or(center.entries().len() - 1);

                    (idx, last_mailbox_idx)
                };

                let is_not_at_end_of_entries = if up { idx > 0 } else { idx < last_mailbox_idx };
                let there_are_at_least_two_mailboxes = last_mailbox_idx > 0;
                if is_not_at_end_of_entries && there_are_at_least_two_mailboxes {
                    let mailbox = backend.get_mailbox_data(&mailbox_id).unwrap();
                    let other_mailbox = {
                        let id = {
                            let columns = columns.lock().unwrap();
                            columns
                                .get(&column_mailbox)
                                .map(|center| {
                                    let other_idx = if up { idx - 1 } else { idx + 1 };
                                    center.entries()[other_idx].clone()
                                })
                                .map(|entry| {
                                    let ColumnStateEntry::Mailbox(id) = entry else {
                                        unreachable!("Only mailboxes can be above!")
                                    };
                                    id
                                })
                                .unwrap()
                        };

                        backend.get_mailbox_data(&id).unwrap()
                    };

                    let update1 = MailboxUpdate {
                        id: mailbox.id,
                        sort_order: Some(other_mailbox.sort_order),
                        ..Default::default()
                    };

                    let update2 = MailboxUpdate {
                        id: other_mailbox.id,
                        sort_order: Some(mailbox.sort_order),
                        ..Default::default()
                    };

                    task_manager.spawn(async move {
                            let updates = vec![update1.clone(), update2.clone()];
                            if let Err(err) = backend.update_mailboxes(updates).await {
                                error!("Couldn't move mailbox up:\n{err}");
                                return;
                            }

                            let mut columns = columns.lock().unwrap();
                            if let Some(column) = columns.get_mut(&column_mailbox) {
                                let entries = column.entries_mut();

                                let pos1 = entries
                                    .iter()
                                    .position(|entry| matches!(entry, ColumnStateEntry::Mailbox(id) if id == &update1.id)).unwrap();
                                let pos2 = entries
                                    .iter()
                                    .position(|entry| matches!(entry, ColumnStateEntry::Mailbox(id) if id == &update2.id)).unwrap();

                                entries.swap(pos1, pos2);

                                if up {
                                    column.state.select_previous();
                                } else {
                                    column.state.select_next();
                                }
                            }
                        })
                }
            }
        }
    }
}

async fn init_mailfs(columns: Columns, backend: Arc<Backend>) {
    match op_init_mailbox(TOP_PARENT_MAILBOX_ID, columns.clone(), backend.clone()).await {
        Ok(()) => {}
        Err(err) => {
            error!("Couldn't initialise root mailbox:\n{err}");
            return;
        }
    }

    let selected_entry = {
        let columns = columns.lock().unwrap();
        columns
            .get(&TOP_PARENT_MAILBOX_ID)
            .unwrap()
            .selected_entry()
            .cloned()
    };

    if let Some(entry) = selected_entry {
        match entry {
            ColumnStateEntry::Mailbox(id) => {
                let right_column_not_loaded = {
                    let columns = columns.lock().unwrap();
                    !columns.contains_key(&Some(id.clone()))
                };

                if right_column_not_loaded {
                    match op_init_mailbox(Some(id), columns, backend).await {
                        Ok(()) => {}
                        Err(err) => {
                            error!(
                                "Couldn't initialize mailbox (the column will be empty):\n{err}"
                            );
                        }
                    }
                }
            }
            ColumnStateEntry::SingleMail(_)
            | ColumnStateEntry::CollapsedThread(_, _)
            | ColumnStateEntry::ThreadStart { .. }
            | ColumnStateEntry::ThreadChild(_, _)
            | ColumnStateEntry::ThreadEnd(_, _) => {
                unreachable!("Root directory can't have mails")
            }
        }
    }
}

async fn create_new_mailbox(
    new_mailbox_name: String,
    column_id: ParentMailboxId,
    columns: Columns,
    backend: Arc<Backend>,
) {
    let new_mailbox = {
        let sort_order = {
            let columns = columns.lock().unwrap();
            let center = columns.get(&column_id).unwrap();
            center
                .entries()
                .iter()
                .map_while(|entry| {
                    if let ColumnStateEntry::Mailbox(id) = entry {
                        let mailbox = backend.get_mailbox_data(id).unwrap();
                        Some(mailbox)
                    } else {
                        None
                    }
                })
                .max_by_key(|mailbox| mailbox.sort_order)
                .map(|last_mailbox| {
                    (last_mailbox.sort_order + 1).next_multiple_of(NORMALIZE_SORT_ORDER_SIZE)
                })
                .unwrap_or(NORMALIZE_SORT_ORDER_SIZE)
        };

        MailboxNew {
            name: new_mailbox_name,
            sort_order: Some(sort_order),
            parent_id: column_id.clone(),
            ..Default::default()
        }
    };

    let new_mailbox_id = match backend.create_mailbox(new_mailbox.clone()).await {
        Ok(created_mailbox_id) => created_mailbox_id,
        Err(err) => {
            error!("Couldn't create mailbox:\n{err}");
            return;
        }
    };

    let mut columns = columns.lock().unwrap();
    let center = columns.get_mut(&column_id).unwrap();
    let new_pos = center
        .entries()
        .iter()
        .take_while(|entry| matches!(entry, ColumnStateEntry::Mailbox(_)))
        .position(|entry| {
            if let ColumnStateEntry::Mailbox(other_id) = entry {
                let mailbox = backend.get_mailbox_data(other_id).unwrap();
                mailbox.sort_order > new_mailbox.sort_order.unwrap()
            } else {
                false
            }
        })
        .unwrap_or(
            center
                .entries()
                .iter()
                .position(|entry| !matches!(entry, ColumnStateEntry::Mailbox(_)))
                .unwrap_or(center.entries().len()),
        );

    center
        .entries_mut()
        .insert(new_pos, ColumnStateEntry::Mailbox(new_mailbox_id));
}
