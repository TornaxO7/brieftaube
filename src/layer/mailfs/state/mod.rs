mod selection;

use super::UserAction;
use crate::{
    backend::{
        Backend, LoadingRole,
        types::{MailId, MailboxId, ParentMailboxId, TOP_PARENT_MAILBOX_ID, ThreadId},
    },
    layer::{
        LayerCore, LayerState,
        mailfs::backend::{MailfsMessage, MailfsSnapshot},
        utils::keybindmanager::KeybindManager,
    },
    task_manager::TaskManager,
};
use futures::stream;
use futures::stream::StreamExt;
use ratatui::{Frame, layout::Rect, widgets::ListState};
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    str::FromStr,
    sync::{Arc, Mutex, mpsc},
};
use throbber_widgets_tui::ThrobberState;
use tokio::sync::watch;
use tracing::{debug, error, warn};

pub use selection::{Selection, SelectionType};

enum OverlayValue {
    Action,
    NewMailboxName,
}

pub enum RightColumn {
    Mailbox(MailboxId),
    MailPreview(MailId),
}

pub struct State {
    keybindings: KeybindManager<UserAction>,
    overlay_value: Option<OverlayValue>,

    tx: mpsc::Sender<MailfsMessage>,
    snapshot: watch::Receiver<MailfsSnapshot>,

    throbber: ThrobberState,
    // pub selection: HashMap<ColumnEntry, Selection>,
    account_column: ListState,
    mailbox_stack: Vec<ParentMailboxId>,
    mailboxes: HashMap<ParentMailboxId, ListState>,
}

