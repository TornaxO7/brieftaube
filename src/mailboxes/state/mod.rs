use super::Action;
use crate::{
    backend::mailbox::{
        MailboxBackend,
        types::{Children, Entry, MailboxData, MailboxId, MailboxNew, MailboxUpdate, SortOrder},
    },
    config::Config,
    mailboxes::widget::types::{ColumnData, MailboxDisplay},
    utils::ui::{
        ScreenOverlay, ScreenOverlayResult, ScreenState, input, keybindmanager::KeybindManager,
        palette,
    },
};
use ratatui::widgets::TableState;
use std::{collections::HashMap, rc::Rc};
use tracing::{error, trace};

const NEW_SORT_ORDER_SIZE: u32 = 32;

#[derive(Debug, Clone)]
pub enum PaletteValue {
    Action(Action),
}

#[derive(Debug, Clone)]
pub enum InputType {
    SortOrder(MailboxId),
    NewMailboxName { parent: Option<MailboxId> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionType {
    Selected,
    Cut,
}

#[derive(Clone)]
struct FocusCtx {
    mailbox: Option<MailboxId>,
    state: TableState,
}

impl FocusCtx {
    pub fn new(mailbox: Option<MailboxId>) -> Self {
        Self {
            mailbox,
            state: TableState::new().with_selected(Some(0)),
        }
    }
}

pub struct State {
    app_actions: Vec<crate::Action>,
    keybindings: KeybindManager<Action>,
    overlay: Option<ScreenOverlay<PaletteValue, InputType>>,

    backend: Rc<MailboxBackend>,
    focus: Vec<FocusCtx>,
    config: Rc<Config>,

    is_in_select_mode: bool,
    selected: HashMap<MailboxId, SelectionType>,
}

impl State {
    pub fn new(backend: Rc<MailboxBackend>, config: Rc<Config>) -> Self {
        backend.init();

        Self {
            app_actions: vec![],
            backend,
            config,

            overlay: None,
            selected: HashMap::new(),
            is_in_select_mode: false,
            focus: vec![FocusCtx::new(None)],

            keybindings: KeybindManager::new(HashMap::from([
                ("q", Action::Quit),
                ("j", Action::NavigateToMailboxBelow),
                ("k", Action::NavigateToMailboxAbove),
                // ("n", super::Action::OpenComposer),
                (":", Action::OpenCommandPalette),
                ("<CR>", Action::ActivateSelectedEntry),
                (" ", Action::ToggleMailbox),
                ("l", Action::ActivateSelectedEntry),
                ("h", Action::GoBack),
                ("<C-l>", Action::OpenLogs),
                ("gg", Action::NavigateToTop),
                ("ge", Action::NavigateToBottom),
                ("v", Action::EnterSelectMode),
                ("<ESC>", Action::LeaveSelectMode),
                ("yx", Action::CutSelection),
                ("p", Action::PasteSelection),
            ])),
        }
    }
}

impl ScreenState<Action, PaletteValue, InputType> for State {
    fn apply_action(&mut self, action: Action) {
        trace!("Action: {:?}", action);
        match action {
            Action::Quit => self.app_actions.push(crate::Action::Quit),
            Action::OpenCommandPalette => {
                self.overlay = Some(ScreenOverlay::Palette(palette::State::new(
                    super::action::palette_options(),
                )))
            }
            Action::OpenLogs => {
                self.app_actions.push(crate::Action::OpenLogViewer);
            }
            Action::NavigateToMailboxBelow => self.navigate_to_mailbox_below(),
            Action::NavigateToMailboxAbove => self.navigate_to_mailbox_above(),
            Action::NavigateToTop => self.navigate_to_top(),
            Action::NavigateToBottom => self.navigate_to_bottom(),
            Action::ActivateSelectedEntry => self.activate_selected_entry(),
            Action::ToggleMailbox => self.toggle_mailbox(),
            Action::EnterSelectMode => self.is_in_select_mode = true,
            Action::LeaveSelectMode => self.is_in_select_mode = false,
            Action::DiscardSelection => self.selected.clear(),
            Action::CutSelection => self.cut_selection(),
            Action::PasteSelection => self.paste_selection(),
            Action::GoBack => self.go_back(),
            Action::SetSortOrder => self.set_sort_order(),
            Action::MoveMailboxUp => self.move_mailbox_up(),
            Action::MoveMailboxDown => self.move_mailbox_down(),
            Action::NormalizeSortOrder => self.normalize_sort_order(),
            Action::CreateMailbox => self.create_mailbox(),
            Action::RemoveSelectedMailboxes => self.remove_selected_mailboxes(),
            Action::RenameSelectedMailboxes => self.rename_selected_mailboxes(),
        }
    }

    fn get_app_actions(&mut self) -> std::vec::Drain<'_, crate::Action> {
        self.app_actions.drain(..)
    }

    fn keybinding_manager(&mut self) -> &mut KeybindManager<Action> {
        &mut self.keybindings
    }

    fn overlay(&mut self) -> Option<&mut ScreenOverlay<PaletteValue, InputType>> {
        self.overlay.as_mut()
    }

    fn handle_overlay_result(&mut self, result: ScreenOverlayResult<PaletteValue, InputType>) {
        self.overlay = None;

        match result {
            ScreenOverlayResult::Palette(value) => match value {
                PaletteValue::Action(action) => self.apply_action(action),
            },
            ScreenOverlayResult::Input { value, typ } => match typ {
                InputType::SortOrder(id) => match value.parse::<u32>() {
                    Ok(new_order) => {
                        let update = MailboxUpdate {
                            id,
                            sort_order: Some(new_order),
                            ..Default::default()
                        };
                        self.backend.update_mailboxes(vec![update]);
                    }
                    Err(err) => {
                        error!(
                            "Can't set sort order: {}' isn't a 32-bit unsigned integer: {}",
                            value, err
                        )
                    }
                },
                InputType::NewMailboxName { parent } => {
                    let new = MailboxNew {
                        name: value,
                        parent_id: parent,
                        ..Default::default()
                    };

                    self.backend.create_mailboxes(vec![new]);
                }
            },
            ScreenOverlayResult::Cancel => {}
        }
    }
}

impl State {
    fn navigate_to_mailbox_above(&mut self) {
        let focus = self.focus.last_mut().unwrap();
        let Some(children) = self.backend.get_children_ids(&focus.mailbox) else {
            return;
        };

        if self.is_in_select_mode {
            if let Some(idx) = focus.state.selected() {
                match &children[idx] {
                    Entry::This => {}
                    Entry::Child(id) => {
                        self.selected.insert(id.clone(), SelectionType::Selected);
                    }
                }
            }
        }

        focus.state.select_previous();
    }

