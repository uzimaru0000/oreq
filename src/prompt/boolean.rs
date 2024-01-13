use inquire::{error::InquireResult, Confirm};
use serde_json::{json, Value};

use super::{
    config::{render_config, render_config_with_skkipable},
    Prompt,
};

pub struct BooleanPrompt<'a> {
    message: &'a str,
    description: Option<&'a str>,
}

impl<'a> BooleanPrompt<'a> {
    pub fn new(message: &'a str, description: Option<&'a str>) -> Self {
        Self {
            message,
            description,
        }
    }

    fn create_prompt(&self) -> Confirm {
        let mut prompt = Confirm::new(self.message).with_placeholder("y/n");
        prompt.help_message = self.description;

        prompt
    }
}

impl<'a> Prompt for BooleanPrompt<'a> {
    fn prompt(&self) -> InquireResult<Value> {
        self.create_prompt()
            .with_render_config(render_config())
            .prompt()
            .map(|x| json!(x))
    }

    fn prompt_skippable(&self) -> InquireResult<Option<Value>> {
        self.create_prompt()
            .with_render_config(render_config_with_skkipable())
            .prompt_skippable()
            .map(|x| x.map(|x| json!(x)))
    }
}

#[cfg(test)]
#[cfg(feature = "manual")]
mod tests {
    use serde_json::json;

    use super::BooleanPrompt;
    use crate::prompt::Prompt;

    #[test]
    fn test_boolean_prompt() {
        let prompt = BooleanPrompt::new("Do you like Rust?", None);
        let v = prompt.prompt().unwrap();
        assert_eq!(v, json!(true));
    }
}
