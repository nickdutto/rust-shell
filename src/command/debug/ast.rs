use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::format::debug::highlight_debug;
use crate::io::stream::IoStreams;
use crate::parser::Parser;
use crate::parser::lexer::lex;
use crate::parser::span::Spanned;
use crate::shell::shell_state::ShellState;
use std::io::Write;
use std::sync::{Arc, RwLock};

pub struct Ast;

impl Command for Ast {
    fn name(&self) -> &'static str {
        "ast"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn run(
        &self,
        cmd: Spanned<String>,
        args: Vec<Spanned<String>>,
        _job_id: Option<usize>,
        _config: Arc<Config>,
        _shell_state: Arc<RwLock<ShellState>>,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let mut args_iter = args.iter();
        let mut pretty_print = false;

        let Some(line_arg) = args_iter.next() else {
            return Err(ShellError::CommandArgument {
                span: cmd.span,
                help: "Add line to get AST statements inside quotes. Example: 'echo \"hello \\\"world\\\"\" && ls -la'"
                    .to_owned(),
                label: "Empty `line` argument".to_owned(),
            });
        };

        for arg in args_iter {
            if arg.item.as_str() == "-pretty" {
                pretty_print = true;
            }
        }

        let tokens = lex(&line_arg.item);
        let statements = Parser::new(tokens).parse_statements();

        let formatted = if pretty_print {
            format!("{statements:#?}")
        } else {
            format!("{statements:?}")
        };

        let styled = highlight_debug(&formatted);
        writeln!(io_streams.output, "{}", styled.render_simple())?;

        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}
