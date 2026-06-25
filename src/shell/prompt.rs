use crate::shell::config::{Config, DEFAULT_DATETIME_FORMAT, PromptMode, PromptSegment};
use jiff::Zoned;
use nu_ansi_term::{Color, Style};
use reedline::DefaultPrompt;
use reedline::{Prompt, PromptEditMode, PromptHistorySearch};
use std::borrow::Cow;
use std::env;

pub struct ShellPrompt {
    config: Config,
    username: String,
}

impl ShellPrompt {
    pub fn new(config: Config, username: String) -> Self {
        Self { config, username }
    }
}

impl Prompt for ShellPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        let prompt_value = ShellPrompt::get_prompt_mode_value(&self.config.prompt.left);
        let prompt = Style::new()
            .fg(Color::Fixed(self.config.theme.colors.prompt_left))
            .paint(prompt_value)
            .to_string();

        Cow::Owned(prompt)
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        let prompt_value = ShellPrompt::get_prompt_mode_value(&self.config.prompt.right);
        let prompt = Style::new()
            .fg(Color::Fixed(self.config.theme.colors.prompt_right))
            .paint(prompt_value)
            .to_string();

        Cow::Owned(prompt)
    }

    fn render_prompt_indicator(&self, prompt_mode: PromptEditMode) -> Cow<'_, str> {
        Cow::Owned(
            DefaultPrompt::default()
                .render_prompt_indicator(prompt_mode)
                .into_owned(),
        )
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Owned(
            DefaultPrompt::default()
                .render_prompt_multiline_indicator()
                .into_owned(),
        )
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<'_, str> {
        Cow::Owned(
            DefaultPrompt::default()
                .render_prompt_history_search_indicator(history_search)
                .into_owned(),
        )
    }
}

impl ShellPrompt {
    fn get_prompt_mode_value(prompt_segment: &PromptSegment) -> String {
        match prompt_segment.mode {
            PromptMode::Basic => prompt_segment.basic_value.clone().unwrap_or_default(),
            PromptMode::CurrentDirectory => {
                env::current_dir().unwrap_or_default().display().to_string()
            }
            PromptMode::DateTime => Zoned::now()
                .strftime(
                    &prompt_segment
                        .datetime_format
                        .clone()
                        .unwrap_or(DEFAULT_DATETIME_FORMAT.into())
                        .to_string(),
                )
                .to_string(),
            PromptMode::Empty => "".to_string(),
        }
    }
}
