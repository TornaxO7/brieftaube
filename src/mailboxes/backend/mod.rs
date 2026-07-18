mod layers;
mod mailbox_data;

use crate::{config::Config, utils::MailboxId};
use jmap_client::{
    URI,
    client::Client,
    core::{session::Capabilities, set::SetObject},
    mailbox::Role,
};
pub use layers::{Layer, Layers};
pub use mailbox_data::MailboxData;
use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
    sync::{Arc, Mutex},
};
use tokio::task::{JoinError, JoinHandle};
use tracing::{ error, warn};

const NEW_SORT_ORDER_SIZE: u32 = 32;
const DATA_INITIALISED_MSG: &str = "Is initialised";

pub struct Data {
    pub layers: Layers,
    state: String,
}

pub struct Backend {
    client: Arc<Client>,
    pub data: Arc<Mutex<Option<Data>>>,
    pub config: Rc<Config>,
    tasks: Arc<Mutex<VecDeque<JoinHandle<()>>>>,
}

impl Backend {
    pub fn new(client: Arc<Client>, config: Rc<Config>) -> Self {
        Self {
            client,
            data: Arc::new(Mutex::new(None)),
            tasks: Arc::new(Mutex::new(VecDeque::with_capacity(16))),
            config,
        }
    }

    pub fn has_tasks_running(&self) -> bool {
        !self.tasks.lock().unwrap().is_empty()
    }

    pub async fn has_changed(&self) -> Option<Result<(), JoinError>> {
        let mut guard = self.tasks.lock().unwrap();
        let task = guard.front_mut().unwrap();
        Some(task.await)
    }

    pub fn pop_task(&self) {
        self.tasks.lock().unwrap().pop_front().expect("There are tasks.");
    }

    pub fn is_initialised(&self) -> bool {
        self.data.lock().unwrap().is_some()
    }
}

// methods which also communicate with the server
impl Backend {
    pub fn init(&self) {
        let client = self.client.clone();
        let data = self.data.clone();

        self.tasks.lock().unwrap().push_back(tokio::spawn(async move {
            let mut request = client.build();
            request.get_mailbox().ids::<[_; 1], String>(None::<[_; 1]>);
            let mut response = match request.send_get_mailbox().await {
                Ok(r) => r,
                Err(err) => {
                    error!("Couldn't request mailboxes from server: {err}");
                    return;
                }
            };

            let state = response.take_state();
            let layers = {
                let mailboxes: Vec<MailboxData> = response
                    .take_list()
                    .into_iter()
                    .map(|mailbox| MailboxData::from(mailbox))
                    .collect();

                Layers::new(mailboxes)
            };

            let mut data = data.lock().unwrap();
            let new_data = Data { layers, state };
            *data = Some(new_data);
        }));
    }

    pub fn destroy_mailboxes(&self, ids: Vec<MailboxId>) {
        if !self.is_initialised() {
            return;
        }

        let data = self.data.clone();
        let client = self.client.clone();

        self.tasks.lock().unwrap().push_back(tokio::spawn(async move {
            let mut response = {
                let mut request = client.build();
                request
                    .set_mailbox()
                    .destroy(&ids)
                    .arguments()
                    .on_destroy_remove_emails(false);

                match request.send_set_mailbox().await {
                    Ok(r) => r,
                    Err(err) => {
                        error!("Couldn't request server to destroy mailboxes: {err}");
                        return;
                    }
                }
            };

            let mut guard = data.lock().unwrap();
            let data = guard.as_mut().expect(DATA_INITIALISED_MSG);
            data.state = response.take_new_state();
            for id in ids.into_iter() {
                match response.destroyed(&id) {
                    Ok(()) => {
                        data.layers.remove_mailbox(id);
                    }
                    Err(err) => match data.layers.get_mailbox(&id) {
                        Some(mailbox) => {
                            let name = mailbox.name.clone();
                            error!("Couldn't destroy the mailbox '{name}': {err}");
                        },
                        None => {
                            error!("Couldn't destroy mailbox:\n{err}");
                        }
                    }
                }
            }
        }));
    }

