use crate::engine::call::Call;
use crate::engine::command::category::Category;
use crate::engine::command::signature::Signature;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::format::debug::highlight_debug;
use crate::io::stream::IoStreams;
use crate::parser::lexer::lex;
use crate::value::syntax_shape::SyntaxShape;
use std::io::Write;

pub struct Lex;

impl Command for Lex {
    fn name(&self) -> &'static str {
        "lex"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name())
            .category(Category::Debug)
            .required_positional("line", SyntaxShape::String, "Line to build lex tokens from")
            .switch("pretty", "Pretty print", Some('p'))
    }

    fn run(
        &self,
        call: Call,
        _engine_state: &EngineState,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let line = call.req::<String>(0)?;
        let pretty_print = call.has_switch("pretty");

        let tokens = lex(&line);
        let formatted = if pretty_print {
            format!("{tokens:#?}")
        } else {
            format!("{tokens:?}")
        };

        let styled = highlight_debug(&formatted);
        writeln!(io_streams.output, "{}", styled.render_simple())?;

        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}
