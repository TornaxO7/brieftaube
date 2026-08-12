mod column_state;
mod input_type;
mod palette_value;
// mod pending_op;
mod selection;

pub use column_state::{ColumnState, ColumnStateEntry};
pub use palette_value::PaletteValue;
pub use selection::{EntryId, SelectionType};

use super::UserAction;
use crate::{
    backend::{
        Backend, MailId, MailboxData, MailboxId, ThreadId,
        mailbox::types::{ParentMailboxId, TOP_PARENT_MAILBOX_ID},
        mails::types::{MailKeyword, MailUpdate},
    },
    task_manager::TaskManager,
    utils::ui::{
        ScreenOverlay, ScreenOverlayResult, ScreenState, keybindmanager::KeybindManager, palette,
    },
};
use input_type::InputType;
use std::{
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};
use tracing::{debug, error};

pub type Columns = Arc<Mutex<HashMap<ParentMailboxId, ColumnState>>>;

pub enum RightColumn {
    Mailbox(MailboxId),
    MailPreview(MailId),
}

pub struct Model {
    keybindings: KeybindManager<UserAction>,
    task_manager: Rc<TaskManager>,

    pub backend: Arc<Backend>,
    pub selection: HashMap<EntryId, SelectionType>,
    pub navigation_stack: Vec<ParentMailboxId>,
    pub columns: Columns,
    pub overlay: Option<ScreenOverlay<PaletteValue, InputType>>,
}

impl Model {
    pub fn new(backend: Arc<Backend>, task_manager: Rc<TaskManager>) -> Self {
        let columns = Arc::new(Mutex::new(HashMap::new()));
        let c = columns.clone();
        let b = backend.clone();

        task_manager.spawn(async move {
            op_init_mailbox(TOP_PARENT_MAILBOX_ID, c.clone(), b.clone()).await;

            let selected_entry = {
                let columns = c.lock().unwrap();
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
                            let columns = c.lock().unwrap();
                            !columns.contains_key(&Some(id.clone()))
                        };

                        if right_column_not_loaded {
                            op_init_mailbox(Some(id), c, b).await;
                        }
                    }
                    ColumnStateEntry::SingleMail(mail_id)
                    | ColumnStateEntry::CollapsedThread(mail_id, _)
                    | ColumnStateEntry::ThreadStart { mail_id, .. }
                    | ColumnStateEntry::ThreadChild(mail_id, _)
                    | ColumnStateEntry::ThreadEnd(mail_id, _) => {
                        match b.prefetch_mail_attachments(&mail_id).await {
                            Ok(()) => {}
                            Err(err) => {
                                error!("Couldn't fetch mail attachments:\n{err}");
                            }
                        }
                    }
                }
            }
        });

        Self {
            columns,
            navigation_stack: vec![TOP_PARENT_MAILBOX_ID],
            selection: HashMap::new(),
            task_manager,

            backend,
            overlay: None,
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

impl<'a> ScreenState<'a, UserAction, PaletteValue, InputType> for Model {
    fn apply_user_action(&mut self, action: UserAction) -> Option<crate::Action> {
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

            UserAction::MarkMailAsUnseen => self.mail_patch_keywords(&[(MailKeyword::Seen, false)]),
            UserAction::MarkMailAsSeen => self.mail_patch_keywords(&[(MailKeyword::Seen, true)]),
        }
    }

    fn keybinding_manager(&mut self) -> &mut KeybindManager<UserAction> {
        &mut self.keybindings
    }

    fn overlay(&mut self) -> Option<&mut ScreenOverlay<PaletteValue, InputType>> {
        self.overlay.as_mut()
    }

    fn handle_overlay_result(
        &mut self,
        result: ScreenOverlayResult<PaletteValue, InputType>,
    ) -> Option<crate::Action> {
        match result {
            ScreenOverlayResult::Palette(value) => match value {
                PaletteValue::Action(action) => {
                    self.overlay = None;
                    return self.apply_user_action(action);
                }
            },
            ScreenOverlayResult::Cancel => self.overlay = None,
            ScreenOverlayResult::Input { value: _, typ } => match typ {
                InputType::NewMailboxName => {
                    // if let Some(center) = self.get_center_column() {
                    //     let parent_id = center.mailbox().clone();
                    //     let _new = MailboxNew {
                    //         name: value,
                    //         parent_id,
                    //         ..Default::default()
                    //     };
                    // }
                    // todo!("update state");
                    todo!()
                }
            },
        };

        None
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
                        op_init_mailbox(Some(id), columns, backend).await;
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
                    mail.attachments.is_none()
                };

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
}