    pub fn set_new_order(&self, id: MailboxId, new_order: u32) {
        if !self.is_initialised() {
            return;
        }

        let data = self.data.clone();
        let client = self.client.clone();

        self.tasks.lock().unwrap().push_back(tokio::spawn(async move {
            let mut response = {
                let mut request = client.build();
                request.set_mailbox().update(&id).sort_order(new_order);

                match request.send_set_mailbox().await {
                    Ok(r) => r,
                    Err(err) => {
                        error!("Couldn't request server to update the sort order: {err}");
                        return;
                    }
                }
            };

            let mut guard = data.lock().unwrap();
            let data = guard.as_mut().expect(DATA_INITIALISED_MSG);
            data.state = response.take_new_state();
            match response.updated(&id) {
                Ok(_) => {
                    data.layers.set_sort_order(id, new_order);
                }
                Err(err) => match data.layers.get_mailbox(&id) {
                    Some(mailbox) => {
                        let name = mailbox.name.clone();
                        error!("Couldn't update the sort order of '{name}':\n{err}");
                    },
                    None => {
                        error!("Couldn't update the sort order:\n{err}");
                    }
                }
            };
        }));
    }

    pub fn create_mailbox(&self, parent: Option<MailboxId>, name: String) {
        if !self.is_initialised() {
            return;
        }

        let client = self.client.clone();
        let data = self.data.clone();
        let caps = self.mail_capability();

        self.tasks.lock().unwrap().push_back(tokio::spawn(async move {
            let err_msg = {
                let guard = data.lock().unwrap();
                let data = guard.as_ref().expect(DATA_INITIALISED_MSG);

                if data.layers.depth() > caps.max_mailbox_depth() {
                    let max = caps.max_mailbox_depth();
                    Some(format!("Max mailbox depth reached for the mail server :( You can't create another sub-mailbox in the current mailbox. The maximum depth is {max} for the server."))
                } else if name.len() > caps.max_size_mailbox_name() {
                    let max = caps.max_size_mailbox_name();
                    Some(format!("The mailbox name is too long. It can be at most {max} characters long."))
                } else if data
                    .layers
                    .get_current_layer()
                    .contains_mailbox_name(name.as_str())
                {
                    let n = name.clone();
                    Some(format!("There's already a mailbox in the current mailbox with the name '{n}'."))
                } else {
                    None
                }
            };

            if let Some(msg) = err_msg {
                error!("Can't create mailbox: {msg}");
                return;
            }

            let mut new_mailbox = {
                let sort_order = {
                    let guard = data.lock().unwrap();
                    let data = guard.as_ref().expect(DATA_INITIALISED_MSG);
                    let layer = data.layers.get_layer(&parent);
                    layer
                        .mailboxes
                        .iter()
                        .map(|mailbox| mailbox.sort_order)
                        .max()
                        .map(|biggest_sort_order| {
                            (biggest_sort_order + 1).next_multiple_of(NEW_SORT_ORDER_SIZE)
                        })
                        .unwrap_or(NEW_SORT_ORDER_SIZE)
                };

                MailboxData {
                    name: name.clone(),
                    role: Role::None,
                    sort_order,
                    parent_id: parent.clone(),
                    ..Default::default()
                }
            };

            let (mut response, id) = {
                let mut request = client.build();

                let id = request
                    .set_mailbox()
                    .create()
                    .name(&new_mailbox.name)
                    .parent_id(new_mailbox.parent_id.as_ref())
                    .role(new_mailbox.role.clone())
                    .sort_order(new_mailbox.sort_order)
                    .create_id()
                    .unwrap();

                match request.send_set_mailbox().await {
                    Ok(r) => (r, id),
                    Err(err) => {
                        error!("Couldn't send request to server to create mailbox: {err}");
                        return;
                    }
                }
            };
            
            
            match response.created(&id) {
                Ok(mut server) => {
                    new_mailbox.id = server.take_id();
                },
                Err(err) => {
                    let name = &new_mailbox.name;
                    error!("Couldn't create mailbox '{name}': {err}");
                    return;
                }
            };

            let mut guard = data.lock().unwrap();
            let data = guard.as_mut().expect(DATA_INITIALISED_MSG);
            data.layers.add_mailbox(new_mailbox);
            data.state = response.take_new_state();
          
        }));
    }

