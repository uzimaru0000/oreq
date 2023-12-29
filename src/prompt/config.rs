use inquire::ui::{Color, RenderConfig, Styled};

pub fn render_config() -> RenderConfig {
    RenderConfig {
        prompt_prefix: Styled::new("*").with_fg(Color::LightGreen),
        password_mask: '#',
        ..Default::default()
    }
}

pub fn render_config_with_skkipable() -> RenderConfig {
    RenderConfig {
        canceled_prompt_indicator: Styled::new("Skipped").with_fg(Color::LightRed),
        prompt_prefix: Styled::new("?").with_fg(Color::LightGreen),
        password_mask: '#',
        ..Default::default()
    }
}
