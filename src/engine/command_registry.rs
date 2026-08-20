use crate::command::core::echo::Echo;
use crate::command::core::external::External;
use crate::command::date::now::Now;
use crate::command::date::timezone::Timezone;
use crate::command::debug::ast::Ast;
use crate::command::debug::lex::Lex;
use crate::command::filesystem::cd::Cd;
use crate::command::filesystem::pwd::Pwd;
use crate::command::help::explain::Explain;
use crate::command::help::help_commands::HelpCommands;
use crate::command::help::type_cmd::TypeCmd;
use crate::command::network::http::Http;
use crate::command::network::http_get::HttpGet;
use crate::command::process::jobs::Jobs;
use crate::command::shell::alias::Alias;
use crate::command::shell::complete::Complete;
use crate::command::shell::declare::Declare;
use crate::command::shell::exit::Exit;
use crate::command::system::sys::Sys;
use crate::command::system::sys_disk::SysDisk;
use crate::command::system::sys_network::SysNetwork;
use crate::command::system::sys_os::SysOs;
use crate::command::system::sys_perf::SysPerf;
use crate::command::system::sys_process::SysProcess;
use crate::command::ui::theme::Theme;
use crate::engine::command::Command;
use crate::engine::command::signature::Signature;
use crate::parser::span::{Span, Spanned};
use std::collections::HashMap;
use std::sync::Arc;

pub struct CommandRegistry {
    pub commands: HashMap<String, Arc<dyn Command + Send + Sync>>,
    executable_command: Arc<dyn Command + Send + Sync>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            executable_command: Arc::new(External),
        }
    }

    pub fn register(&mut self, command: Arc<dyn Command + Send + Sync>) {
        self.commands.insert(command.name().to_string(), command);
    }

    pub fn get(&self, cmd_name: &str) -> Option<Arc<dyn Command + Send + Sync>> {
        self.commands.get(cmd_name).cloned()
    }

    pub fn resolve(
        &self,
        cmd_name: Spanned<String>,
        mut args: Vec<Spanned<String>>,
    ) -> (
        Arc<dyn Command + Send + Sync>,
        Spanned<String>,
        Vec<Spanned<String>>,
    ) {
        if !args.is_empty() {
            let sub_cmd_name = format!("{} {}", cmd_name.item, args[0].item);

            if let Some(command) = self.commands.get(&sub_cmd_name) {
                let first_arg = args.remove(0);
                return (
                    Arc::clone(command),
                    Spanned::new(
                        sub_cmd_name,
                        Span::new(cmd_name.span.start, first_arg.span.end),
                    ),
                    args,
                );
            }
        }

        if let Some(command) = self.commands.get(&cmd_name.item) {
            return (Arc::clone(command), cmd_name, args);
        }

        (Arc::clone(&self.executable_command), cmd_name, args)
    }

    pub fn is_builtin(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }

    pub fn all_signatures(&self) -> Vec<Signature> {
        self.commands.values().map(|v| v.signature()).collect()
    }

    pub fn register_builtins(mut self) -> Self {
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

        self.register(Arc::new(Now));
        self.register(Arc::new(Timezone));

        self.register(Arc::new(Ast));
        self.register(Arc::new(Lex));

        self.register(Arc::new(Explain));
        self.register(Arc::new(HelpCommands));
        self.register(Arc::new(TypeCmd));

        self.register(Arc::new(Http));
        self.register(Arc::new(HttpGet));

        self.register(Arc::new(Sys));
        self.register(Arc::new(SysDisk));
        self.register(Arc::new(SysNetwork));
        self.register(Arc::new(SysOs));
        self.register(Arc::new(SysPerf));
        self.register(Arc::new(SysProcess));

        self
    }
}
