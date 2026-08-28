use crate::engine::call::Call;
use crate::engine::command::category::Category;
use crate::engine::command::signature::Signature;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::engine_state::EngineState;
use crate::engine::exit::ExitCode;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::parser::syntax_shape::SyntaxShape;
use crate::value::data_size_unit::{DataSizeUnit, convert_data_size_unit};
use comfy_table::presets::NOTHING;
use comfy_table::{ContentArrangement, Table};
use std::io::Write;
use sysinfo::System;

pub struct SysPerf;

impl Command for SysPerf {
    fn name(&self) -> &'static str {
        "sys perf"
    }

    fn description(&self) -> &'static str {
        "System hardware and performance information"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name())
            .category(Category::System)
            .named(
                "units",
                SyntaxShape::String,
                "Data size units (default GB).\
                Options (case insensitive): byte, KiB, MiB, GiB, TiB, KB, MB, GB, TB",
                Some('u'),
            )
    }

    fn run(
        &self,
        call: Call,
        _engine_state: &EngineState,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let units = call
            .opt_named::<DataSizeUnit>("units")?
            .unwrap_or(DataSizeUnit::GB);

        writeln!(io_streams.output, "{}", Self::perf_table(units))?;

        Ok(CommandData::ExitCode(ExitCode::SUCCESS))
    }
}

impl SysPerf {
    fn perf_table(units: DataSizeUnit) -> Table {
        let mut sys = System::new_all();
        sys.refresh_all();

        let mut table = Table::new();

        table
            .load_preset(NOTHING)
            .set_content_arrangement(ContentArrangement::Dynamic);

        table.add_row([String::from("CPUs"), sys.cpus().len().to_string()]);

        table.add_row([
            String::from("Memory"),
            format!(
                "{:.2} {units} / {:.2} {units}",
                convert_data_size_unit(sys.used_memory(), units),
                convert_data_size_unit(sys.total_memory(), units)
            ),
        ]);

        table.add_row([
            String::from("Swap"),
            format!(
                "{:.2} {units} / {:.2} {units}",
                convert_data_size_unit(sys.used_swap(), units),
                convert_data_size_unit(sys.total_swap(), units)
            ),
        ]);

        table
    }
}
