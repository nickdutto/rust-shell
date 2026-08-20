use crate::engine::call::Call;
use crate::engine::command::category::Category;
use crate::engine::command::signature::Signature;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use comfy_table::presets::NOTHING;
use comfy_table::{Attribute, Cell, ContentArrangement, Table};
use std::io::Write;
use sysinfo::System;

pub struct SysProcess;

impl Command for SysProcess {
    fn name(&self) -> &'static str {
        "sys process"
    }

    fn description(&self) -> &'static str {
        "System process information"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name()).category(Category::System)
    }

    fn run(
        &self,
        _call: Call,
        _engine_state: &EngineState,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        writeln!(io_streams.output, "{}", Self::process_table())?;

        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}

impl SysProcess {
    fn process_table() -> Table {
        let mut sys = System::new_all();
        sys.refresh_all();

        let mut table = Table::new();

        table
            .load_preset(NOTHING)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec![
                Cell::new("pid").add_attribute(Attribute::Bold),
                Cell::new("name").add_attribute(Attribute::Bold),
            ]);

        for (pid, process) in sys.processes() {
            table.add_row([
                pid.to_string(),
                process.name().to_string_lossy().to_string(),
            ]);
        }

        table
    }
}
