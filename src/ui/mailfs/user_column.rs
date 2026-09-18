use crate::{
    CONFIG,
    config::{self, UserConfig},
    types::AccountData,
    ui::{Loadable, mailfs::MailfsColumn},
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

    pub fn get_selected_entry<'a>(&'a self) -> Option<UserColumnEntry<'a>> {
        let selected_idx = self.state.selected()?;

        let mut idx = 0;
        for user in self.users.iter() {
            if idx == selected_idx {
                return Some(UserColumnEntry::User(user));
            }

            idx += 1;

            match &user.accounts {
                Loadable::NotLoaded => {
                    if idx == selected_idx {
                        return Some(UserColumnEntry::AccountNotLoaded);
                    }

                    idx += 1;
                }
                Loadable::Loading => {
                    if idx == selected_idx {
                        return Some(UserColumnEntry::AccountLoading);
                    }

                    idx += 1;
                }
                Loadable::Error => {
                    if idx == selected_idx {
                        return Some(UserColumnEntry::AccountError);
                    }

                    idx += 1;
                }
                Loadable::Loaded(accounts) => {
                    for account in accounts.iter() {
                        if idx == selected_idx {
                            return Some(UserColumnEntry::Account(account));
                        }

                        idx += 1;
                    }
                }
            }
        }

        None
    }

    pub fn get_selected_entry_mut<'a>(&'a mut self) -> Option<UserColumnEntryMut<'a>> {
        let selected_idx = self.state.selected()?;

        let mut idx = 0;
        for user in self.users.iter_mut() {
            if idx == selected_idx {
                return Some(UserColumnEntryMut::User(user));
            }

            idx += 1;

            match &mut user.accounts {
                Loadable::NotLoaded => {
                    if idx == selected_idx {
                        return Some(UserColumnEntryMut::AccountNotLoaded);
                    }

                    idx += 1;
                }
                Loadable::Loading => {
                    if idx == selected_idx {
                        return Some(UserColumnEntryMut::AccountLoading);
                    }

                    idx += 1;
                }
                Loadable::Error => {
                    if idx == selected_idx {
                        return Some(UserColumnEntryMut::AccountError);
                    }

                    idx += 1;
                }
                Loadable::Loaded(accounts) => {
                    for account in accounts.iter_mut() {
                        if idx == selected_idx {
                            return Some(UserColumnEntryMut::Account(account));
                        }

                        idx += 1;
                    }
                }
            }
        }

        None
    }

    pub fn navigate_right(&mut self) -> Vec<crate::ui::Message> {
        let Some(selected_entry) = self.get_selected_entry_mut() else {
            return vec![];
        };

        match selected_entry {
            UserColumnEntryMut::User(user_ctx) => {
                if user_ctx.is_collapsed {
                    user_ctx.is_collapsed = false;
                }

                if matches!(user_ctx.accounts, Loadable::NotLoaded) {
                    user_ctx.accounts = Loadable::Loading;
                    return vec![
                        super::MessageRequest::GetAccountsOf(user_ctx.config.username.clone())
                            .into(),
                    ];
                }

                vec![]
            }
            UserColumnEntryMut::Account(account_data) => todo!(),
            UserColumnEntryMut::AccountNotLoaded => todo!(),
            UserColumnEntryMut::AccountLoading => todo!(),
            UserColumnEntryMut::AccountError => todo!(),
        }
    }

    pub fn navigate_left(&mut self) -> Vec<crate::ui::Message> {
        todo!()
    }
}

impl MailfsColumn for UserColumn {
    fn navigate_up(&mut self) -> Vec<crate::ui::Message> {
        self.state.select_previous();
        vec![]
    }

    fn navigate_down(&mut self) -> Vec<crate::ui::Message> {
        self.state.select_next();
        vec![]
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

pub enum UserColumnEntry<'a> {
    User(&'a UserCtx),
    Account(&'a AccountData),
    AccountNotLoaded,
    AccountLoading,
    AccountError,
}

pub enum UserColumnEntryMut<'a> {
    User(&'a mut UserCtx),
    Account(&'a mut AccountData),
    AccountNotLoaded,
    AccountLoading,
    AccountError,
}