    fn navigate_to_mailbox_below(&mut self) {
        let focus = self.focus.last_mut().unwrap();
        let Some(children) = self.backend.get_children_ids(&focus.mailbox) else {
            return;
        };

        if self.is_in_select_mode {
            if let Some(idx) = focus.state.selected() {
                match &children[idx] {
                    Entry::This => {}
                    Entry::Child(id) => {
                        self.selected.insert(id.clone(), SelectionType::Selected);
                    }
                }
            }
        }

        focus.state.select_next();
    }

    fn navigate_to_top(&mut self) {
        let focus = self.focus.last_mut().unwrap();
        focus.state.select_first();
    }

    fn navigate_to_bottom(&mut self) {
        let focus = self.focus.last_mut().unwrap();
        focus.state.select_last();
    }

    fn activate_selected_entry(&mut self) {
        let focus = self.focus.last_mut().unwrap();
        let Some(idx) = focus.state.selected() else {
            return;
        };

        let Some(children) = self.backend.get_children_ids(&focus.mailbox) else {
            return;
        };

        match &children[idx] {
            Entry::This => {
                self.app_actions
                    .push(crate::Action::OpenRootMails(focus.mailbox.clone().unwrap()));
            }
            Entry::Child(id) => {
                self.focus.push(FocusCtx::new(Some(id.clone())));
            }
        }
    }

    fn toggle_mailbox(&mut self) {
        let focus = self.focus.last().unwrap();
        let Some(idx) = focus.state.selected() else {
            return;
        };

        let Some(children) = self.backend.get_children_ids(&focus.mailbox) else {
            return;
        };

        match &children[idx] {
            Entry::This => {}
            Entry::Child(id) => {
                if self.selected.remove(id).is_none() {
                    self.selected.insert(id.clone(), SelectionType::Selected);
                }
                self.navigate_to_mailbox_below();
            }
        }
    }

    fn cut_selection(&mut self) {
        if self.selected.is_empty() {
            let focus = self.focus.last().unwrap();
            let Some(idx) = focus.state.selected() else {
                return;
            };

            let Some(children) = self.backend.get_children_ids(&focus.mailbox) else {
                return;
            };

            match &children[idx] {
                Entry::This => {}
                Entry::Child(id) => {
                    self.selected.insert(id.clone(), SelectionType::Cut);
                }
            }
        } else {
            for selected in self.selected.values_mut() {
                *selected = SelectionType::Cut;
            }
        }
    }

