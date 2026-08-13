use chumsky::{Parser, prelude::*};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum Entry<A> {
    NotFinished,
    Action(A),
}

pub enum HandleEvent<A> {
    Action(A),
    Registered,
    Cancel,
}

#[derive(Debug)]
pub struct KeybindManager<A> {
    int_prefix: String,
    mapping: Vec<HashMap<KeyEvent, Entry<A>>>,
    idx: usize,
}

impl<A: Clone + std::fmt::Debug> KeybindManager<A> {
    pub fn new<S: AsRef<str>>(raw_mapping: HashMap<S, A>) -> Self {
        let mapping = {
            let mut mapping = Vec::new();

            for (key, value) in raw_mapping.into_iter() {
                let keybinding = keybinding_parser().parse(key.as_ref()).unwrap();
                let keybinding_len = keybinding.len();

                if mapping.len() < keybinding_len {
                    mapping.resize(keybinding_len, HashMap::new());
                }

                // Mark the keybinding as "it's valid"
                for (idx, key) in keybinding.iter().enumerate() {
                    mapping[idx].insert(*key, Entry::NotFinished);
                }

                mapping[keybinding_len - 1]
                    .insert(*keybinding.last().unwrap(), Entry::Action(value));
            }

            mapping
        };

        Self {
            int_prefix: String::with_capacity(4),
            mapping,
            idx: 0,
        }
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> HandleEvent<A> {
        if let KeyCode::Char(c) = event.code
            && c.is_digit(10)
        {
            self.int_prefix.push(c);
            return HandleEvent::Registered;
        }

        let next_map = &self.mapping[self.idx];

        match next_map.get(&event) {
            Some(entry) => match entry {
                Entry::NotFinished => {
                    self.idx += 1;
                    HandleEvent::Registered
                }
                Entry::Action(a) => {
                    self.idx = 0;
                    HandleEvent::Action(a.clone())
                }
            },
            None => {
                self.idx = 0;
                self.int_prefix.clear();
                HandleEvent::Cancel
            }
        }
    }

    pub fn flush_int_prefix(&mut self) -> Option<usize> {
        let num = std::mem::take(&mut self.int_prefix);
        num.parse::<usize>().ok()
    }
}

fn keybinding_parser<'src>()
-> impl Parser<'src, &'src str, Vec<KeyEvent>, chumsky::extra::Err<Rich<'src, char>>> {
    choice((
        keybinding_special(),
        keybinding_with_modifier(),
        keybinding_char(),
    ))
    .repeated()
    .at_least(1)
    .collect()
}

fn keybinding_char<'src>()
-> impl Parser<'src, &'src str, KeyEvent, chumsky::extra::Err<Rich<'src, char>>> {
    any().filter(char::is_ascii).map(|c| {
        let modifiers = if c.is_uppercase() {
            KeyModifiers::SHIFT
        } else {
            KeyModifiers::NONE
        };

        let code = KeyCode::Char(c.to_ascii_lowercase());

        KeyEvent::new(code, modifiers)
    })
}

fn keybinding_special<'src>()
-> impl Parser<'src, &'src str, KeyEvent, chumsky::extra::Err<Rich<'src, char>>> {
    just('<')
        .ignore_then(
            one_of('a'..='z')
                .or(one_of('A'..='Z'))
                .repeated()
                .at_least(2)
                .collect::<String>(),
        )
        .then_ignore(just('>'))
        .try_map(|s, span| match s.to_lowercase().as_str() {
            "cr" => Ok(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            "bs" => Ok(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)),
            "esc" => Ok(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            "tab" => Ok(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
            "btab" => Ok(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT)),
            _ => Err(Rich::custom(span, "Not a known special key.")),
        })
}

fn keybinding_with_modifier<'src>()
-> impl Parser<'src, &'src str, KeyEvent, chumsky::extra::Err<Rich<'src, char>>> {
    just('<')
        .ignore_then(one_of("CAScas"))
        .then_ignore(just('-'))
        .then(one_of('a'..='z').or(one_of('A'..='Z')))
        .then_ignore(just('>'))
        .map(|(special, value): (char, char)| {
            let modifiers = match special.to_ascii_lowercase() {
                'c' => KeyModifiers::CONTROL,
                'a' => KeyModifiers::ALT,
                's' => KeyModifiers::SHIFT,
                _ => todo!(),
            };

            let code = KeyCode::Char(value);

            KeyEvent::new(code, modifiers)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_keybinding() {
        assert_eq!(
            keybinding_parser().parse("<C-n>abc").unwrap(),
            vec![
                KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL),
                KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE),
            ]
        );
    }

    #[test]
    fn greater_than_and_lower_than_symbols() {
        assert_eq!(
            keybinding_parser().parse("<<>>").unwrap(),
            vec![
                KeyEvent::new(KeyCode::Char('<'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('<'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('>'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('>'), KeyModifiers::NONE),
            ]
        );
    }

    #[test]
    fn alt_keybinding() {
        assert_eq!(
            keybinding_parser().parse("<A-s>").unwrap(),
            vec![KeyEvent::new(KeyCode::Char('s'), KeyModifiers::ALT)]
        );
    }

    #[test]
    fn tab() {
        assert_eq!(
            keybinding_parser().parse("<tab>").unwrap(),
            vec![KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)]
        );

        assert_eq!(
            keybinding_parser().parse("<TaB>").unwrap(),
            vec![KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)]
        );
    }

    #[test]
    fn backtab() {
        assert_eq!(
            keybinding_parser().parse("<btab>").unwrap(),
            vec![KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT)]
        );

        assert_eq!(
            keybinding_parser().parse("<bTAb>").unwrap(),
            vec![KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT)]
        );
    }
}
