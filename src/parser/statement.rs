use crate::parser::command_node::CommandNode;

#[derive(Debug, PartialEq)]
pub enum Statement {
    Command(CommandNode),
    Background(Box<Statement>),
    And {
        left: Box<Statement>,
        right: Box<Statement>,
    },
    Pipeline(Vec<CommandNode>),
    Sequential {
        left: Box<Statement>,
        right: Box<Statement>,
    },
}

impl Statement {
    pub fn to_statement_string(&self) -> String {
        match self {
            Statement::Command(command_node) => command_node.to_command_string(),

            Statement::Pipeline(statements) => statements
                .iter()
                .map(CommandNode::to_command_string)
                .collect::<Vec<_>>()
                .join(" | "),

            Statement::And { left, right } => {
                format!(
                    "{} && {}",
                    left.to_statement_string(),
                    right.to_statement_string()
                )
            }

            Statement::Sequential { left, right } => {
                format!(
                    "{} ; {}",
                    left.to_statement_string(),
                    right.to_statement_string()
                )
            }

            Statement::Background(inner) => {
                format!("{} &", inner.to_statement_string())
            }
        }
    }
}
