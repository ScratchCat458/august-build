use std::iter::Peekable;
use std::slice::Iter;

use crate::{Command, CommandDefinition, InternalCommand, Module, Task};

use super::error::{ParserError, ParserScope};
use super::token::{EncapsulatorType, Node, Token};

const NULL_SPAN: (usize, usize) = (0, 0);

fn err_unwrap<T>(option: Option<T>, scope: ParserScope) -> Result<T, ParserError> {
    match option {
        Some(t) => Ok(t),
        None => Err(ParserError::OutOfTokens { scope }),
    }
}

pub fn parse_module(tokens: Vec<Token>) -> Result<Module, ParserError> {
    let mut cursor = tokens.iter().peekable();
    let mut module = Module::default();

    while let Some(t) = cursor.peek() {
        match t {
            Token::Ident(i, _) if i == &"Task".to_string() => {
                let (task_name, task) = parse_task(&mut cursor)?;
                module.task(task_name, task);
            }
            Token::Ident(i, _) if i == &"cmddef".to_string() => {
                let (name, definition) = parse_cmd_def(&mut cursor)?;
                module.cmd_def(name, definition);
            }
            Token::Punct('#', _) => {
                // for both link and pragma
                parse_pragma(&mut cursor, &mut module)?;
            }
            Token::Punct('@', _) => {
                let namespace = parse_namespace(&mut cursor)?;
                module.namespace(namespace);
            }
            _ => {
                cursor.next();
            }
        }
    }

    Ok(module)
}

pub fn parse_task(cursor: &mut Peekable<Iter<Token>>) -> Result<(String, Task), ParserError> {
    #![allow(unused_assignments)]
    let mut name = String::new();
    let mut task = Task::default();

    if err_unwrap(cursor.peek(), ParserScope::Task)?
        != &&Token::Ident("Task".to_string(), NULL_SPAN)
    {
        return Err(ParserError::TokenMismatch {
            scope: ParserScope::Task,
            token: cursor.next().unwrap().clone(),
            expected_token: Token::Ident("Task".to_string(), NULL_SPAN),
        });
    }
    cursor.next();

    if let Token::Ident(i, _) = err_unwrap(cursor.peek(), ParserScope::Task)? {
        name = i.to_string();
    } else {
        return Err(ParserError::TokenMismatch {
            scope: ParserScope::Task,
            token: cursor.next().unwrap().clone(),
            expected_token: Token::Ident("task_name".to_string(), NULL_SPAN),
        });
    }
    cursor.next();

    if err_unwrap(cursor.peek(), ParserScope::Task)? == &&Token::Punct(':', NULL_SPAN) {
        cursor.next();
        if let Token::Node(node, s) = err_unwrap(cursor.peek(), ParserScope::Task)? {
            if node.encapsulator != EncapsulatorType::Square {
                return Err(ParserError::EncapsulatorMismatch {
                    scope: ParserScope::Task,
                    encap: node.encapsulator.clone(),
                    expected_encap: EncapsulatorType::Square,
                    node_span: s.to_owned(),
                });
            }

            let chunks = node.children.split(|x| x == &Token::Punct(',', NULL_SPAN));
            for chunk in chunks {
                if chunk.is_empty() {
                    continue;
                }

                if chunk.len() != 1 {
                    let chunk_span = (chunk[0].span_start(), chunk[chunk.len() - 1].span_end());
                    return Err(ParserError::BadChunkLength {
                        scope: ParserScope::Task,
                        len: chunk.len(),
                        valid_len: vec![1, 0],
                        chunk_span,
                    });
                }

                if let Token::Ident(i, _) = chunk[0].clone() {
                    task.dependency(i);
                } else {
                    return Err(ParserError::TokenMismatch {
                        scope: ParserScope::Task,
                        token: chunk[0].clone(),
                        expected_token: Token::Ident("dependency_name".to_string(), NULL_SPAN),
                    });
                }
            }
            cursor.next();
        } else {
            return Err(ParserError::TokenMismatch {
                scope: ParserScope::Task,
                token: cursor.next().unwrap().clone(),
                expected_token: Token::Node(Node::new(EncapsulatorType::Square), NULL_SPAN),
            });
        }
    }

    if let Token::Node(node, s) = err_unwrap(cursor.peek(), ParserScope::Task)? {
        if node.encapsulator != EncapsulatorType::Curly {
            return Err(ParserError::EncapsulatorMismatch {
                scope: ParserScope::Task,
                encap: node.encapsulator.clone(),
                expected_encap: EncapsulatorType::Curly,
                node_span: s.to_owned(),
            });
        }

        let commands = parse_body_cmds(node.children.clone(), ParserScope::TaskBody)?;

        for i in commands {
            task.command(i);
        }
    } else {
        return Err(ParserError::TokenMismatch {
            scope: ParserScope::Task,
            token: cursor.next().unwrap().clone(),
            expected_token: Token::Node(Node::new(EncapsulatorType::Curly), NULL_SPAN),
        });
    }
    Ok((name, task))
}