    pub fn normalize_sort_order(&self) {
        if !self.is_initialised() {
            return;
        }

        let data = self.data.clone();
        let client = self.client.clone();

        self.tasks.lock().unwrap().push_back(tokio::spawn(async move {
            let ids: Vec<MailboxId> = {
                let guard = data.lock().unwrap();
                let data = guard.as_ref().expect(DATA_INITIALISED_MSG);
                let layer = data.layers.get_current_layer();
                layer
                    .mailboxes
                    .iter()
                    .map(|mailbox| mailbox.id.clone())
                    .collect()
            };

            let new_sort_orders: HashMap<MailboxId, u32> = ids
                .into_iter()
                .enumerate()
                .map(|(idx, id)| (id, (idx + 1) as u32 * NEW_SORT_ORDER_SIZE))
                .collect();

            // send changes
            let mut response = {
                let mut request = client.build();
                let set_mailbox = request.set_mailbox();
                for (id, new_sort_order) in new_sort_orders.iter() {
                    set_mailbox.update(id).sort_order(*new_sort_order);
                }
                match request.send_set_mailbox().await {
                    Ok(r) => r,
                    Err(err) => {
                        error!("Couldn't send new sort-order requests: {err}");
                        return;
                    }
                }
            };

            let mut guard = data.lock().unwrap();
            let data = guard.as_mut().expect(DATA_INITIALISED_MSG);
            data.state = response.take_new_state();

            // check that everything worked fine
            for (id, new_sort_order) in new_sort_orders.into_iter() {
                let Some(mailbox) = data.layers.get_mailbox_mut(&id) else {
                    warn!(concat![
                        "It looks like as if a mailbox has been removed while requesting a new sort order.\n",
                        "We are going to skip one mailbox."
                    ]);
                    continue;
                };
                if let Err(err) = response.updated(&id) {
                    let name = mailbox.name.clone();
                    warn!("Couldn't update sort order of '{name}': {err}\nGoing to skip it ...");
                    continue;   
                }

                mailbox.sort_order = new_sort_order;
            }
        }));
    }

    pub fn rename_mailboxes(&self, mapping: Vec<(MailboxId, String)>) {
        if !self.is_initialised() {
            return;
        }

        // TODO: add check if that's even possible

        let data = self.data.clone();
        let client = self.client.clone();

        self.tasks.lock().unwrap().push_back(tokio::spawn(async move {
            let mut response = {
                let mut request = client.build();
                {
                    let set_mailbox = request.set_mailbox();
                    for map in mapping.iter() {
                        set_mailbox.update(&map.0).name(&map.1);
                    }
                }
                match request.send_set_mailbox().await {
                    Ok(r) => r,
                    Err(err) => {
                        error!("Couldn't request server to rename mailboxes: {err}");
                        return;
                    }
                }
            };

            let mut guard = data.lock().unwrap();
            let data = guard.as_mut().expect(DATA_INITIALISED_MSG);
            data.state = response.take_new_state();
            for (id, new_name) in mapping.into_iter() {
                let mailbox = data.layers.get_mailbox_mut(&id).expect("Mailbox exists");
                if let Err(err) = response.updated(&id) {
                    let old_name = &mailbox.name;
                    let new_name = &new_name;
                    warn!("Couldn't rename mailbox '{old_name}' to '{new_name}': {err}\nSkipping this mailbox.");
                    continue;
                }

                mailbox.name = new_name;
            }
            
            

        }));
    }

    pub fn move_mailbox_up(&self, id: MailboxId) {
        if !self.is_initialised() {
            return;
        }

        let new_order = {
            let guard = self.data.lock().unwrap();
            let data = guard.as_ref().expect(DATA_INITIALISED_MSG);

            // validate
            let layer = data.layers.get_layer_containing_mailbox(&id);
            let idx = layer.mailboxes.iter().enumerate().find_map(|(idx, mailbox)| (mailbox.id == id).then_some(idx)).unwrap();
            let is_at_top = idx == 0;
            if is_at_top {
                return;
            }
            
            let top1 = layer.mailboxes[idx - 1].sort_order;
            let top2 = if idx < 2 {
                0
            } else {
                layer.mailboxes[idx - 2].sort_order
            };

            top1 - (top1 - top2) / 2
        };

        self.set_new_order(id, new_order);
    }

