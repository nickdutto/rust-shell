use crate::engine::call::Call;
use crate::engine::command::category::Category;
use crate::engine::command::signature::Signature;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use comfy_table::presets::NOTHING;
use comfy_table::{ContentArrangement, Table};
use std::io::Write;
use sysinfo::System;

pub struct SysOs;

impl Command for SysOs {
    fn name(&self) -> &'static str {
        "sys os"
    }

    fn description(&self) -> &'static str {
        "System OS information"
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
        writeln!(io_streams.output, "{}", Self::os_table())?;

        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}

impl SysOs {
    fn os_table() -> Table {
        let mut table = Table::new();

        table
            .load_preset(NOTHING)
            .set_content_arrangement(ContentArrangement::Dynamic);

        table.add_row([
            String::from("System name"),
            System::name().unwrap_or_default(),
        ]);

        table.add_row([
            String::from("OS Version"),
            System::os_version().unwrap_or_default(),
        ]);

        table.add_row([
            String::from("Kernel Version"),
            System::kernel_version().unwrap_or_default(),
        ]);

        table.add_row([String::from("CPU Architecture"), System::cpu_arch()]);

        table.add_row([
            String::from("Host name"),
            System::host_name().unwrap_or_default(),
        ]);

        table
    }
}
