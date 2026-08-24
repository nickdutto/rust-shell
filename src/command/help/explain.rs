use crate::config::Config;
use crate::engine::call::Call;
use crate::engine::command::category::Category;
use crate::engine::command::signature::Signature;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::command_registry::BUILTIN_COMMANDS;
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::redirection::RedirectionMode;
use crate::io::stream::IoStreams;
use crate::parser::lexer::{TokenKind, lex};
use crate::parser::span::Spanned;
use crate::parser::syntax_shape::SyntaxShape;
use crate::parser::word::Word;
use crate::shell::highlighter::SyntaxHighlighter;
use nu_ansi_term::{AnsiGenericString, Color, Style};
use std::io::Write;
use std::sync::Arc;

pub struct Explain;

impl Command for Explain {
    fn name(&self) -> &'static str {
        "explain"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name())
            .category(Category::Help)
            .required_positional("line", SyntaxShape::String, "Line to be explained")
    }

    fn run(
        &self,
        call: Call,
        engine_state: &EngineState,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let line = call.req::<String>(0)?;

        let highlighter = SyntaxHighlighter::new(engine_state.config.clone(), BUILTIN_COMMANDS);
        let tokens = lex(&line);

        let mut command_id = 1;
        let mut is_first_word = true;
        let mut is_redirect_word = false;
        let mut explain_buffer = vec![];

        for token in tokens {
            match token.kind {
                TokenKind::Word(words) => match_word(
                    &mut explain_buffer,
                    words,
                    command_id,
                    &mut is_first_word,
                    is_redirect_word,
                    &highlighter,
                ),

                TokenKind::Redirection(mode) => match_redirection(
                    &mut explain_buffer,
                    &mode,
                    command_id,
                    &mut is_first_word,
                    &mut is_redirect_word,
                    &engine_state.config,
                ),

                flag @ (TokenKind::Pipe
                | TokenKind::Sequential
                | TokenKind::And
                | TokenKind::Background) => match_statement(
                    &mut explain_buffer,
                    &flag,
                    &mut command_id,
                    &mut is_first_word,
                    &mut is_redirect_word,
                    &engine_state.config,
                ),
            }
        }

        writeln!(
            io_streams.output,
            "\n{}",
            explain_buffer
                .iter()
                .map(ToString::to_string)
                .collect::<String>()
        )?;

        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}

fn match_word(
    explain_buffer: &mut Vec<AnsiGenericString<str>>,
    words: Vec<Spanned<Word>>,
    command_id: i32,
    is_first_word: &mut bool,
    is_redirect_word: bool,
    syntax_highlighter: &SyntaxHighlighter,
) {
    let word_style = Style::new();
    let cmd_style = Style::new().fg(Color::Fixed(64));

    for word in words {
        if *is_first_word && !is_redirect_word {
            explain_buffer.push(cmd_style.paint(format!("  cmd {command_id:<3}")));
        }

        if is_redirect_word {
            explain_buffer.push(cmd_style.paint(format!("  {:<7}", "file")));
        }

        if *is_first_word {
            *is_first_word = false;
        }

        explain_buffer.extend([
            word_style.paint(
                syntax_highlighter
                    .highlight_tokens(word.item.to_original_string().as_str())
                    .render_simple(),
            ),
            word_style.paint(" "),
        ]);
    }
}

fn match_redirection(
    explain_buffer: &mut Vec<AnsiGenericString<str>>,
    redirection_mode: &RedirectionMode,
    command_id: i32,
    is_first_word: &mut bool,
    is_redirect_word: &mut bool,
    config: &Arc<Config>,
) {
    let explain_str = format!("redirect cmd {command_id} output to file");
    let explain_color_redirection = Color::Fixed(126);

    *is_first_word = true;
    *is_redirect_word = true;

    match redirection_mode {
        RedirectionMode::Out => {
            explain_human_token_kind(
                explain_buffer,
                explain_str.as_str(),
                "> OR 1>",
                "RedirectionMode",
                "Out",
                config.theme.colors.redirection_out,
                explain_color_redirection,
            );
        }

        RedirectionMode::OutAppend => {
            explain_human_token_kind(
                explain_buffer,
                explain_str.as_str(),
                ">> OR 1>>",
                "RedirectionMode",
                "OutAppend",
                config.theme.colors.redirection_out_append,
                explain_color_redirection,
            );
        }

        RedirectionMode::Error => {
            explain_human_token_kind(
                explain_buffer,
                explain_str.as_str(),
                "2>",
                "RedirectionMode",
                "Error",
                config.theme.colors.redirection_error,
                explain_color_redirection,
            );
        }

        RedirectionMode::ErrorAppend => {
            explain_human_token_kind(
                explain_buffer,
                explain_str.as_str(),
                "2>>",
                "RedirectionMode",
                "ErrorAppend",
                config.theme.colors.redirection_error_append,
                explain_color_redirection,
            );
        }

        RedirectionMode::Nothing => {}
    }
}

fn match_statement(
    explain_buffer: &mut Vec<AnsiGenericString<str>>,
    token_kind: &TokenKind,
    command_id: &mut i32,
    is_first_word: &mut bool,
    is_redirect_word: &mut bool,
    config: &Arc<Config>,
) {
    let explain_color_statement = Color::Fixed(26);

    *command_id += 1;
    *is_first_word = true;
    *is_redirect_word = false;

    match token_kind {
        TokenKind::Pipe => {
            explain_human_token_kind(
                explain_buffer,
                format!(
                    "pipe cmd {} output into cmd {command_id} input",
                    *command_id - 1
                )
                .as_str(),
                "|",
                "TokenKind",
                "Pipe",
                config.theme.colors.pipe,
                explain_color_statement,
            );
        }

        TokenKind::Sequential => {
            explain_human_token_kind(
                explain_buffer,
                format!("when cmd {} exit run cmd {command_id}", *command_id - 1).as_str(),
                ";",
                "TokenKind",
                "Sequential",
                config.theme.colors.sequential,
                explain_color_statement,
            );
        }

        TokenKind::And => {
            explain_human_token_kind(
                explain_buffer,
                format!(
                    "if cmd {} exit success run cmd {command_id}",
                    *command_id - 1
                )
                .as_str(),
                "&&",
                "TokenKind",
                "And",
                config.theme.colors.and,
                explain_color_statement,
            );
        }

        TokenKind::Background => {
            explain_human_token_kind(
                explain_buffer,
                "run line in background thread",
                "&",
                "TokenKind",
                "Background",
                config.theme.colors.background,
                explain_color_statement,
            );
        }

        TokenKind::Word(_) | TokenKind::Redirection(_) => {}
    }
}

fn explain_human_token_kind(
    buffer: &mut Vec<AnsiGenericString<str>>,
    explain_str: &str,
    symbol: &str,
    kind_name: &str,
    kind: &str,
    symbol_color: u8,
    explain_color: Color,
) {
    let symbol_style = Style::new().fg(Color::Fixed(symbol_color));
    let dimmed_style = Style::new().dimmed();

    buffer.extend([
        Style::new()
            .fg(explain_color)
            .paint(format!("\n{explain_str} ")),
        dimmed_style.paint("{ Symbol: "),
        symbol_style.paint(symbol.to_owned()),
        dimmed_style.paint(format!(", {kind_name}: ")),
        symbol_style.paint(kind.to_owned()),
        dimmed_style.paint(" }\n"),
    ]);
}
