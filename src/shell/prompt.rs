use crate::shell::config::{Config, DEFAULT_DATETIME_FORMAT, PromptMode, PromptSegment};
use jiff::Zoned;
use nu_ansi_term::{Color, Style};
use reedline::{DefaultPrompt, PromptViMode};
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
        let mut prompt = String::new();

        let iter = self
            .config
            .prompt
            .left
            .iter()
            .filter(|s| s.mode != PromptMode::Empty)
            .peekable();

        for segment in iter {
            let prompt_value = self.get_prompt_mode_value(segment);
            let bg = Color::Fixed(segment.background);
            let fg = Color::Fixed(segment.foreground);
            let arrow_symbol = "\u{e0b0}";
            let segment_style = if segment.bold {
                Style::new().fg(fg).on(bg).bold()
            } else {
                Style::new().fg(fg).on(bg)
            };

            if segment.arrow_left && segment.gap {
                prompt.push_str(
                    &Style::new()
                        .fg(Color::Fixed(segment.arrow_left_color))
                        .reverse()
                        .paint(arrow_symbol)
                        .to_string(),
                );
            } else if segment.arrow_left && !segment.gap {
                prompt.push_str(
                    &Style::new()
                        .fg(bg)
                        .on(Color::Fixed(segment.arrow_left_color))
                        .reverse()
                        .paint(arrow_symbol)
                        .to_string(),
                );
            } else if !segment.arrow_left && segment.gap {
                prompt.push(' ');
            }

            prompt.push_str(&segment_style.paint(format!(" {prompt_value} ")).to_string());

            if segment.arrow_right {
                prompt.push_str(
                    &Style::new()
                        .fg(Color::Fixed(segment.arrow_right_color))
                        .paint(arrow_symbol)
                        .to_string(),
                );
            }
        }

        Cow::Owned(prompt)
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        let mut prompt = String::new();

        let iter = self
            .config
            .prompt
            .right
            .iter()
            .filter(|s| s.mode != PromptMode::Empty)
            .peekable();

        for segment in iter {
            let prompt_value = self.get_prompt_mode_value(segment);
            let bg = Color::Fixed(segment.background);
            let fg = Color::Fixed(segment.foreground);
            let arrow_symbol = "\u{e0b2}";
            let segment_style = if segment.bold {
                Style::new().fg(fg).on(bg).bold()
            } else {
                Style::new().fg(fg).on(bg)
            };

            if segment.arrow_left && segment.gap {
                prompt.push_str(
                    &Style::new()
                        .fg(Color::Fixed(segment.arrow_left_color))
                        .paint(arrow_symbol)
                        .to_string(),
                );
            } else if segment.arrow_left && !segment.gap {
                prompt.push_str(
                    &Style::new()
                        .fg(bg)
                        .on(Color::Fixed(segment.arrow_left_color))
                        .paint(arrow_symbol)
                        .to_string(),
                );
            } else if !segment.arrow_left && segment.gap {
                prompt.push(' ');
            }

            prompt.push_str(&segment_style.paint(format!(" {prompt_value} ")).to_string());

            if segment.arrow_right {
                prompt.push_str(
                    &Style::new()
                        .fg(Color::Fixed(segment.arrow_right_color))
                        .reverse()
                        .paint(arrow_symbol)
                        .to_string(),
                );
            }
        }

        Cow::Owned(prompt)
    }

    fn render_prompt_indicator(&self, prompt_mode: PromptEditMode) -> Cow<'_, str> {
        match prompt_mode {
            PromptEditMode::Default | PromptEditMode::Emacs => Cow::Borrowed(" "),
            PromptEditMode::Vi(vi_mode) => match vi_mode {
                PromptViMode::Normal => Cow::Owned("〉".into()),
                PromptViMode::Insert => Cow::Owned(": ".into()),
            },
            _ => Cow::Borrowed(" "),
        }
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
    fn get_prompt_mode_value(&self, prompt_segment: &PromptSegment) -> String {
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
            PromptMode::Username => self.username.to_string(),
            PromptMode::Empty => "".to_string(),
        }
    }
}