    pub fn move_mailbox_down(&self, id: MailboxId) {
        if !self.is_initialised() {
            return;
        }

        let new_order = {
            let guard = self.data.lock().unwrap();
            let data = guard.as_ref().expect(DATA_INITIALISED_MSG);

            // validate
            let layer = data.layers.get_layer_containing_mailbox(&id);
            let idx = layer.mailboxes.iter().enumerate().find_map(|(idx, mailbox)| (mailbox.id == id).then_some(idx)).unwrap();
            let is_at_bottom = idx == layer.mailboxes.len() - 1;
            if is_at_bottom {
                return;
            }

            let below1 = layer.mailboxes[idx + 1].sort_order;
            match layer.mailboxes.get(idx + 2) {
                Some(below2_mailbox) => {
                    let below2 = below2_mailbox.sort_order;
                    below1 + (below2 - below1) / 2
                }
                None => (below1 + 1).next_multiple_of(NEW_SORT_ORDER_SIZE),
                
            }
        };

        self.set_new_order(id, new_order);
    }

    pub fn move_mailboxes_to(&self, ids: Vec<MailboxId>, new_parent: Option<MailboxId>) {
        if !self.is_initialised() {
            return;
        }

        struct MailboxToMove {
            id: MailboxId,
            old_name: String,
            new_name: Option<String>,
        }

        let filtered_ids: Vec<MailboxToMove> = {
            let guard = self.data.lock().unwrap();
            let data = guard.as_ref().expect(DATA_INITIALISED_MSG);

            let mut filtered_ids = Vec::with_capacity(ids.len());
            let new_parent_layer = data.layers.get_layer(&new_parent);
            for id in ids.into_iter() {
                let mailbox = data.layers.get_mailbox(&id).unwrap();
                let old_name = mailbox.name.clone();

                if new_parent_layer.contains_mailbox(&id) {
                    continue;
                }

                let new_name = if new_parent_layer.contains_mailbox_name(&mailbox.name) {
                    Some(format!("{}-1", &mailbox.name))
                } else {
                    None
                };

                filtered_ids.push(MailboxToMove {
                    id,
                    old_name,
                    new_name
                });
            }

            filtered_ids
        };

        let data = self.data.clone();
        let client = self.client.clone();

        self.tasks.lock().unwrap().push_back(tokio::spawn(async move {
            let mut response = {
                let mut request = client.build();
                let set_mailbox = request.set_mailbox();
                for MailboxToMove { id, new_name,.. } in filtered_ids.iter() {
                    match new_name {
                        Some(name) => set_mailbox.update(id).parent_id(new_parent.clone()).name(name),
                        None => set_mailbox.update(id).parent_id(new_parent.clone()),
                    };
                }
                match request.send_set_mailbox().await {
                    Ok(r) => r,
                    Err(err) => {
                        error!("Couldn't send request to server for moving mailboxes: {err}");
                        return;
                    }
                }
            };

            let mut guard = data.lock().unwrap();
            let data = guard.as_mut().expect(DATA_INITIALISED_MSG);
            data.state = response.take_new_state();
            
            for MailboxToMove {id, old_name, new_name} in filtered_ids.into_iter() {
                match response.updated(&id) {
                    Ok(server_changes) => {
                        let old = data.layers.get_mailbox(&id).unwrap().clone();
                        let mut new = old.clone();
                        new.parent_id = new_parent.clone();
                        if let Some(name) = new_name {
                            new.name = name;
                        }

                        if let Some(server) = server_changes {
                            if let Some(name) = server.name() {
                                new.name = name.to_string()
                            }

                            if let Some(parent_id) = server.parent_id() {
                                new.parent_id = Some(parent_id.to_string());
                            }
                        }

                        // update its layer
                        {
                            let mut layer = data.layers.remove_layer(&Some(old.id.clone()));
                            layer.mailbox_owner = Some(new.id.clone());
                            data.layers.insert_layer(Some(new.id.clone()), layer);
                        }

                        // update its old and new parent layer
                        if old.parent_id != new.parent_id {
                            let prev_layer = data.layers.get_layer_mut(&old.parent_id);
                            prev_layer
                                .mailboxes
                                .extract_if(.., |mailbox| mailbox.id == old.id)
                                .for_each(drop);

                            let new_layer = data.layers.get_layer_mut(&new.parent_id);
                            new_layer.mailboxes.push(new);
                            new_layer.sort_mailboxes();
                        }
                    },
                    Err(err) => {
                        error!("Couldn't update '{old_name}': {err}");
                        continue;
                    }
                }
            }
        }));
    }

