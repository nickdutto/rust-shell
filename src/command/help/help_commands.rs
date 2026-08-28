use crate::engine::call::Call;
use crate::engine::command::category::Category;
use crate::engine::command::signature::Signature;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::value::syntax_shape::SyntaxShape;
use comfy_table::modifiers::{UTF8_ROUND_CORNERS, UTF8_SOLID_INNER_BORDERS};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, CellAlignment, Color, Table};
use std::io::Write;

pub struct HelpCommands;

impl Command for HelpCommands {
    fn name(&self) -> &'static str {
        "help commands"
    }

    fn description(&self) -> &'static str {
        "find commands"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name()).category(Category::Help).named(
            "category",
            SyntaxShape::String,
            "category filter",
            Some('c'),
        )
    }

    fn run(
        &self,
        call: Call,
        engine_state: &EngineState,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let mut table = Table::new();

        table.load_preset(UTF8_FULL);
        table.apply_modifier(UTF8_SOLID_INNER_BORDERS);
        table.apply_modifier(UTF8_ROUND_CORNERS);

        table.set_header(vec![
            Cell::new("name")
                .fg(Color::Green)
                .set_alignment(CellAlignment::Center),
            Cell::new("category")
                .fg(Color::Green)
                .set_alignment(CellAlignment::Center),
            Cell::new("command_type")
                .fg(Color::Green)
                .set_alignment(CellAlignment::Center),
            Cell::new("description")
                .fg(Color::Green)
                .set_alignment(CellAlignment::Center),
        ]);

        let mut commands = engine_state
            .command_registry
            .commands
            .values()
            .map(|v| (v.name(), v.description(), v.command_type(), v.signature()))
            .collect::<Vec<_>>();

        if let Some(category) = call.opt_named::<String>("category")? {
            commands = commands
                .into_iter()
                .filter(|(_, _, _, s)| s.category.to_string() == category)
                .collect::<Vec<_>>();
        }

        commands.sort_by(|a, b| a.0.cmp(b.0));

        for (name, description, command_type, signature) in commands {
            table.add_row(vec![
                name.to_string(),
                signature.category.to_string(),
                command_type.to_string(),
                description.to_string(),
            ]);
        }

        writeln!(io_streams.output, "{table}")?;

        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}
