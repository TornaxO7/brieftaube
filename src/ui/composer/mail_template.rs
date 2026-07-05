use crate::backend::Account;
use chrono::{DateTime, Local};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug)]
pub struct MailTemplate {
    date: DateTime<Local>,
    pub to: String,
    pub from: String,
    pub subject: String,
    pub extra_headers: HashMap<String, String>,

    pub body: String,
}

impl MailTemplate {
    pub fn new(account: Arc<Account>) -> Self {
        Self {
            date: Local::now(),
            to: String::new(),
            from: account.address.clone(),
            subject: String::new(),

            extra_headers: HashMap::new(),
            body: String::new(),
        }
    }

    pub fn from(&self) -> &str {
        &self.from
    }

    pub fn to(&self) -> &str {
        &self.to
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn date(&self) -> String {
        self.date.format("%A, %d %B %Y %T").to_string()
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn renderable(&self) -> String {
        let date = self.date();
        let from = self.from();
        let to = self.to();
        let subject = self.subject();

        let headers = {
            let mut s = String::new();

            for (header, value) in self.extra_headers.iter() {
                s.push_str(&format!("{}: {}", header, value));
            }

            s
        };

        let body = self.body();

        format!(
            "\
Date: {date}
From: {from}
To: {to}
Subject: {subject}
{headers}
{body}"
        )
    }
}
