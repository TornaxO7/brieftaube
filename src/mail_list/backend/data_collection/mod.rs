pub mod error;
pub mod types;

mod root_mails;
mod threads;

use crate::{
    mail_list::backend::data_collection::{error::UnfoldRowError, threads::Thread},
    utils::{MailId, ThreadId},
};
use jmap_client::core::{
    query::QueryResponse,
    response::{EmailGetResponse, ThreadGetResponse},
};
use ratatui::widgets::TableState;
use root_mails::RootMails;
use threads::Threads;
use types::MailData;

type EmailGetState = String;

pub trait Data {
    fn get_mail(&self, id: &MailId) -> Option<&MailData>;

    fn get_mail_mut(&mut self, id: &MailId) -> Option<&mut MailData>;
}

#[derive(Debug, Clone)]
pub enum Row {
    Root(MailId),
    Child(ThreadId, MailId),
}

pub struct DataCollection {
    root_mails: RootMails,
    threads: Threads,

    pub rows: Vec<Row>,
    pub table_state: TableState,

    state: EmailGetState,
}

impl DataCollection {
    pub fn new(mut query: QueryResponse, mut get: EmailGetResponse) -> Self {
        let get_state = get.take_state();
        let query_state = query.take_query_state();

        let mails = get.take_list();
        let table_state = if !mails.is_empty() {
            TableState::new().with_selected(Some(0))
        } else {
            TableState::new()
        };

        let rows = mails
            .iter()
            .map(|mail| Row::Root(mail.id().unwrap().to_string()))
            .collect();

        let root_mails = RootMails::new(mails, query_state);

        Self {
            root_mails,
            rows,
            threads: Threads::new(),
            table_state,
            state: get_state,
        }
    }

    pub fn get_selected_mail(&self) -> Option<MailData> {
        self.table_state
            .selected()
            .and_then(|idx| match &self.rows[idx] {
                Row::Root(id) => self.root_mails.get_mail(id).cloned(),
                Row::Child(thread_id, mail_id) => self
                    .threads
                    .get_mail_from_thread(thread_id, mail_id)
                    .cloned(),
            })
    }

    pub fn get_mail_state(&self) -> String {
        self.state.clone()
    }

    pub fn set_mail_state(&mut self, new_email_get_state: String) {
        self.state = new_email_get_state;
    }
}

// Thread stuff
impl DataCollection {
    pub fn unfold_row(&mut self, row_idx: usize) -> Result<(), UnfoldRowError> {
        let Row::Root(root_mail_id) = &self.rows[row_idx] else {
            return Err(UnfoldRowError::NonRootRow);
        };
        let mail = self.get_mail(root_mail_id).expect("Mail exists.");

        match self.rows.get(row_idx + 1) {
            Some(Row::Child(thread_id, _)) if *thread_id == mail.thread_id => {
                return Err(UnfoldRowError::AlreadyUnfolded);
            }
            _ => {}
        };

        let Some(thread) = self.threads.get_thread(&mail.thread_id) else {
            return Err(UnfoldRowError::NotInitialised(mail.thread_id.clone()));
        };
        let thread_mails = thread.mails();

        self.rows.reserve(thread_mails.len());
        let thread_mail_rows: Vec<Row> = thread_mails
            .iter()
            .map(|mail| Row::Child(mail.thread_id.clone(), mail.id.clone()))
            .collect();

        let dst = row_idx + 1;
        self.rows.splice(dst..dst, thread_mail_rows);
        Ok(())
    }

    pub fn fold_row(&mut self, row_idx: usize) -> bool {
        let thread_id = match &self.rows[row_idx] {
            Row::Root(_) => match self.rows.get(row_idx + 1) {
                Some(Row::Child(thread_id, _)) => thread_id.clone(),
                _ => return false,
            },
            Row::Child(thread_id, _) => thread_id.clone(),
        };

        let thread_start = self
            .rows
            .iter()
            .position(|row| matches!(row, Row::Child(id, _) if *id == thread_id))
            .unwrap();

        self.table_state.select(Some(thread_start - 1));
        self.rows
            .retain(|row| !matches!(row, Row::Child(id, _) if *id == thread_id));
        true
    }

    pub fn insert_thread(
        &mut self,
        id: &ThreadId,
        mut mail_response: EmailGetResponse,
        mut thread_response: ThreadGetResponse,
    ) {
        self.state = mail_response.take_state();

        let new_thread = {
            let thread_mails = mail_response
                .take_list()
                .into_iter()
                .map(MailData::from)
                .collect();

            let thread_state = thread_response.take_state();

            Thread::new(thread_state, thread_mails)
        };

        self.threads.insert_thread(id.clone(), new_thread);
    }
}

// methods for widget
impl DataCollection {
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    pub fn get_mail_from_row(&self, row: &Row) -> &MailData {
        match row {
            Row::Root(id) => self.root_mails.get_mail(id).unwrap(),
            Row::Child(thread_id, mail_id) => self
                .threads
                .get_mail_from_thread(thread_id, mail_id)
                .unwrap(),
        }
    }
}

impl Data for DataCollection {
    fn get_mail(&self, id: &MailId) -> Option<&MailData> {
        self.root_mails
            .get_mail(id)
            .or_else(|| self.threads.get_mail(id))
    }

    fn get_mail_mut(&mut self, id: &MailId) -> Option<&mut MailData> {
        self.root_mails
            .get_mail_mut(id)
            .or_else(|| self.threads.get_mail_mut(id))
    }
}
