use promptuity::{style::Styled, PromptBody, PromptInput};

pub fn fmt_body(body: &PromptBody) -> String {
    match body {
        PromptBody::Raw(s) => s.to_owned(),
        _ => String::new(),
    }
}

pub fn fmt_input(input: &PromptInput) -> String {
    match input {
        PromptInput::Raw(s) => s.to_owned(),
        PromptInput::Cursor(c) => {
            let (left, cursor, right) = c.split();
            format!("{left}{}{right}", Styled::new(cursor).rev())
        }
        PromptInput::None => String::new(),
    }
}