    fn paste_selection(&mut self) {
        let focus = self.focus.last().unwrap();
        let new_parent = focus.mailbox.clone();

        let mailboxes_to_be_updated = self
            .selected
            .drain()
            .filter_map(|(id, ty)| (ty == SelectionType::Cut).then_some(id))
            .map(|id| MailboxUpdate {
                id,
                parent_id: Some(new_parent.clone()),
                ..Default::default()
            })
            .collect::<Vec<MailboxUpdate>>();

        self.backend.update_mailboxes(mailboxes_to_be_updated);
    }

    fn go_back(&mut self) {
        if self.focus.len() > 1 {
            self.focus.pop();
        }
    }

    fn set_sort_order(&mut self) {
        let focus = self.focus.last().unwrap();
        let Some(idx) = focus.state.selected() else {
            return;
        };

        let Some(children) = self.backend.get_children_ids(&focus.mailbox) else {
            return;
        };

        match &children[idx] {
            Entry::This => {}
            Entry::Child(id) => {
                self.overlay = Some(ScreenOverlay::Input(input::State::new(
                    "Set sort order (>= 0):",
                    InputType::SortOrder(id.clone()),
                )));
            }
        }
    }

    fn move_mailbox_up(&mut self) {
        let focus = self.focus.last_mut().unwrap();
        let Some(idx) = focus.state.selected() else {
            return;
        };

        let Some(children) = self.backend.get_children_sort_order(&focus.mailbox) else {
            return;
        };

        let child = match &children[idx] {
            Children::This => return,
            Children::Child(id, _) => id.clone(),
        };

        let above1 = children.get(idx - 1);
        let new_sort_order = match above1 {
            None | Some(Children::This) => return,
            Some(Children::Child(_, order1)) => {
                let order2 = {
                    let above2 = children.get(idx - 2);
                    match above2 {
                        None | Some(Children::This) => 0,
                        Some(Children::Child(_, order2)) => *order2,
                    }
                };
                *order1 - (*order1 - order2) / 2
            }
        };

        self.backend.update_mailboxes(vec![MailboxUpdate {
            id: child,
            sort_order: Some(new_sort_order),
            ..Default::default()
        }]);

        self.navigate_to_mailbox_above();
    }

    fn move_mailbox_down(&mut self) {
        let focus = self.focus.last_mut().unwrap();
        let Some(idx) = focus.state.selected() else {
            return;
        };

        let Some(children) = self.backend.get_children_sort_order(&focus.mailbox) else {
            return;
        };

        let child = match &children[idx] {
            Children::This => return,
            Children::Child(id, _) => id.clone(),
        };

        let below1 = children.get(idx - 1);
        let new_sort_order = match below1 {
            None | Some(Children::This) => return,
            Some(Children::Child(_, order1)) => {
                let below2 = children.get(idx - 2);
                match below2 {
                    None | Some(Children::This) => {
                        (*order1 + 1).next_multiple_of(NEW_SORT_ORDER_SIZE)
                    }
                    Some(Children::Child(_, order2)) => *order1 - (*order1 - order2) / 2,
                }
            }
        };

        self.backend.update_mailboxes(vec![MailboxUpdate {
            id: child,
            sort_order: Some(new_sort_order),
            ..Default::default()
        }]);

        self.navigate_to_mailbox_below();
    }

    fn normalize_sort_order(&mut self) {
        let focus = self.focus.last().unwrap();

        if let Some(children) = self.backend.get_children_ids(&focus.mailbox) {
            let mut updates: Vec<MailboxUpdate> = Vec::with_capacity(children.len());

            for (idx, entry) in children.into_iter().enumerate() {
                match entry {
                    Entry::This => continue,
                    Entry::Child(id) => updates.push(MailboxUpdate {
                        id,
                        sort_order: Some(NEW_SORT_ORDER_SIZE * (idx as SortOrder + 1)),
                        ..Default::default()
                    }),
                }
            }

            self.backend.update_mailboxes(updates);
        }
    }

    fn create_mailbox(&mut self) {
        let focus = self.focus.last().unwrap();
        let parent = focus.mailbox.clone();

        self.overlay = Some(ScreenOverlay::Input(input::State::new(
            "Create:",
            InputType::NewMailboxName { parent },
        )));
    }

