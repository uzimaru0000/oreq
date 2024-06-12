use inquire::{error::InquireResult, CustomType, DateSelect, Password, Select};
use openapiv3::{StringFormat, StringType, VariantOrUnknownOrEmpty};
use serde_json::{json, Value};

use super::{
    config::{render_config, render_config_with_skkipable},
    Prompt,
};

pub struct StringPrompt<'a> {
    message: &'a str,
    description: Option<&'a str>,
    string: &'a StringType,
}

impl<'a> StringPrompt<'a> {
    pub fn new(message: &'a str, description: Option<&'a str>, string: &'a StringType) -> Self {
        Self {
            message,
            description,
            string,
        }
    }

    fn create_select_prompt(&self) -> Option<Select<Value>> {
        let enumeration = self
            .string
            .enumeration
            .iter()
            .map(|x| x.clone().map(|x| json!(x)))
            .filter_map(|x| x.to_owned())
            .collect::<Vec<_>>();

        if !enumeration.is_empty() {
            let mut prompt = Select::new(self.message, enumeration);
            prompt.help_message = self.description;

            Some(prompt)
        } else {
            None
        }
    }

    fn create_prompt(&self) -> CustomType<Value> {
        let mut prompt = CustomType::new(self.message).with_parser(&|x| Ok(json!(x)));
        prompt.help_message = self.description;

        prompt
    }

    fn create_password_prompt(&self) -> Option<Password> {
        if let VariantOrUnknownOrEmpty::Item(StringFormat::Password) = self.string.format {
            let mut prompt = Password::new(&self.message)
                .with_display_mode(inquire::PasswordDisplayMode::Masked)
                .without_confirmation();
            prompt.help_message = self.description;

            Some(prompt)
        } else {
            None
        }
    }

    fn create_date_prompt(&self) -> Option<DateSelect> {
        if let VariantOrUnknownOrEmpty::Item(StringFormat::Date) = self.string.format {
            let mut prompt = DateSelect::new(&self.message)
                .with_vim_mode(true)
                .with_week_start(chrono::Weekday::Sun);
            prompt.help_message = self.description;

            Some(prompt)
        } else {
            None
        }
    }
}

impl<'a> Prompt for StringPrompt<'a> {
    fn prompt(&self) -> InquireResult<Value> {
        let select = self.create_select_prompt();
        let password = self.create_password_prompt();
        let date = self.create_date_prompt();

        if let Some(select) = select {
            select.with_render_config(render_config()).prompt()
        } else if let Some(password) = password {
            password
                .with_render_config(render_config())
                .prompt()
                .map(|x| json!(x))
        } else if let Some(date) = date {
            date.with_render_config(render_config())
                .prompt()
                .map(|x| json!(x.to_string()))
        } else {
            self.create_prompt()
                .with_render_config(render_config())
                .prompt()
        }
    }

    fn prompt_skippable(&self) -> InquireResult<Option<Value>> {
        let select = self.create_select_prompt();
        let password = self.create_password_prompt();
        let date = self.create_date_prompt();

        if let Some(select) = select {
            select
                .with_render_config(render_config_with_skkipable())
                .prompt_skippable()
        } else if let Some(password) = password {
            password
                .with_render_config(render_config_with_skkipable())
                .prompt_skippable()
                .map(|x| x.map(|x| json!(x)))
        } else if let Some(date) = date {
            date.with_render_config(render_config_with_skkipable())
                .prompt_skippable()
                .map(|x| x.map(|x| json!(x.to_string())))
        } else {
            self.create_prompt()
                .with_render_config(render_config_with_skkipable())
                .prompt_skippable()
        }
    }
}

#[cfg(test)]
#[cfg(feature = "manual")]
mod tests {
    use indoc::indoc;
    use openapiv3::StringType;
    use serde_json::json;

    use crate::prompt::Prompt;

    use super::StringPrompt;

    #[test]
    fn test_string_prompt_enum() {
        let schema = indoc! {"
            type: string
            enum:
                - foo
                - bar
        "};
        let schema = serde_yaml::from_str::<StringType>(schema).unwrap();

        let prompt = StringPrompt::new("select foo", None, &schema);
        let res = prompt.prompt().unwrap();

        assert_eq!(res, json!("foo"));
    }

    #[test]
    fn test_string_prompt_skippable() {
        let schema = indoc! {"
            type: string
            enum:
                - foo
                - bar
        "};
        let schema = serde_yaml::from_str::<StringType>(schema).unwrap();

        let prompt = StringPrompt::new("skip", None, &schema);
        let res = prompt.prompt_skippable().unwrap();

        assert_eq!(res, None);
    }

    #[test]
    fn test_string_prompt_simple() {
        let schema = indoc! {"
            type: string
        "};
        let schema = serde_yaml::from_str::<StringType>(schema).unwrap();

        let prompt = StringPrompt::new("string", None, &schema);
        let res = prompt.prompt().unwrap();

        assert!(res.is_string());
    }
}
