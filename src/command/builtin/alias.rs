use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::exit::ExitCode;
use crate::engine::signature::Signature;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::parser::argument::ParsedArguments;
use crate::parser::span::Spanned;
use crate::shell::aliases::Aliases;
use crate::shell::shell_state::ShellState;
use std::io::Write;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
enum Format {
    List,
    Table,
}

struct NamedSpec<T> {
    name: &'static str,
    description: &'static str,
    short: Option<char>,
    mode: T,
}

const FORMAT_SPECS: &[NamedSpec<Format>] = &[
    NamedSpec {
        name: "list",
        description: "Print all variables with list format",
        short: Some('l'),
        mode: Format::List,
    },
    NamedSpec {
        name: "table",
        description: "Print all variables with table format",
        short: Some('t'),
        mode: Format::Table,
    },
];

pub struct Alias;

impl Command for Alias {
    fn name(&self) -> &'static str {
        "alias"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        let mut signature = Signature::new(self.name());

        for spec in FORMAT_SPECS {
            signature = signature.switch(spec.name, spec.description, spec.short);
        }

        signature
    }

    fn run(
        &self,
        _cmd: Spanned<String>,
        args: ParsedArguments,
        _job_id: Option<usize>,
        _config: Arc<Config>,
        shell_state: Arc<RwLock<ShellState>>,
        mut io_streams: IoStreams,
    ) -> Result<CommandData, ShellError> {
        let mut final_exit_code = ExitCode::SUCCESS;
        let mut format = Format::Table;

        if args.is_empty() {
            let table = shell_state.read().unwrap().aliases.to_table();
            writeln!(io_streams.output, "{table}")?;
            return Ok(CommandData::ExitCode(final_exit_code));
        }

        for spec in FORMAT_SPECS {
            if args.has_switch(spec.name) {
                format = spec.mode.clone();
            }
        }

        match format {
            Format::List => {
                let aliases = shell_state.read().unwrap().aliases.to_list_string();
                writeln!(io_streams.output, "{}", aliases.trim_end())?;
            }
            Format::Table => {
                let table = shell_state.read().unwrap().aliases.to_table();
                writeln!(io_streams.output, "{table}")?;
            }
        }

        // while let Some(arg) = args_iter.next() {
        //     match arg.item.as_str() {
        //         "-p" => {
        //             if let Some(alias_key_arg) = args_iter.next() {
        //                 match shell_state
        //                     .write()
        //                     .unwrap()
        //                     .aliases
        //                     .get(&alias_key_arg.item)
        //                 {
        //                     Some(value) => {
        //                         writeln!(
        //                             io_streams.output,
        //                             "{}",
        //                             Aliases::format_item_string(&alias_key_arg.item, value)
        //                         )?;
        //                     }
        //                     None => {
        //                         writeln!(
        //                             io_streams.error,
        //                             "alias: no alias named {} found",
        //                             alias_key_arg.item
        //                         )?;
        //                     }
        //                 }
        //             } else {
        //                 writeln!(io_streams.error, "alias: missing alias name after -p")?;
        //                 final_exit_code = ExitCode::SYNTAX_ERROR;
        //             }
        //         }
        //
        //         "-r" => {
        //             if let Some(alias_key_arg) = args_iter.next() {
        //                 match shell_state
        //                     .write()
        //                     .unwrap()
        //                     .aliases
        //                     .remove(&alias_key_arg.item)
        //                 {
        //                     Some(value) => {
        //                         writeln!(
        //                             io_streams.output,
        //                             "alias: removed: {}",
        //                             Aliases::format_item_string(&alias_key_arg.item, &value)
        //                         )?;
        //                     }
        //                     None => {
        //                         writeln!(
        //                             io_streams.error,
        //                             "alias: no alias named \"{}\" to remove",
        //                             alias_key_arg.item
        //                         )?;
        //                         final_exit_code = ExitCode::FAILURE;
        //                     }
        //                 }
        //             } else {
        //                 writeln!(io_streams.error, "alias: missing alias name after -r")?;
        //                 final_exit_code = ExitCode::SYNTAX_ERROR;
        //             }
        //         }
        //
        //         alias_name_arg => {
        //             let eq_arg = args_iter.next().map(|s| s.item.as_str());
        //             if eq_arg == Some("=") {
        //                 let mut aliased_args = vec![];
        //                 for arg in args_iter.by_ref() {
        //                     aliased_args.push(arg.clone());
        //                 }
        //
        //                 if !aliased_args.is_empty() {
        //                     shell_state.write().unwrap().aliases.insert(
        //                         alias_name_arg.to_owned(),
        //                         aliased_args
        //                             .iter()
        //                             .map(|s| s.item.as_str())
        //                             .collect::<Vec<&str>>()
        //                             .join(" "),
        //                     );
        //                 }
        //             } else {
        //                 writeln!(
        //                     io_streams.error,
        //                     "alias: missing = between alias name and aliased value. Example: alias ll = ls -la"
        //                 )?;
        //                 final_exit_code = ExitCode::SYNTAX_ERROR;
        //             }
        //         }
        //     }
        // }

        Ok(CommandData::ExitCode(final_exit_code))
    }
}
