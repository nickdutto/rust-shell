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
use crate::engine::command::Command;
use crate::parser::span::{Span, Spanned};
use std::collections::HashMap;
use std::sync::Arc;

pub struct CommandRegistry {
    commands: HashMap<String, Arc<dyn Command + Send + Sync>>,
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
            executable_command: Arc::new(Executable),
        }
    }

    pub fn register(&mut self, command: Arc<dyn Command + Send + Sync>) {
        self.commands.insert(command.name().to_string(), command);
    }

    pub fn get(
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
        self.register(Arc::new(TypeCmd));

        self.register(Arc::new(Now));
        self.register(Arc::new(Timezone));

        self.register(Arc::new(Ast));
        self.register(Arc::new(Lex));

        self.register(Arc::new(Explain));

        self.register(Arc::new(Http));

        self
    }
}