pub fn parse_cmd_def(
    cursor: &mut Peekable<Iter<Token>>,
) -> Result<(Command, CommandDefinition), ParserError> {
    #![allow(unused_assignments)]
    let mut name = String::new();
    let mut cmd_def = CommandDefinition::default();

    if err_unwrap(cursor.peek(), ParserScope::CommandDefinition)?
        != &&Token::Ident("cmddef".to_string(), NULL_SPAN)
    {
        return Err(ParserError::TokenMismatch {
            scope: ParserScope::CommandDefinition,
            token: cursor.next().unwrap().clone(),
            expected_token: Token::Ident("cmddef".to_string(), NULL_SPAN),
        });
    }
    cursor.next();

    if let Token::Ident(i, _) = err_unwrap(cursor.peek(), ParserScope::CommandDefinition)? {
        name = i.to_string();
    } else {
        return Err(ParserError::TokenMismatch {
            scope: ParserScope::CommandDefinition,
            token: cursor.next().unwrap().clone(),
            expected_token: Token::Ident("cmddef_name".to_string(), NULL_SPAN),
        });
    }
    cursor.next();

    if let Token::Node(node, s) = err_unwrap(cursor.peek(), ParserScope::CommandDefinition)? {
        if node.encapsulator != EncapsulatorType::Curly {
            return Err(ParserError::EncapsulatorMismatch {
                scope: ParserScope::CommandDefinition,
                encap: node.encapsulator.clone(),
                expected_encap: EncapsulatorType::Curly,
                node_span: s.to_owned(),
            });
        }

        let commands = parse_body_cmds(node.children.clone(), ParserScope::CommandDefinitionBody)?;

        for i in commands {
            cmd_def.command(i);
        }
    } else {
        return Err(ParserError::TokenMismatch {
            scope: ParserScope::CommandDefinition,
            token: cursor.next().unwrap().clone(),
            expected_token: Token::Node(Node::new(EncapsulatorType::Curly), NULL_SPAN),
        });
    }

    Ok((Command::Local(name), cmd_def))
}