impl State {
    pub fn new(snapshot: watch::Receiver<MailfsSnapshot>, tx: mpsc::Sender<MailfsMessage>) -> Self {
        let columns = Arc::new(Mutex::new(HashMap::new()));

        Self {
            overlay_value: None,
            throbber: ThrobberState::default(),
            account_column: ListState::new(),
            mailbox_stack: vec![],
            mailboxes: HashMap::new(),
            // selection: HashMap::new(),
            tx,
            snapshot,
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

impl LayerCore for State {
    fn handle_event(&mut self, event: crossterm::event::Event) -> Option<crate::Action> {
        todo!()
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        todo!()
    }
}

impl LayerState<UserAction> for State {
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

impl LayerModelDefaultHandleEvent<UserAction> for State {
    fn keybinding_manager(&mut self) -> &mut KeybindManager<UserAction> {
        &mut self.keybindings
    }
}

/// Helper functions
impl<'a> State {
    pub fn left_column_mailbox(&self) -> Option<&ParentMailboxId> {
        (self.mailbox_stack.len().checked_sub(2)).map(|idx| &self.mailbox_stack[idx])
    }

    pub fn center_column_mailbox(&self) -> &ParentMailboxId {
        self.mailbox_stack.last().unwrap()
    }

    pub fn right_column(
        &self,
        columns: &HashMap<ParentMailboxId, ColumnState>,
    ) -> Option<RightColumn> {
        let center = self.center_column_mailbox();

        columns
            .get(&center)
            .and_then(|center| center.loaded()?.selected_entry())
            .map(|selected| match selected.clone() {
                ColumnEntry::Mailbox(id) => RightColumn::Mailbox(id),
                ColumnEntry::Mail(id) => RightColumn::MailPreview(id.mail_id),
            })
    }

    fn load_right_column_for(&self, entry: ColumnEntry) {
        match entry {
            ColumnEntry::Mailbox(id) => {
                let id = id.clone();
                let columns = self.columns.clone();
                let backend = self.backend.clone();

                self.task_manager.spawn(async move {
                    match init_mailbox(Some(id), columns, backend).await {
                        Ok(()) => {}
                        Err(err) => {
                            error!("Couldn't initialize mailbox for right column:\n{err}");
                            return;
                        }
                    }
                });
            }
            ColumnEntry::Mail(mail) => {
                let id = mail.mail_id;
                let backend = self.backend.clone();
                self.task_manager.spawn(async move {
                    match backend.prefetch_mail_attachments(&id).await {
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

/// Action implementations
impl State {
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
            let state = columns.get_mut(&self.center_column_mailbox())?;
            let center = state.loaded_mut()?;
            let pos = center.state.selected();
            let new_pos = pos.map(|old_pos| (old_pos + 1).min(center.entries().len() - 1));
            center.state.select(new_pos);
            center.selected_entry().cloned()?
        };

        self.load_right_column_for(selected_entry);
        None
    }

    fn navigate_up(&self) -> Option<crate::Action> {
        let selected_entry = {
            let mut columns = self.columns.lock().unwrap();
            let state = columns.get_mut(self.center_column_mailbox())?;
            let center = state.loaded_mut()?;
            center.state.select_previous();
            center.selected_entry().cloned()?
        };

        self.load_right_column_for(selected_entry);
        None
    }

    fn navigate_to_top(&mut self) -> Option<crate::Action> {
        let selected_entry = {
            let mut columns = self.columns.lock().unwrap();
            let state = columns.get_mut(self.center_column_mailbox())?;
            let center = state.loaded_mut()?;
            center.state.select_first();
            center.selected_entry().cloned()?
        };

        self.load_right_column_for(selected_entry);
        None
    }

    fn navigate_to_bottom(&mut self) -> Option<crate::Action> {
        let selected_entry = {
            let mut columns = self.columns.lock().unwrap();
            let state = columns.get_mut(self.center_column_mailbox())?;
            let center = state.loaded_mut()?;
            if center.entries().is_empty() {
                center.state.select(None);
            } else {
                let len = center.entries().len();
                center.state.select(Some(len - 1));
            }

            center.selected_entry().cloned()?
        };

        self.load_right_column_for(selected_entry);
        None
    }

    fn navigate_right(&mut self) -> Option<crate::Action> {
        let selected_entry = {
            let columns = self.columns.lock().unwrap();
            let center = columns.get(self.center_column_mailbox())?;
            center.loaded()?.selected_entry().cloned()?
        };

        match selected_entry {
            ColumnEntry::Mailbox(id) => {
                self.mailbox_stack.push(Some(id));

                let selected_entry = {
                    let columns = self.columns.lock().unwrap();

                    columns
                        .get(self.center_column_mailbox())?
                        .loaded()?
                        .selected_entry()
                        .cloned()?
                };

                self.load_right_column_for(selected_entry);
            }
            ColumnEntry::Mail(mail) => match mail.thread_role {
                ThreadRole::Single
                | ThreadRole::ThreadStart
                | ThreadRole::ThreadChild
                | ThreadRole::ThreadEnd => {
                    return Some(crate::Action::OpenMailViewer(mail.mail_id));
                }
                ThreadRole::Collapsed => {
                    let column_mailbox = self
                        .center_column_mailbox()
                        .clone()
                        .expect("Is not root mailbox");
                    let columns = self.columns.clone();
                    let backend = self.backend.clone();

                    self.task_manager.spawn(async move {
                        match op_uncollapse_thread(column_mailbox, mail.thread_id, columns, backend)
                            .await
                        {
                            Ok(()) => {}
                            Err(err) => {
                                error!("Can't uncollapse thread:\n{err}");
                            }
                        }
                    });
                }
            },
        }

        None
    }

    fn navigate_left(&mut self) -> Option<crate::Action> {
        let selected_entry = {
            let mut columns = self.columns.lock().unwrap();
            let center = columns.get_mut(self.center_column_mailbox())?;
            center.loaded()?.selected_entry().cloned()?
        };

        match selected_entry {
            ColumnEntry::Mailbox(_) => self.navigate_to_parent(),
            ColumnEntry::Mail(mail_entry) => match mail_entry.thread_role {
                ThreadRole::Single | ThreadRole::Collapsed => self.navigate_to_parent(),
                ThreadRole::ThreadStart | ThreadRole::ThreadChild | ThreadRole::ThreadEnd => {
                    let mut columns = self.columns.lock().unwrap();
                    let column = columns
                        .get_mut(self.center_column_mailbox())
                        .expect("Left column should be there?!")
                        .loaded_mut()
                        .expect("Left columns should be already loaded?!");

                    let (start_pos, new_entry) = column
                        .entries()
                        .iter()
                        .cloned()
                        .enumerate()
                        .find_map(|(idx, entry)| {
                            if let ColumnEntry::ThreadStart {
                                thread_id: entry_thread_id,
                                collapsed_mail_id,
                                ..
                            } = entry
                            {
                                if &entry_thread_id == thread_id {
                                    Some((
                                        idx,
                                        ColumnEntry::CollapsedThread(
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

                    let end_pos = column.entries().iter().position(|entry| matches!(entry, ColumnEntry::ThreadEnd(_, entry_thread_id) if entry_thread_id == thread_id))
                            .expect("Same as in the previous `.expect`.");

                    column
                        .entries_mut()
                        .splice(start_pos..=end_pos, [new_entry]);

                    column.state.select(Some(start_pos));

                    None
                }
            },
        }
    }

    fn navigate_to_parent(&mut self) -> Option<crate::Action> {
        if self.mailbox_stack.len() > 1 {
            self.mailbox_stack.pop();
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
                .loaded()?
                .selected_entry()
                .cloned()
        };

        if let Some(entry) = selected_entry {
            if self.selection.remove(&entry).is_none() {
                self.selection.insert(
                    entry,
                    Selection {
                        mailbox: self.center_column_mailbox().clone(),
                        ty: SelectionType::Selected,
                    },
                );
            }

            self.navigate_down();
        }

        None
    }

    fn cut_selected_entries(&mut self) -> Option<crate::Action> {
        if self.selection.is_empty() {
            let columns = self.columns.lock().unwrap();
            let entry = columns
                .get(self.center_column_mailbox())
                .loaded()?
                .selected_entry()?
                .clone();

            self.selection.insert(
                entry,
                Selection {
                    mailbox: self.center_column_mailbox().clone(),
                    ty: SelectionType::Cut,
                },
            );
        } else {
            for (_id, selection) in self.selection.iter_mut() {
                selection.ty = SelectionType::Cut;
            }
        }

        None
    }

    fn paste_selected_entries(&mut self) -> Option<crate::Action> {
        let center_mailbox = self.center_column_mailbox().clone();
        todo!();

        // TODO: create batch request
        // for (entry, selection) in self.selection.drain() {
        //     match entry {
        //         ColumnEntry::Mailbox(mailbox_id) => match selection.ty {
        //             SelectionType::Selected => {
        //                 warn!("You can't copy mailboxes.");
        //             }
        //             SelectionType::Cut => {
        //                 let columns = self.columns.clone();
        //                 let backend = self.backend.clone();

        //                 let source_mailbox = selection.mailbox.clone();
        //                 let destination_mailbox = center_mailbox.clone();

        //                 let update = MailboxUpdate {
        //                     id: mailbox_id.clone(),
        //                     parent_id: Some(center_mailbox.clone()),
        //                     ..Default::default()
        //                 };

        //                 self.task_manager.spawn(async move {
        //                     if let Err(err) = backend.update_mailboxes(vec![update]).await {
        //                         error!("Couldn't move mailbox: {err}");
        //                     }

        //                     let mut columns = columns.lock().unwrap();

        //                     // remove from old parent
        //                     columns.entry(source_mailbox).and_modify(|state| {
        //                         let column = state.loaded_mut().unwrap();
        //                         column.remove_entry(ColumnEntryDiff::Mailbox(mailbox_id), backend);
        //                     });

        //                     // add to new column
        //                     columns
        //                         .entry(destination_mailbox.clone())
        //                         .and_modify(|state| {
        //                             let column = state.loaded_mut().unwrap();
        //                             column.add_entry(entry, backend.clone());
        //                         });
        //                 });
        //             }
        //         },
        //         ColumnEntry::Mail(mail_id) => match selection.ty {
        //             SelectionType::Selected => {
        //                 let Some(center_mailbox) = center_mailbox.clone() else {
        //                     warn!("You can't put mails into the root mailbox.");
        //                     continue;
        //                 };

        //                 let columns = self.columns.clone();
        //                 let backend = self.backend.clone();
        //                 let update = MailUpdate {
        //                     id: mail_id.clone(),
        //                     mailbox_ids: Some(vec![(center_mailbox.clone(), true)]),
        //                     ..Default::default()
        //                 };

        //                 self.task_manager.spawn(async move {
        //                     if let Err(err) = backend.update_mails(vec![update]).await {
        //                         warn!("Couldn't add mailbox to mail: {err}");
        //                         return;
        //                     }

        //                     let mut columns = columns.lock().unwrap();

        //                     columns
        //                         .entry(Some(center_mailbox.clone()))
        //                         .and_modify(|state| {
        //                             let column = state.loaded_mut().unwrap();
        //                             todo!()
        //                             // column.add_mail(&mail_id, backend.clone());
        //                         });
        //                 });
        //             }
        //             SelectionType::Cut => {
        //                 todo!()
        //             }
        //         },
        //     }
        // }

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
        let selected_entry = {
            let columns = self.columns.lock().unwrap();

            let selected_entry = columns
                .get(&self.center_column_mailbox())
                .loaded()?
                .selected_entry()
                .cloned();

            selected_entry
        };

        if let Some(entry) = selected_entry {
            let ColumnEntry::Mailbox(mailbox_id) = entry else {
                warn!("You can only remove a mailbox, if you've selected it.");
                return None;
            };

            let current_mailbox = self.center_column_mailbox().clone();
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
                let column = columns.get_mut(&current_mailbox).loaded_mut().unwrap();

                todo!()
                // column.remove_entry(Columnn)
            });
        }

        None
    }

    fn mail_patch_keywords(&mut self, patch: &[(MailKeyword, bool)]) -> Option<crate::Action> {
        if !self.selection.is_empty() {
            let mut updates = Vec::with_capacity(self.selection.len());

            for (entry, selection) in self.selection.drain() {
                if selection.ty == SelectionType::Selected {
                    match entry {
                        // SelectedEntry::Mail(id) => updates.push(MailUpdate {
                        //     id,
                        //     patch_keywords: Some(patch.to_vec()),
                        //     ..Default::default()
                        // }),
                        // SelectedEntry::Mailbox(_) => {}
                        ColumnEntry::Mailbox(mailbox_id) => todo!(),
                        ColumnEntry::SingleMail(mail_id) => todo!(),
                        ColumnEntry::CollapsedThread(mail_id, thread_id) => todo!(),
                        ColumnEntry::ThreadStart {
                            mail_id,
                            thread_id,
                            collapsed_mail_id,
                        } => todo!(),
                        ColumnEntry::ThreadChild(mail_id, thread_id) => todo!(),
                        ColumnEntry::ThreadEnd(mail_id, thread_id) => todo!(),
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
                .loaded()?
                .selected_entry()
                .cloned()
        };

        if let Some(entry) = selected_entry {
            match entry {
                ColumnEntry::Mailbox(_) => {}
                ColumnEntry::SingleMail(mail_id)
                | ColumnEntry::CollapsedThread(mail_id, _)
                | ColumnEntry::ThreadStart { mail_id, .. }
                | ColumnEntry::ThreadChild(mail_id, _)
                | ColumnEntry::ThreadEnd(mail_id, _) => {
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
            None
        }
    }
}

async fn init_mailbox(
    id: ParentMailboxId,
    columns: Columns,
    backend: Arc<Backend>,
) -> Result<(), jmap_client::Error> {
    let role = {
        let mut columns = columns.lock().unwrap();
        let state = columns.entry(id.clone()).or_insert(ColumnState::NotLoaded);

        match state {
            ColumnState::NotLoaded => {
                let (tx, rx) = watch::channel(());
                *state = ColumnState::loading(rx);
                LoadingRole::Request(tx)
            }
            ColumnState::Loading { notifier, .. } => LoadingRole::Wait(notifier.clone()),
            ColumnState::Loaded(_) => return Ok(()),
        }
    };

    match role {
        LoadingRole::Wait(mut receiver) => {
            receiver.changed().await.unwrap();
            Ok(())
        }
        LoadingRole::Request(sender) => {
            let mut entries: Vec<ColumnEntry> = Vec::new();

            // mailbox children
            {
                let mut mailboxes = {
                    let ids = backend.get_mailbox_children(id.clone()).await?;
                    backend.get_mailboxes(&ids)
                };

                mailboxes.sort_by(|a, b| {
                    if a.sort_order == b.sort_order {
                        a.name.cmp(&b.name)
                    } else {
                        a.sort_order.cmp(&b.sort_order)
                    }
                });

                let all_have_unique_sort_order = {
                    let mut used_sort_orders = HashSet::new();
                    for mailbox in mailboxes.iter() {
                        used_sort_orders.insert(mailbox.sort_order);
                    }

                    used_sort_orders.len() == mailboxes.len()
                };

                // normalize sort order (if possible)
                if !all_have_unique_sort_order {
                    let ids = mailboxes.iter().map(|mailbox| &mailbox.id);
                    let updates: Vec<MailboxUpdate> = ids
                        .enumerate()
                        .map(|(idx, id)| MailboxUpdate {
                            id: id.clone(),
                            sort_order: Some(idx as u32),
                            ..Default::default()
                        })
                        .collect();

                    match backend.update_mailboxes(updates).await {
                        Ok(()) => {
                            for (idx, mailbox) in mailboxes.iter_mut().enumerate() {
                                mailbox.sort_order = idx as u32;
                            }
                        }
                        Err(err) => {
                            warn!("Couldn't save current sort order to server: {err}");
                        }
                    }
                }

                entries.extend(
                    mailboxes
                        .into_iter()
                        .map(|mailbox| ColumnEntry::Mailbox(mailbox.id)),
                );
            }

            // the first mails from the mailbox
            if let Some(parent_mailbox_id) = id.as_ref() {
                let root_mails = backend.get_mailbox_root_mails(parent_mailbox_id).await?;
                let mail_entries: Vec<ColumnEntry> = stream::iter(root_mails)
                    .map(|id| MailEntry::new(id, &backend))
                    .buffered(BATCH_SIZE)
                    .map(ColumnEntry::Mail)
                    .collect()
                    .await;

                entries.extend(mail_entries);
            }

            let created_column = Column::new(id.clone(), entries);
            let mut columns = columns.lock().unwrap();
            let state = columns.get_mut(&id).unwrap();
            *state = ColumnState::Loaded(created_column);

            let _ = sender.send(());
            Ok(())
        }
    }
}

#[derive(thiserror::Error, Debug)]
enum UncollapseThreadError {
    #[error("Thread doesn't exist in column anymore. Abort uncollapsing...")]
    ThreadGone,

    #[error(transparent)]
    Jmap(#[from] jmap_client::Error),
}

async fn op_uncollapse_thread(
    column_mailbox: MailboxId,
    thread_id: ThreadId,
    columns: Columns,
    backend: Arc<Backend>,
) -> Result<(), UncollapseThreadError> {
    let mut thread_mails = backend.get_thread(&thread_id);

    let mut columns = columns.lock().unwrap();
    let column = columns
        .get_mut(&Some(column_mailbox))
        .expect("Column exists")
        .loaded_mut()
        .expect("Loaded. How else can we uncollapse a thread?");

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

        let mut new_entries = vec![ColumnEntry::Mail(MailEntry {
            mail_id: first.clone(),
            thread_id: thread_id.clone(),
            thread_role: ThreadRole::ThreadStart,
        })];

        new_entries.extend(inner.iter().map(|mail| {
            ColumnEntry::Mail(MailEntry {
                mail_id: mail.clone(),
                thread_id: thread_id.clone(),
                thread_role: ThreadRole::ThreadChild,
            })
        }));

        new_entries.push(ColumnEntry::Mail(MailEntry {
            mail_id: last.clone(),
            thread_id: thread_id.clone(),
            thread_role: ThreadRole::ThreadEnd,
        }));

        new_entries
    };

    let insert_pos = column
        .entries()
        .iter()
        .position(|entry| matches!(entry, ColumnEntry::Mail(MailEntry {thread_id: entry_thread_id, ..}) if entry_thread_id == &thread_id))
        .ok_or(UncollapseThreadError::ThreadGone)?;

    column
        .entries_mut()
        .splice(insert_pos..(insert_pos + 1), thread_children_entries);

    Ok(())
}

fn move_mailbox(
    up: bool,
    column_mailbox: ParentMailboxId,
    backend: Arc<Backend>,
    columns: Arc<Mutex<Columns>>,
    task_manager: Rc<TaskManager>,
) {
    // TODO: Check `self.selection` so that the user can move multiple mailboxes
    let selected_entry = {
        let columns = columns.lock().unwrap();
        columns
            .get(&column_mailbox)
            .and_then(|state| state.loaded()?.selected_entry().cloned())
    };

    if let Some(entry) = selected_entry {
        match entry {
            ColumnEntry::SingleMail(_)
            | ColumnEntry::CollapsedThread(_, _)
            | ColumnEntry::ThreadStart { .. }
            | ColumnEntry::ThreadChild(_, _)
            | ColumnEntry::ThreadEnd(_, _) => {
                warn!("This action can be only applied to mailboxes.");
            }
            ColumnEntry::Mailbox(mailbox_id) => {
                let (idx, last_mailbox_idx) = {
                    let mut columns = columns.lock().unwrap();
                    let center = columns
                        .get_mut(&column_mailbox)
                        .unwrap()
                        .loaded_mut()
                        .unwrap();

                    let idx = center.selected_idx().unwrap();
                    let last_mailbox_idx = center
                        .entries()
                        .iter()
                        .position(|entry| !matches!(entry, ColumnEntry::Mailbox(_)))
                        .unwrap_or(center.entries().len() - 1);

                    (idx, last_mailbox_idx)
                };

                let is_not_at_end_of_entries = if up { idx > 0 } else { idx < last_mailbox_idx };
                let there_are_at_least_two_mailboxes = last_mailbox_idx > 0;
                if is_not_at_end_of_entries && there_are_at_least_two_mailboxes {
                    let mailbox = backend.get_mailbox(&mailbox_id).unwrap();
                    let other_mailbox = {
                        let id = {
                            let columns = columns.lock().unwrap();
                            columns
                                .get(&column_mailbox)
                                .map(|state| {
                                    let center = state.loaded().unwrap();
                                    let other_idx = if up { idx - 1 } else { idx + 1 };
                                    center.entries()[other_idx].clone()
                                })
                                .map(|entry| {
                                    let ColumnEntry::Mailbox(id) = entry else {
                                        unreachable!("Only mailboxes can be above!")
                                    };
                                    id
                                })
                                .unwrap()
                        };

                        backend.get_mailbox(&id).unwrap()
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
                            if let Some(state) = columns.get_mut(&column_mailbox) {
                                let column = state.loaded_mut().unwrap();
                                let entries = column.entries_mut();

                                let pos1 = entries
                                    .iter()
                                    .position(|entry| matches!(entry, ColumnEntry::Mailbox(id) if id == &update1.id)).unwrap();
                                let pos2 = entries
                                    .iter()
                                    .position(|entry| matches!(entry, ColumnEntry::Mailbox(id) if id == &update2.id)).unwrap();

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
    match init_mailbox(TOP_PARENT_MAILBOX_ID, columns.clone(), backend.clone()).await {
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
            .loaded()
            .expect("`init_mailbox` should've loaded it")
            .selected_entry()
            .cloned()
    };

    if let Some(entry) = selected_entry {
        match entry {
            ColumnEntry::Mailbox(id) => match init_mailbox(Some(id), columns, backend).await {
                Ok(()) => {}
                Err(err) => {
                    error!("Couldn't initialize mailbox (the column will be empty):\n{err}");
                }
            },
            ColumnEntry::Mail(mail) => match backend.prefetch_mail_attachments(&mail.mail_id).await
            {
                Ok(()) => {}
                Err(err) => {
                    error!("Couldn't prefetch mail attachments:\n{err}");
                }
            },
        }
    }
}

async fn create_new_mailbox(
    new_mailbox_name: String,
    column_id: ParentMailboxId,
    columns: Arc<Mutex<Columns>>,
    backend: Arc<Backend>,
) {
    let new_mailbox = {
        let sort_order = {
            let mut columns = columns.lock().unwrap();
            let center = columns.get_mut(&column_id).loaded_mut().unwrap();
            center
                .entries()
                .iter()
                .map_while(|entry| {
                    if let ColumnEntry::Mailbox(id) = entry {
                        let mailbox = backend.get_mailbox(id);
                        Some(mailbox)
                    } else {
                        None
                    }
                })
                .max_by_key(|mailbox| mailbox.sort_order)
                .map(|last_mailbox| last_mailbox.sort_order + 1)
                .unwrap_or(0)
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
    let center = columns.get_mut(&column_id).loaded_mut().unwrap();
    let new_pos = center
        .entries()
        .iter()
        .take_while(|entry| matches!(entry, ColumnEntry::Mailbox(_)))
        .position(|entry| {
            if let ColumnEntry::Mailbox(other_id) = entry {
                let mailbox = backend.get_mailbox(other_id);
                mailbox.sort_order > new_mailbox.sort_order.unwrap()
            } else {
                false
            }
        })
        .unwrap_or(
            center
                .entries()
                .iter()
                .position(|entry| !matches!(entry, ColumnEntry::Mailbox(_)))
                .unwrap_or(center.entries().len()),
        );

    center
        .entries_mut()
        .insert(new_pos, ColumnEntry::Mailbox(new_mailbox_id));
}
