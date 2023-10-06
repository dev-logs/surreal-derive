use proc_macro::TokenStream;

enum Matching {
    STARTED,
    REFERENCE(String),
    METHOD(String),
}

enum Matched {
    REFERENCE(String),
    METHOD(String),
}

enum State {
    NORMAL,
    MATCHING(Matching),
    MATCHED(Matched),
}

#[proc_macro]
pub fn surreal_quote(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::LitStr);
    let mut current_state = State::NORMAL;

    let it = input.value().char_indices().into_iter();
    for (i, c) in it {
        current_state = match current_state {
            State::NORMAL => match c {
                char::from("#") => State::MATCHING(Matching::STARTED),
                _ => State::NORMAL
            },
            State::MATCHING(cc) => matching()
            State::MATCHED(_) => {}
        };
    }

    TokenStream::new()
}

fn matching(current_state: Matching, c: char) -> State {
    match current_state {
        Matching::STARTED =>  match c {
            char::from("#") => panic!(),
            char::from("(") => State::MATCHING(Matching::METHOD());
        }
        Matching::REFERENCE(_) => {}
        Matching::METHOD(_) => {}
    }
}