/// Action implementations
impl Model {
    fn quit(&self) -> Option<crate::Action> {
        Some(crate::Action::Quit)
    }

    fn open_command_palette(&mut self) -> Option<crate::Action> {
        self.overlay = Some(ScreenOverlay::Palette(palette::State::new(
            UserAction::palette_options(),
        )));

        None
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
                        columns
                            .get(self.center_column_mailbox())
                            .and_then(|column| column.selected_entry().cloned())
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
                        op_uncollapse_thread(column_mailbox, mail_id, thread_id, columns, backend)
                            .await;
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
        // Some(crate::Action::OpenLogViewer)
        todo!()
    }

    fn select_entry(&mut self) -> Option<crate::Action> {
        let columns = self.columns.lock().unwrap();
        if let Some(column) = columns.get(self.center_column_mailbox()) {
            if let Some(entry) = column.selected_entry() {
                let id = EntryId::from(entry);

                if self.selection.remove(&id).is_none() {
                    self.selection.insert(id, SelectionType::Selected);
                }

                self.navigate_down();
            }
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
        // TODO: Check `self.selection` so that the user can move multiple mailboxes
        // if let Some(center) = self.get_center_column() {
        //     if let Some(entry) = center.selected_entry() {
        //         match entry {
        //             ColumnStateEntry::SingleMail(_)
        //             | ColumnStateEntry::CollapsedThread(_, _)
        //             | ColumnStateEntry::ThreadStart { .. }
        //             | ColumnStateEntry::ThreadChild(_, _)
        //             | ColumnStateEntry::ThreadEnd(_, _) => {
        //                 warn!("This action can be only applied to mailboxes.");
        //             }
        //             ColumnStateEntry::Mailbox(mailbox_id) => {}
        //         }
        //     }
        // }
        todo!();
    }

    fn move_mailbox_down(&mut self) -> Option<crate::Action> {
        todo!();
    }

    fn create_mailbox(&mut self) -> Option<crate::Action> {
        self.overlay = Some(ScreenOverlay::input(
            "Create mailbox:",
            InputType::NewMailboxName,
        ));

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

/// Methods for the pending ops
// impl Model {

//     fn op_mail_attachments(&mut self, id: &MailId) -> Result<(), error::BackendNotReady> {
//         self.backend.prefetch_mail_attachments(id);
//         Ok(())
//     }

//     fn op_move_mailbox_up(&mut self, data: &OpMoveMailboxUp) -> Result<(), error::BackendNotReady> {
//         let Some(column) = self.columns.get(&data.parent) else {
//             warn!(
//                 "Can't move mailbox up. Column for mailbox with the id '{:?}' doesn't exist (anymore?).",
//                 data.parent
//             );
//             return Ok(());
//         };

//         let mailboxes: Vec<MailboxData> = column
//             .entries()
//             .iter()
//             .cloned()
//             .map_while(|entry| match entry {
//                 ColumnStateEntry::Mailbox(id) => Some(id),
//                 _ => None,
//             })
//             .map(|id| self.backend.get_mailbox_data(&id).unwrap())
//             .collect();

//         let idx_of_mailbox_to_move = mailboxes
//             .iter()
//             .position(|mailbox| mailbox.id == data.mailbox)
//             .unwrap();

//         if idx_of_mailbox_to_move == 0 {
//             return Ok(());
//         }

//         todo!()
//     }
// }

async fn op_init_mailbox(id: ParentMailboxId, columns: Columns, backend: Arc<Backend>) {
    let mut entries: Vec<ColumnStateEntry> = Vec::new();

    // mailbox children
    {
        let mut mailboxes: Vec<MailboxData> = backend
            .get_mailbox_children(id.clone())
            .await
            .expect("Children come successfully. No retry-mechanism implemented yet :(");

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
            .await
            .expect("No error handling yet");

        entries.extend(collapsed_mails.into_iter().map(ColumnStateEntry::from));
    }

    let created_column = ColumnState::new(id.clone(), entries);
    let mut guard = columns.lock().unwrap();
    guard.insert(id.clone(), created_column);
}

async fn op_uncollapse_thread(
    column_mailbox: MailboxId,
    collapsed_mail_id: MailId,
    thread_id: ThreadId,
    columns: Columns,
    backend: Arc<Backend>,
) {
    let mut thread_mails = backend
        .get_or_request_thread_mails(&thread_id)
        .await
        .expect("No error handling yet");

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
}