    pub fn mail_capability(&self) -> jmap_client::email::MailCapabilities {
        let id = self.client.default_account_id();

        match self
            .client
            .session()
            .account(id)
            .unwrap()
            .capability(URI::Mail.as_ref())
            .unwrap()
            .clone()
        {
            Capabilities::Mail(cap) => cap,
            _ => unreachable!(),
        }
    }
}

// methods for `state.rs`
impl Backend {
    pub fn select_next_mailbox(&self) {
        let mut guard = self.data.lock().unwrap();
        if let Some(data) = guard.as_mut() {
            let current_layer = data.layers.get_current_layer_mut();
            current_layer.state.select_next();
        }
    }

    pub fn select_previous_mailbox(&self) {
        let mut guard = self.data.lock().unwrap();
        if let Some(data) = guard.as_mut() {
            let current_layer = data.layers.get_current_layer_mut();
            current_layer.state.select_previous();
        }
    }

    pub fn select_first_mailbox(&self) {
        let mut guard = self.data.lock().unwrap();
        if let Some(data) = guard.as_mut() {
            let current_layer = data.layers.get_current_layer_mut();
            *current_layer.state.selected_mut() = Some(0);
        }
    }

    pub fn select_last_mailbox(&self) {
        let mut guard = self.data.lock().unwrap();
        if let Some(data) = guard.as_mut() {
            let layer = data.layers.get_current_layer_mut();
            *layer.state.selected_mut() = if layer.is_root_layer() {
                Some(layer.mailboxes.len())
            } else {
                Some(layer.mailboxes.len() - 1)
            };
        }
    }

    pub fn activate_selected_entry(&self) -> Option<MailboxId> {
        let mut guard = self.data.lock().unwrap();
        guard
            .as_mut()
            .and_then(|data| data.layers.open_selected_entry())
    }

    pub fn go_back(&self) {
        let mut guard = self.data.lock().unwrap();
        if let Some(data) = guard.as_mut() {
            data.layers.go_up_one_level();
        }
    }

    pub fn can_set_sort_order(&self) -> Option<bool> {
        let guard = self.data.lock().unwrap();
        guard.as_ref().map(|data| {
            let layer = data.layers.get_current_layer();
            !layer.selected_parent()
        })
    }

    pub fn get_selected_mailbox(&self) -> Option<MailboxData> {
        let guard = self.data.lock().unwrap();
        guard
            .as_ref()
            .map(|data| data.layers.get_current_layer())
            .and_then(|layer| layer.get_selected_mailbox())
            .map(|mailbox| mailbox.clone())
    }

    pub fn get_mailboxes(&self, ids: &[MailboxId]) -> Option<Vec<MailboxData>> {
        let guard = self.data.lock().unwrap();
        guard.as_ref().map(|data| &data.layers).map(|layers| {
            let mut mailboxes = Vec::with_capacity(ids.len());

            for id in ids.iter() {
                let mailbox = layers
                    .get_mailbox(id)
                    .expect(&format!("Mailbox with id '{}' exsist.", id));
                mailboxes.push(mailbox.clone());
            }

            mailboxes
        })
    }

    pub fn get_parent_mailbox(&self) -> Option<MailboxId> {
        let guard = self.data.lock().unwrap();
        guard
            .as_ref()
            .map(|data| data.layers.get_current_layer())
            .and_then(|layer| layer.mailbox_owner.clone())
    }
}
