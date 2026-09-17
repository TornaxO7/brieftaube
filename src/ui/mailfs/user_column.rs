use crate::{
    CONFIG,
    config::{self, UserConfig},
    types::AccountData,
    ui::{mailfs::MailfsColumn, utils::Loadable},
};
use ratatui::widgets::TableState;

#[derive(Debug, Clone)]
pub struct UserColumn {
    pub users: Vec<UserCtx>,
    pub state: TableState,
}

impl UserColumn {
    pub fn new() -> Self {
        let config = CONFIG.get().unwrap();

        let users: Vec<UserCtx> = config.users.iter().cloned().map(UserCtx::new).collect();

        let state = if users.is_empty() {
            TableState::new()
        } else {
            TableState::new().with_selected(Some(0))
        };

        Self { users, state }
    }

    pub fn set_accounts(
        &mut self,
        username: config::Username,
        accounts: Loadable<Vec<AccountData>>,
    ) {
        let user = self
            .users
            .iter_mut()
            .find(|user| user.config.username == username)
            .unwrap();

        user.accounts = accounts;
    }

    pub fn iter<'a>(&'a self) -> UserColumnIterator<'a> {
        UserColumnIterator::new(self)
    }

    pub fn select_next(&mut self) {
        todo!()
    }

    pub fn navigate_right(&mut self) -> Vec<super::super::Message> {
        todo!()
    }

    pub fn navigate_left(&mut self) -> Vec<super::super::Message> {
        todo!()
    }
}

impl MailfsColumn for UserColumn {
    fn navigate_up(&mut self) -> Vec<crate::ui::Message> {
        todo!()
    }

    fn navigate_down(&mut self) -> Vec<crate::ui::Message> {
        todo!()
    }

    fn navigate_to_bottom(&mut self) -> Vec<crate::ui::Message> {
        todo!()
    }

    fn navigate_to_top(&mut self) -> Vec<crate::ui::Message> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct UserCtx {
    pub config: UserConfig,
    pub is_collapsed: bool,
    pub accounts: Loadable<Vec<AccountData>>,
}

impl UserCtx {
    pub fn new(user: UserConfig) -> Self {
        Self {
            config: user,
            is_collapsed: true,
            accounts: Loadable::NotLoaded,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserColumnIterator<'a> {
    column: &'a UserColumn,

    user_idx: Option<usize>,
    account_idx: Option<usize>,
}

impl<'a> UserColumnIterator<'a> {
    fn new(column: &'a UserColumn) -> Self {
        let ctx_idx = if column.users.is_empty() {
            None
        } else {
            Some(0)
        };

        Self {
            column,
            user_idx: ctx_idx,
            account_idx: None,
        }
    }
}

impl<'a> Iterator for UserColumnIterator<'a> {
    type Item = UserColumnEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let ctx_idx = self.user_idx?;

        let ctx = &self.column.users[ctx_idx];
        match self.account_idx {
            Some(account_idx) => {
                let ret_value = match ctx.accounts.as_ref() {
                    Loadable::NotLoaded => UserColumnEntry::AccountNotLoaded,
                    Loadable::Loading => UserColumnEntry::AccountLoading,
                    Loadable::Loaded(accounts) => {
                        let no_accounts_left = account_idx >= accounts.len() - 1;
                        if no_accounts_left {
                            self.account_idx = None;
                        } else {
                            self.account_idx = Some(account_idx + 1);
                        }

                        UserColumnEntry::Account(&accounts[account_idx])
                    }
                    Loadable::Error => UserColumnEntry::AccountError,
                };

                Some(ret_value)
            }
            None => {
                let ret_value = Some(UserColumnEntry::User(&ctx));

                if ctx.is_collapsed {
                    let no_users_left = ctx_idx >= self.column.users.len() - 1;
                    if no_users_left {
                        self.user_idx = None;
                        self.account_idx = None;
                    } else {
                        self.user_idx = Some(ctx_idx + 1);
                    }
                } else {
                    self.account_idx = Some(0);
                }

                ret_value
            }
        }
    }
}

pub enum UserColumnEntry<'a> {
    User(&'a UserCtx),
    Account(&'a AccountData),
    AccountNotLoaded,
    AccountLoading,
    AccountError,
}
