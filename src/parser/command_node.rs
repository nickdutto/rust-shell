use crate::io::redirection::RedirectionMode;
use crate::parser::span::Spanned;
use crate::parser::word::Word;

#[derive(Debug, Default, PartialEq)]
pub struct Redirection {
    pub mode: RedirectionMode,
    pub path: Vec<Spanned<Word>>,
}

#[derive(Debug, Default, PartialEq)]
pub struct CommandNode {
    pub cmd: Vec<Spanned<Word>>,
    pub args: Vec<Vec<Spanned<Word>>>,
    pub redirection: Redirection,
}

impl CommandNode {
    pub fn to_command_string(&self) -> String {
        let mut buffer = String::new();

        let original_words = |words: &[Spanned<Word>]| -> String {
            words
                .iter()
                .map(|s| s.item.to_original_string())
                .collect::<String>()
        };

        buffer.push_str(&original_words(&self.cmd));
        for arg in &self.args {
            buffer.push(' ');
            buffer.push_str(&original_words(arg));
        }

        match self.redirection.mode {
            // TODO: technically this is not correct if the output where set with 1>/1>>
            RedirectionMode::Out => buffer.push_str(" > "),
            RedirectionMode::OutAppend => buffer.push_str(" >> "),
            RedirectionMode::Error => buffer.push_str(" 2> "),
            RedirectionMode::ErrorAppend => buffer.push_str(" 2>> "),
            RedirectionMode::Nothing => {}
        }

        if self.redirection.mode != RedirectionMode::Nothing {
            buffer.push_str(&original_words(&self.redirection.path));
        }

        buffer
    }
}
