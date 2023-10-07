use proc_macro::TokenStream;
use syn::Expr::Match;
use crate::surrealql_machine::State::MATCHING;

#[derive(Clone)]
enum Matching {
    STARTED,
    REFERENCE(String),
    METHOD(String),
}

#[derive(Clone)]
enum Matched {
    REFERENCE(String),
    METHOD(String),
}

#[derive(Clone)]
enum State {
    NORMAL,
    // Matching State and true/false to say whether it is bounded inside a ()
    MATCHING(Matching, bool),
    MATCHED(Matched),
}

#[proc_macro]
pub fn surreal_quote(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::LitStr);
    let mut current_state = State::NORMAL;
    let mut final_states: Vec<Matched> = vec![];

    let it = input.value().char_indices().into_iter();
    for (i, c) in it {
        current_state = match current_state {
            State::NORMAL => match c {
                '#' => State::MATCHING(Matching::STARTED, false),
                _ => State::NORMAL
            },
            State::MATCHING(matching, is_blocked) => match matching {
                Matching::STARTED => match c {
                    '#' => panic!("Can not place '#' after '#'"),
                    '(' => State::MATCHING(Matching::STARTED, true),
                    c if (a..z).contains(&c) || (A..Z).contains(&c) || (c == "_") => State::MATCHING(Matching::REFERENCE(c.to_string()), is_blocked),
                    _ => {}
                }
                Matching::REFERENCE(str) => match c {
                    '(' => MATCHING(Matching::METHOD(format!("{}{}", str, c)), is_blocked),
                    _ => {}
                },
                Matching::METHOD(str) => match c {
                    ')' => State::MATCHED(Matched::METHOD(format!("{}{}", str, ")"))),
                    _ => {}
                },
            },
            State::MATCHED(f) => match f {
                Matched::REFERENCE(_) => match c {
                    ' ' => {
                        final_states.push(f);
                        State::NORMAL
                    }
                    _ => {}
                }
                Matched::METHOD(_) => match c {
                    _ => {}
                }
            },
        };

        if State::MATCHED(matched_state) == current_state {
            final_states.insert(matched_state.clone());
        }
    }

    TokenStream::new()
}
