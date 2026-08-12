use crate::command::builtin::alias::Alias;
use crate::command::builtin::cd::Cd;
use crate::command::builtin::complete::Complete;
use crate::command::builtin::declare::Declare;
use crate::command::builtin::echo::Echo;
use crate::command::builtin::executable::Executable;
use crate::command::builtin::exit::Exit;
use crate::command::builtin::jobs::Jobs;
use crate::command::builtin::pwd::Pwd;
use crate::command::builtin::theme::Theme;
use crate::command::builtin::type_cmd::TypeCmd;
use crate::command::date::now::Now;
use crate::command::date::timezone::Timezone;
use crate::command::debug::ast::Ast;
use crate::command::debug::lex::Lex;
use crate::command::help::explain::Explain;
use crate::command::network::http::Http;
use crate::config::Config;
use crate::engine::command::Command;
use crate::engine::process::ProcessHandle;
use crate::error::shell_error::ShellError;
use crate::io::stream::IoStreams;
use crate::parser::span::Spanned;
use crate::shell::shell_state::ShellState;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct CommandRouter {
    commands: HashMap<String, Arc<dyn Command + Send + Sync>>,
    executable_command: Arc<dyn Command + Send + Sync>,
}

impl Default for CommandRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRouter {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            executable_command: Arc::new(Executable),
        }
    }

    pub fn register(&mut self, command: Arc<dyn Command + Send + Sync>) {
        self.commands.insert(command.name().to_string(), command);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Command + Send + Sync>> {
        self.commands.get(name).cloned()
    }

    pub fn is_builtin(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }

    pub fn register_builtins(&mut self) {
        self.register(Arc::new(Alias));
        self.register(Arc::new(Cd));
        self.register(Arc::new(Complete));
        self.register(Arc::new(Declare));
        self.register(Arc::new(Echo));
        self.register(Arc::new(Exit));
        // self.register(Arc::new(History));
        self.register(Arc::new(Jobs));
        self.register(Arc::new(Pwd));
        self.register(Arc::new(Theme));
        self.register(Arc::new(TypeCmd));

        self.register(Arc::new(Now));
        self.register(Arc::new(Timezone));

        self.register(Arc::new(Ast));
        self.register(Arc::new(Lex));

        self.register(Arc::new(Explain));

        self.register(Arc::new(Http));
    }

    pub fn dispatch(
        &self,
        cmd: Spanned<String>,
        args: Vec<Spanned<String>>,
        needs_thread: bool,
        current_job_id: Option<usize>,
        config: Arc<Config>,
        shell_state: Arc<RwLock<ShellState>>,
        io_streams: IoStreams,
    ) -> Result<ProcessHandle, ShellError> {
        let command = self
            .get(&cmd.item)
            .unwrap_or_else(|| self.executable_command.clone());

        let parsed_args = command.signature().parse(args)?;

        let handle = ProcessHandle::run_producer(
            Box::new(move || {
                command.run(
                    cmd,
                    parsed_args,
                    current_job_id,
                    config,
                    shell_state,
                    io_streams,
                )
            }),
            needs_thread,
        );

        Ok(handle)
    }
}
