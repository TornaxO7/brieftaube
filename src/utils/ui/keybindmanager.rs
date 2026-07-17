use chumsky::{Parser, prelude::*};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum Entry<A> {
    NotFinished,
    Action(A),
}

#[derive(Debug)]
pub struct KeybindManager<A> {
    mapping: Vec<HashMap<KeyEvent, Entry<A>>>,
    idx: usize,
}

impl<A: Clone + std::fmt::Debug> KeybindManager<A> {
    pub fn new<S: AsRef<str>>(raw_mapping: HashMap<S, A>) -> Self {
        let mapping = {
            let mut mapping = Vec::new();

            for (key, value) in raw_mapping.into_iter() {
                let keybinding = parse_keybinding().parse(key.as_ref()).unwrap();
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

        Self { mapping, idx: 0 }
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> Option<A> {
        let next_map = &self.mapping[self.idx];

        match next_map.get(&event) {
            Some(entry) => match entry {
                Entry::NotFinished => self.idx += 1,
                Entry::Action(a) => {
                    self.idx = 0;
                    return Some(a.clone());
                }
            },
            None => {
                self.idx = 0;
            }
        }

        None
    }
}

fn parse_keybinding<'src>()
-> impl Parser<'src, &'src str, Vec<KeyEvent>, chumsky::extra::Err<Rich<'src, char>>> {
    choice((
        keybinding_special(),
        keybinding_with_modifier(),
        keybinding_char(),
    ))
    .validate(|event, e, emitter| {
        if event.code == KeyCode::Esc {
            emitter.emit(Rich::custom(
                e.span(),
                format!("You are not allowed to map the escape key!"),
            ))
        }
        event
    })
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
        .ignore_then(choice((just("CR"), just("BS"), just("ESC"))))
        .then_ignore(just('>'))
        .map(|s| {
            let code = match s {
                "CR" => KeyCode::Enter,
                "BS" => KeyCode::Backspace,
                "ESC" => KeyCode::Esc,
                _ => todo!(),
            };

            KeyEvent::new(code, KeyModifiers::NONE)
        })
}

fn keybinding_with_modifier<'src>()
-> impl Parser<'src, &'src str, KeyEvent, chumsky::extra::Err<Rich<'src, char>>> {
    just('<')
        .ignore_then(one_of("CA"))
        .then(just('-').ignored())
        .then(one_of('a'..='z'))
        .then_ignore(just('>'))
        .map(|((special, _), c)| {
            let modifiers = match special {
                'C' => KeyModifiers::CONTROL,
                'A' => KeyModifiers::ALT,
                _ => todo!(),
            };

            let code = KeyCode::Char(c);

            KeyEvent::new(code, modifiers)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_keybinding() {
        assert_eq!(
            parse_keybinding().parse("<C-n>abc").unwrap(),
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
            parse_keybinding().parse("<<>>").unwrap(),
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
            parse_keybinding().parse("<A-s>").unwrap(),
            vec![KeyEvent::new(KeyCode::Char('s'), KeyModifiers::ALT)]
        );
    }

    #[test]
    #[should_panic]
    fn esc_keybinding() {
        parse_keybinding().parse("<ESC>").unwrap();
    }
}
