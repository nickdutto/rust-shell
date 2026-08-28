use crate::parser::word::{Word, total_word_span, words_to_string};
use crate::shell::shell_state::ShellState;
use crate::value::span::Spanned;
use std::sync::{Arc, RwLock};

pub fn expand_command_node_values(
    node_cmd: Vec<Spanned<Word>>,
    node_args: Vec<Vec<Spanned<Word>>>,
    shell_state: &Arc<RwLock<ShellState>>,
) -> (Spanned<String>, Vec<Spanned<String>>) {
    let cmd_span = total_word_span(&node_cmd);
    let mut cmd_name = words_to_string(node_cmd, &shell_state.read().unwrap().variables);
    let mut args = vec![];

    if let Some(aliased_value) = shell_state.read().unwrap().aliases.get(&cmd_name) {
        let mut aliased_args = aliased_value
            .split_whitespace()
            .map(String::from)
            .collect::<Vec<String>>();

        if !aliased_args.is_empty() {
            cmd_name = aliased_args.remove(0);
            for aliased_arg in aliased_args {
                args.push(Spanned::new(aliased_arg, cmd_span));
            }
        }
    }

    for arg in node_args {
        let arg_span = total_word_span(&arg);
        let arg_string = words_to_string(arg, &shell_state.read().unwrap().variables);

        args.push(Spanned::new(arg_string, arg_span));
    }

    (Spanned::new(cmd_name, cmd_span), args)
}