pub fn parse_body_cmds(
    tokens: Vec<Token>,
    scope: ParserScope,
) -> Result<Vec<Command>, ParserError> {
    let chunks = tokens.split(|x| x == &Token::Punct(';', NULL_SPAN));
    let mut commands = Vec::new();

    for chunk in chunks {
        if chunk.is_empty() {
            continue;
        }

        let chunk_span = (chunk[0].span_start(), chunk[chunk.len() - 1].span_end());

        match chunk.len() {
            3 => {
                // External
                let Token::Ident(cmd_namespace, _) = chunk[0].clone() else {
                        return Err(ParserError::TokenMismatch {
                            scope,
                            token: chunk[0].clone(),
                            expected_token: Token::Ident(
                                "command_namespace".to_string(),
                                NULL_SPAN,
                            ),
                        });
                };

                if chunk[1] != Token::Punct('.', NULL_SPAN) {
                    return Err(ParserError::TokenMismatch {
                        scope,
                        token: chunk[1].clone(),
                        expected_token: Token::Punct('.', NULL_SPAN),
                    });
                }

                let Token::Ident(cmd_name, _) = chunk[2].clone() else {
                    return Err(ParserError::TokenMismatch {
                        scope,
                        token: chunk[2].clone(),
                        expected_token: Token::Ident("command_name".to_string(), NULL_SPAN),
                    });
                };

                commands.push(Command::External(cmd_namespace, cmd_name));
            }
            2 => {
                //Internal with arguments
                let Token::Ident(cmd_name, _) = chunk[0].clone() else {
                    return Err(ParserError::TokenMismatch {
                        scope,
                        token: chunk[0].clone(),
                        expected_token: Token::Ident("command_name".to_string(), NULL_SPAN),
                    });
                };

                let (arg_node, arg_span) = match chunk[1].clone() {
                    Token::Node(node, s) => {
                        if node.encapsulator != EncapsulatorType::Round {
                            return Err(ParserError::EncapsulatorMismatch {
                                scope,
                                encap: node.encapsulator,
                                expected_encap: EncapsulatorType::Round,
                                node_span: s.to_owned(),
                            });
                        }

                        (node, s)
                    }
                    _ => {
                        return Err(ParserError::TokenMismatch {
                            scope,
                            token: chunk[1].clone(),
                            expected_token: Token::Node(
                                Node::new(EncapsulatorType::Round),
                                NULL_SPAN,
                            ),
                        });
                    }
                };

                match cmd_name.as_str() {
                    "exec" => {
                        if arg_node.children.len() != 1 {
                            return Err(ParserError::BadChunkLength {
                                scope,
                                len: arg_node.children.len(),
                                valid_len: vec![1],
                                chunk_span: arg_span,
                            });
                        }

                        if let Token::StringLiteral(l, _) = arg_node.children[0].clone() {
                            commands.push(Command::Internal(InternalCommand::Exec(l)));
                        } else {
                            return Err(ParserError::TokenMismatch {
                                scope,
                                token: arg_node.children[0].clone(),
                                expected_token: Token::StringLiteral(
                                    "shell --command".to_string(),
                                    NULL_SPAN,
                                ),
                            });
                        }
                    }
                    "set_env_var" => {
                        if arg_node.children.len() != 3 {
                            return Err(ParserError::BadChunkLength {
                                scope,
                                len: arg_node.children.len(),
                                valid_len: vec![3],
                                chunk_span: arg_span,
                            });
                        }

                        let Token::StringLiteral(var_name, _) = arg_node.children[0].clone() else {
                            return Err(ParserError::TokenMismatch {
                                scope,
                                token: arg_node.children[0].clone(),
                                expected_token: Token::StringLiteral(
                                    "ENVVARNAME".to_string(),
                                    NULL_SPAN,
                                ),
                            });
                        };

                        if arg_node.children[1] != Token::Punct(',', NULL_SPAN) {
                            return Err(ParserError::TokenMismatch {
                                scope,
                                token: arg_node.children[1].clone(),
                                expected_token: Token::Punct(',', NULL_SPAN),
                            });
                        }

                        let Token::StringLiteral(var_contents, _) = arg_node.children[2].clone() else {
                            return Err(ParserError::TokenMismatch {
                                scope,
                                token: arg_node.children[2].clone(),
                                expected_token: Token::StringLiteral(
                                    "envvar_contents".to_string(),
                                    NULL_SPAN,
                                ),
                            });
                        };

                        commands.push(Command::Internal(InternalCommand::SetEnvironmentVar(
                            var_name,
                            var_contents,
                        )));
                    }
                    "print" => {
                        if arg_node.children.len() != 1 {
                            return Err(ParserError::BadChunkLength {
                                scope,
                                len: arg_node.children.len(),
                                valid_len: vec![1],
                                chunk_span: arg_span,
                            });
                        }

                        if let Token::StringLiteral(l, _) = arg_node.children[0].clone() {
                            commands.push(Command::Internal(InternalCommand::PrintString(l)));
                        } else {
                            return Err(ParserError::TokenMismatch {
                                scope,
                                token: arg_node.children[0].clone(),
                                expected_token: Token::StringLiteral(
                                    "some text".to_string(),
                                    NULL_SPAN,
                                ),
                            });
                        }
                    }
                    "print_file" => {
                        if arg_node.children.len() != 1 {
                            return Err(ParserError::BadChunkLength {
                                scope,
                                len: arg_node.children.len(),
                                valid_len: vec![1],
                                chunk_span: arg_span,
                            });
                        }

                        if let Token::StringLiteral(l, _) = arg_node.children[0].clone() {
                            commands.push(Command::Internal(InternalCommand::PrintFile(l)));
                        } else {
                            return Err(ParserError::TokenMismatch {
                                scope,
                                token: arg_node.children[0].clone(),
                                expected_token: Token::StringLiteral(
                                    "file_name".to_string(),
                                    NULL_SPAN,
                                ),
                            });
                        }
                    }
                    "make_dir" => {
                        if arg_node.children.len() != 1 {
                            return Err(ParserError::BadChunkLength {
                                scope,
                                len: arg_node.children.len(),
                                valid_len: vec![1],
                                chunk_span: arg_span,
                            });
                        }

                        if let Token::StringLiteral(l, _) = arg_node.children[0].clone() {
                            commands.push(Command::Internal(InternalCommand::MakeDirectory(l)));
                        } else {
                            return Err(ParserError::TokenMismatch {
                                scope,
                                token: arg_node.children[0].clone(),
                                expected_token: Token::StringLiteral(
                                    "directory".to_string(),
                                    NULL_SPAN,
                                ),
                            });
                        }
                    }
                    "make_empty_file" => {
                        if arg_node.children.len() != 1 {
                            return Err(ParserError::BadChunkLength {
                                scope,
                                len: arg_node.children.len(),
                                valid_len: vec![1],
                                chunk_span: arg_span,
                            });
                        }

                        if let Token::StringLiteral(l, _) = arg_node.children[0].clone() {
                            commands.push(Command::Internal(InternalCommand::MakeFile(l)));
                        } else {
                            return Err(ParserError::TokenMismatch {
                                scope,
                                token: arg_node.children[0].clone(),
                                expected_token: Token::StringLiteral(
                                    "file_name".to_string(),
                                    NULL_SPAN,
                                ),
                            });
                        }
                    }
                    "remove_dir" => {
                        if arg_node.children.len() != 1 {
                            return Err(ParserError::BadChunkLength {
                                scope,
                                len: arg_node.children.len(),
                                valid_len: vec![1],
                                chunk_span: arg_span,
                            });
                        }

                        if let Token::StringLiteral(l, _) = arg_node.children[0].clone() {
                            commands.push(Command::Internal(InternalCommand::RemoveDirectory(l)));
                        } else {
                            return Err(ParserError::TokenMismatch {
                                scope,
                                token: arg_node.children[0].clone(),
                                expected_token: Token::StringLiteral(
                                    "directory".to_string(),
                                    NULL_SPAN,
                                ),
                            });
                        }
                    }
                    "remove_file" => {
                        if arg_node.children.len() != 1 {
                            return Err(ParserError::BadChunkLength {
                                scope,
                                len: arg_node.children.len(),
                                valid_len: vec![1],
                                chunk_span: arg_span,
                            });
                        }

                        if let Token::StringLiteral(l, _) = arg_node.children[0].clone() {
                            commands.push(Command::Internal(InternalCommand::RemoveFile(l)));
                        } else {
                            return Err(ParserError::TokenMismatch {
                                scope,
                                token: arg_node.children[0].clone(),
                                expected_token: Token::StringLiteral(
                                    "file_name".to_string(),
                                    NULL_SPAN,
                                ),
                            });
                        }
                    }
                    "copy_file" => {
                        if arg_node.children.len() != 3 {
                            return Err(ParserError::BadChunkLength {
                                scope,
                                len: arg_node.children.len(),
                                valid_len: vec![3],
                                chunk_span: arg_span,
                            });
                        }
                        let Token::StringLiteral(source_path, _) = arg_node.children[0].clone() else {
                            return Err(ParserError::TokenMismatch {
                                scope,
                                token: arg_node.children[0].clone(),
                                expected_token: Token::StringLiteral(
                                    "source_path".to_string(),
                                    NULL_SPAN,
                                ),
                            });
                        };

                        if arg_node.children[1] != Token::Punct(',', NULL_SPAN) {
                            return Err(ParserError::TokenMismatch {
                                scope,
                                token: arg_node.children[1].clone(),
                                expected_token: Token::Punct(',', NULL_SPAN),
                            });
                        }

                        let Token::StringLiteral(destination_path, _) = arg_node.children[2].clone() else {
                            return Err(ParserError::TokenMismatch {
                                scope,
                                token: arg_node.children[2].clone(),
                                expected_token: Token::StringLiteral(
                                    "destination_path".to_string(),
                                    NULL_SPAN,
                                ),
                            });
                        };

                        commands.push(Command::Internal(InternalCommand::CopyFile(
                            source_path,
                            destination_path,
                        )));
                    }
                    "move_file" => {
                        if arg_node.children.len() != 3 {
                            return Err(ParserError::BadChunkLength {
                                scope,
                                len: arg_node.children.len(),
                                valid_len: vec![3],
                                chunk_span: arg_span,
                            });
                        }

                        let Token::StringLiteral(source_path, _) = arg_node.children[0].clone() else {
                            return Err(ParserError::TokenMismatch {
                                scope,
                                token: arg_node.children[0].clone(),
                                expected_token: Token::StringLiteral(
                                    "source_path".to_string(),
                                    NULL_SPAN,
                                ),
                            });
                        };

                        if arg_node.children[1] != Token::Punct(',', NULL_SPAN) {
                            return Err(ParserError::TokenMismatch {
                                scope,
                                token: arg_node.children[1].clone(),
                                expected_token: Token::Punct(',', NULL_SPAN),
                            });
                        }

                        let Token::StringLiteral(destination_path, _) = arg_node.children[2].clone() else {
                            return Err(ParserError::TokenMismatch {
                                scope,
                                token: arg_node.children[2].clone(),
                                expected_token: Token::StringLiteral(
                                    "destination_path".to_string(),
                                    NULL_SPAN,
                                ),
                            });
                        };

                        commands.push(Command::Internal(InternalCommand::MoveFile(
                            source_path,
                            destination_path,
                        )));
                    }
                    _ => {
                        return Err(ParserError::InvalidBody {
                            scope,
                            token: chunk[0].clone(),
                            valid_body: vec!["exec".to_string()],
                        });
                    }
                }
            }
            1 => {
                // Internal or local cdef
                if let Token::Ident(i, _) = chunk[0].clone() {
                    commands.push(Command::Local(i));
                }
            }
            _ => {
                return Err(ParserError::BadChunkLength {
                    scope,
                    len: chunk.len(),
                    valid_len: vec![1, 2, 3],
                    chunk_span,
                });
            }
        }
    }

    Ok(commands)
}

