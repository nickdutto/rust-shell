use crate::config::Config;
use crate::engine::command::{Command, CommandData, CommandType};
use crate::engine::exit::ExitCode;
use crate::engine::signature::Signature;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::parser::argument::ParsedArguments;
use crate::parser::shape::SyntaxShape;
use crate::parser::span::Spanned;
use crate::shell::shell_state::ShellState;
use std::io::Write;
use std::sync::{Arc, RwLock};

pub struct Complete;

impl Command for Complete {
    fn name(&self) -> &'static str {
        "complete"
    }

    fn command_type(&self) -> CommandType {
        CommandType::Builtin
    }

    fn signature(&self) -> Signature {
        Signature::new(self.name())
            // TODO: new logic required for requiring name and path
            .named("add", SyntaxShape::String, "Add completion path", Some('a'))
            .named(
                "remove",
                SyntaxShape::String,
                "Remove completion",
                Some('r'),
            )
            // TODO better name
            .named(
                "print",
                SyntaxShape::String,
                "find and print completion",
                Some('p'),
            )
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

        let print = args.opt_named::<String>("print")?;

        if let Some(p) = print {
            match shell_state.read().unwrap().completions.get_key_value(&p) {
                Ok((name, path)) => {
                    writeln!(io_streams.output, "complete -C '{path}' {name}")?;
                }
                Err(e) => {
                    writeln!(io_streams.error, "{e}")?;
                    final_exit_code = ExitCode::FAILURE;
                }
            }
        } else if print.unwrap_or(String::new()).is_empty() {
            writeln!(
                io_streams.error,
                "complete: missing specification name for -p",
            )?;
            final_exit_code = ExitCode::SYNTAX_ERROR;
        }

        if let Some(remove_name) = args.opt_named::<String>("remove")? {
            shell_state
                .write()
                .unwrap()
                .completions
                .remove(&remove_name);
        }

        // TODO
        // while let Some(arg) = args_iter.next() {
        //     match arg.item.as_str() {
        //         "-C" => {
        //             if let Some(path_arg) = args_iter.next()
        //                 && let Some(name_arg) = args_iter.next()
        //             {
        //                 shell_state
        //                     .write()
        //                     .unwrap()
        //                     .completions
        //                     .insert(name_arg.item.clone(), path_arg.item.clone());
        //             }
        //         }
        //         _ => (),
        //     }
        // }

        Ok(CommandData::ExitCode(final_exit_code))
    }
}