    fn remove_selected_mailboxes(&mut self) {
        if self.selected.is_empty() {
            let focus = self.focus.last().unwrap();
            let Some(idx) = focus.state.selected() else {
                return;
            };

            let Some(children) = self.backend.get_children_ids(&focus.mailbox) else {
                return;
            };

            match &children[idx] {
                Entry::This => {}
                Entry::Child(id) => {
                    self.backend.destroy_mailboxes(vec![id.clone()]);
                }
            }
        } else {
            let ids: Vec<MailboxId> = self.selected.drain().map(|(id, _)| id).collect();
            self.backend.destroy_mailboxes(ids);
        }
    }

    fn rename_selected_mailboxes(&mut self) {
        let mailboxes: Vec<(MailboxId, String)> = if self.selected.is_empty() {
            let focus = self.focus.last().unwrap();
            let Some(idx) = focus.state.selected() else {
                return;
            };

            let Some(children) = self.backend.get_children_names(&focus.mailbox) else {
                return;
            };

            match &children[idx] {
                Children::This => vec![],
                Children::Child(id, name) => vec![(id.clone(), name.clone())],
            }
        } else {
            self.selected
                .drain()
                .map(|(id, _)| (id.clone(), self.backend.get_mailbox_name(&id).unwrap()))
                .collect()
        };

        if !mailboxes.is_empty() {
            const RENAMING_PATH: &str = "/tmp/brieftaube-renaming.txt";
            let file_content = mailboxes
                .iter()
                .map(|(_, name)| format!("{}\n", name))
                .collect::<String>();
            std::fs::write(RENAMING_PATH, file_content).expect("Create renaming file");

            // open editor
            {
                ratatui::restore();
                let status = std::process::Command::new(self.config.editor().unwrap())
                    .arg(RENAMING_PATH)
                    .status()
                    .unwrap();
                if !status.success() {
                    error!("Couldn't start editor: {}", status);
                    return;
                }
                ratatui::init();
            }

            let new_names: Vec<String> = std::fs::read_to_string(RENAMING_PATH)
                .unwrap()
                .lines()
                .map(|line| line.to_string())
                .collect();

            let updates: Vec<MailboxUpdate> = {
                let ids = mailboxes.iter().map(|(id, _)| id.clone());
                ids.zip(new_names.into_iter())
                    .map(|(id, new_name)| MailboxUpdate {
                        id,
                        name: Some(new_name),
                        ..Default::default()
                    })
                    .collect()
            };

            self.backend.update_mailboxes(updates);

            self.app_actions.push(crate::Action::Redraw);
        }
    }
}

// render
impl State {
    pub fn is_waiting_for_data(&self) -> bool {
        !self.backend.cache_is_initialised()
    }

    pub fn get_parent_column<'a>(&'a mut self) -> Option<ColumnData<'a>> {
        let focus = self
            .focus
            .len()
            .checked_sub(2)
            .and_then(|prev_last_idx| self.focus.get_mut(prev_last_idx))?;

        let children = self.backend.get_children(&focus.mailbox)?;
        let mailboxes = to_mailbox_display(children, &self.selected);

        Some(ColumnData {
            mailboxes,
            state: Some(&mut focus.state),
        })
    }

    pub fn get_center_column<'a>(&'a mut self) -> ColumnData<'a> {
        let focus = self.focus.last_mut().unwrap();
        let mailboxes = {
            let children = self.backend.get_children(&focus.mailbox).unwrap();
            to_mailbox_display(children, &self.selected)
        };

        ColumnData {
            mailboxes,
            state: Some(&mut focus.state),
        }
    }

    pub fn get_right_column<'a>(&'a self) -> Option<ColumnData<'a>> {
        let focus = self.focus.last()?;
        let idx = focus.state.selected()?;
        let children = self.backend.get_children_ids(&focus.mailbox)?;

        match &children[idx] {
            Entry::This => Some(ColumnData {
                mailboxes: vec![MailboxDisplay::This],
                state: None,
            }),
            Entry::Child(id) => {
                let children = self.backend.get_children(&Some(id.clone()))?;
                let mailboxes = to_mailbox_display(children, &self.selected);
                Some(ColumnData {
                    mailboxes,
                    state: None,
                })
            }
        }
    }
}

fn to_mailbox_display(
    mailboxes: Vec<Children<MailboxData>>,
    selection: &HashMap<MailboxId, SelectionType>,
) -> Vec<MailboxDisplay> {
    mailboxes
        .into_iter()
        .map(|child| match child {
            Children::This => MailboxDisplay::This,
            Children::Child(_, child) => {
                let selection_type = selection.get(&child.id).cloned();

                MailboxDisplay::Entry {
                    selection_type,
                    sort_order: child.sort_order,
                    name: child.name.clone(),
                    unread_mails: child.unread_mails,
                }
            }
        })
        .collect()
}