pub fn parse_pragma(
    cursor: &mut Peekable<Iter<Token>>,
    module: &mut Module,
) -> Result<(), ParserError> {
    if err_unwrap(cursor.peek(), ParserScope::Pragma)? != &&Token::Punct('#', NULL_SPAN) {
        return Err(ParserError::TokenMismatch {
            scope: ParserScope::Pragma,
            token: cursor.next().unwrap().clone(),
            expected_token: Token::Punct('#', NULL_SPAN),
        });
    }
    cursor.next();

    match err_unwrap(cursor.peek(), ParserScope::Pragma)? {
        Token::Ident(i, _) if i == &"pragma".to_string() => {
            cursor.next();

            match err_unwrap(cursor.peek(), ParserScope::Pragma)? {
                Token::Ident(i, _) if i == &"test".to_string() => {
                    cursor.next();
                    if let Token::Ident(i, _) = err_unwrap(cursor.peek(), ParserScope::Pragma)? {
                        module.pragmas.test(i);
                        cursor.next();
                    } else {
                        return Err(ParserError::TokenMismatch {
                            scope: ParserScope::Pragma,
                            token: cursor.next().unwrap().clone(),
                            expected_token: Token::Ident("job_name".to_string(), NULL_SPAN),
                        });
                    }
                }
                Token::Ident(i, _) if i == &"build".to_string() => {
                    cursor.next();
                    if let Token::Ident(i, _) = err_unwrap(cursor.peek(), ParserScope::Pragma)? {
                        module.pragmas.build(i);
                        cursor.next();
                    } else {
                        return Err(ParserError::TokenMismatch {
                            scope: ParserScope::Pragma,
                            token: cursor.next().unwrap().clone(),
                            expected_token: Token::Ident("job_name".to_string(), NULL_SPAN),
                        });
                    }
                }
                Token::Ident(_, _) => {
                    return Err(ParserError::InvalidBody {
                        scope: ParserScope::Pragma,
                        token: cursor.next().unwrap().clone(),
                        valid_body: vec![String::from("test"), String::from("build")],
                    });
                }
                _ => {
                    return Err(ParserError::TokenMismatch {
                        scope: ParserScope::Pragma,
                        token: cursor.next().unwrap().clone(),
                        expected_token: Token::Ident("build".to_string(), NULL_SPAN),
                    });
                }
            }
        }
        Token::Ident(i, _) if i == &"link".to_string() => {
            cursor.next();
            if let Token::Ident(i, _) = err_unwrap(cursor.peek(), ParserScope::Pragma)? {
                module.link(i);
                cursor.next();
            } else {
                return Err(ParserError::TokenMismatch {
                    scope: ParserScope::Pragma,
                    token: cursor.next().unwrap().clone(),
                    expected_token: Token::Ident("module_name".to_string(), NULL_SPAN),
                });
            }
        }
        Token::Ident(_, _) => {
            return Err(ParserError::InvalidBody {
                scope: ParserScope::Pragma,
                token: cursor.next().unwrap().clone(),
                valid_body: vec![String::from("pragma"), String::from("link")],
            });
        }
        _ => {
            return Err(ParserError::TokenMismatch {
                scope: ParserScope::Pragma,
                token: cursor.next().unwrap().clone(),
                expected_token: Token::Ident("pragma".to_string(), NULL_SPAN),
            });
        }
    }

    Ok(())
}

pub fn parse_namespace(cursor: &mut Peekable<Iter<Token>>) -> Result<String, ParserError> {
    if err_unwrap(cursor.peek(), ParserScope::Namespace)? != &&Token::Punct('@', NULL_SPAN) {
        return Err(ParserError::TokenMismatch {
            scope: ParserScope::Namespace,
            token: cursor.next().unwrap().clone(),
            expected_token: Token::Punct('@', NULL_SPAN),
        });
    }
    cursor.next();

    if let Token::Ident(i, _) = err_unwrap(cursor.peek(), ParserScope::Namespace)? {
        Ok(i.clone())
    } else {
        Err(ParserError::TokenMismatch {
            scope: ParserScope::Namespace,
            token: cursor.next().unwrap().clone(),
            expected_token: Token::Ident(String::from("namespace"), NULL_SPAN),
        })
    }
}
