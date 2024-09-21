use std::{cmp, fs};
use std::path::PathBuf;

use clap::{Parser as _, Subcommand};
use dfrs_core::config::Config;
use dfrs_core::send::send;
use dfrs_core::token::Position;
use dfrs_core::compile::compile;
use dfrs_core::lexer::{Lexer, LexerError};
use dfrs_core::load_config;
use dfrs_core::parser::{ParseError, Parser};
use dfrs_core::validate::{Validator, ValidateError};
use lsp::run_lsp;

use colored::Colorize;
use dfrs_core::decompile::Decompiler;

mod lsp;

fn print_err(message: String, data: String, start_pos: Position, end_pos: Option<Position>) {
    let lines = data.split("\n").collect::<Vec<&str>>();
    let line = lines.get((start_pos.line - 1) as usize).unwrap();
    let ln = start_pos.line;
    let ln_length = ln.to_string().chars().count();

    println!("{} {}", "Error:".bright_red(), message);
    println!("{} {}", " ".repeat(ln_length), "|".bright_black());
    println!("{} {} {}", ln.to_string().bright_black(), "|".bright_black(), line);
    let arrows;
    match end_pos {
        Some(end_pos) => {
            if end_pos.line != start_pos.line {
                // TODO
                return;
            }
            arrows = "^".repeat(cmp::max(end_pos.col - start_pos.col, 1) as usize).bright_blue();
        }
        None => {
            arrows = "^".bright_blue();
        }
    }
    println!("{} {} {}{}", " ".repeat(ln_length), "|".bright_black(), " ".repeat((start_pos.col - 1) as usize), arrows);
}

fn compile_cmd(file: &PathBuf) {
    println!("{} {}", "Compiling".bright_black(), file.file_name().unwrap().to_string_lossy());
    let mut config_file = file.clone();
    config_file.set_file_name("dfrs.toml");
    let config = match load_config(&config_file) {
        Ok(res) => res,
        Err(_) => {
            println!("{} No config file found", "Error:".bright_red());
            println!("{} dfrs init <path> {}", "Use".bright_black(), "to create a new config file".bright_black());
            return;
        }
    };

    let data = std::fs::read_to_string(file).expect("could not open file");

    let mut lexer = Lexer::new(data.clone());
    let result = lexer.run();

    let res = match result {
        Ok(res) => {
            if config.debug.tokens {
                for token in &res {
                    println!("{:?}", token);
                }
                println!("\n");
            }
            res
        }
        Err(err) => {
            match err {
                LexerError::InvalidNumber { pos } => {
                    print_err(format!("Invalid number in line {pos}"), data, pos, None);
                }
                LexerError::InvalidToken { token, pos } => {
                    print_err(format!("Invalid token '{token}' in line {pos}"), data, pos, None);
                }
                LexerError::UnterminatedString { pos } => {
                    print_err(format!("Unterminated string in line {pos}"), data, pos, None);
                }
                LexerError::UnterminatedText { pos } => {
                    print_err(format!("Unterminated text in line {pos}"), data, pos, None);
                }
                LexerError::UnterminatedVariable { pos } => {
                    print_err(format!("Unterminated variable in line {pos}"), data, pos, None);
                }
            }
            std::process::exit(0);
        }
    };

    let mut parser = Parser::new(res);
    let res = parser.run();
    let node;
    match res {
        Ok(res) => {
            if config.debug.nodes {
                for event in &res.events {
                    println!("{}", event.event);
                    for expression in &event.expressions {
                        match &expression.node {
                            dfrs_core::node::Expression::Action { node } => {
                                println!("{:?} {:?} {:?} {:?}", node.action_type, node.selector, node.name, node.args)
                            } 
                            dfrs_core::node::Expression::Conditional { node } => {
                                println!("{:?} {:?} {:?} {:?}", node.conditional_type, node.selector, node.name, node.args)
                            },
                            dfrs_core::node::Expression::Call { node } => {
                                println!("{:?} {:?}", node.name, node.args)
                            }
                            dfrs_core::node::Expression::Repeat { node } => {
                                println!("{:?} {:?}", node.name, node.args)
                            },
                            dfrs_core::node::Expression::Variable { node } => {
                                println!("{:?} {:?} {:?}", node.var_type, node.dfrs_name, node.df_name)
                            },
                            
                        }
                    }
                }
                println!("\n");
                for function in &res.functions {
                    println!("{}", function.name);
                    for param in &function.params {
                        println!("{:?}", param);
                    }
                    for expression in &function.expressions {
                        match &expression.node {
                            dfrs_core::node::Expression::Action { node } => {
                                println!("{:?} {:?} {:?} {:?}", node.action_type, node.selector, node.name, node.args)
                            }
                            dfrs_core::node::Expression::Conditional { node } => {
                                println!("{:?} {:?} {:?} {:?}", node.conditional_type, node.selector, node.name, node.args)
                            }
                            dfrs_core::node::Expression::Call { node } => {
                                println!("{:?} {:?}", node.name, node.args)
                            }
                            dfrs_core::node::Expression::Repeat { node } => {
                                println!("{:?} {:?}", node.name, node.args)
                            },
                            dfrs_core::node::Expression::Variable { node } => {
                                println!("{:?} {:?} {:?}", node.var_type, node.dfrs_name, node.df_name)
                            },
                            
                        }
                    }
                }
                println!("\n");
            }
            node = res;
        }
        Err(err) => {
            match err {
                ParseError::InvalidToken { found,expected} => {
                    if found.is_some() {
                        let found = found.unwrap();

                        let mut i = 0;
                        let mut expected_string = "".to_owned();
                        for token in expected.clone() {
                            expected_string.push_str(&format!("'{token}'"));
                            if i < expected.len() - 1 {
                                expected_string.push_str(", ");
                            }
                            i += 1;
                        }

                        print_err(format!("Invalid token '{}', expected: {expected_string}", found.token), data, found.start_pos, Some(found.end_pos));
                    } else {
                        println!("Invalid EOF, expected: {expected:?}");
                    }
                }
                ParseError::InvalidCall { pos, msg } => {
                    print_err(format!("Invalid function call: {}", msg), data, pos, None)
                }
                ParseError::InvalidLocation { pos, msg } => {
                    print_err(format!("Invalid Location: {}", msg), data, pos, None)
                }
                ParseError::InvalidVector { pos, msg } => {
                    print_err(format!("Invalid Vector: {}", msg), data, pos, None)
                }
                ParseError::InvalidSound { pos, msg } => {
                    print_err(format!("Invalid Sound: {}", msg), data, pos, None)
                }
                ParseError::InvalidPotion { pos, msg } => {
                    print_err(format!("Invalid Potion: {}", msg), data, pos, None)
                }
                ParseError::UnknownVariable { found, start_pos, end_pos } => {
                    print_err(format!("Unknown variable: {}", found), data, start_pos, Some(end_pos))
                }
                ParseError::InvalidType { found, start_pos } => {
                    match found {
                        Some(found) => print_err(format!("Unknown type: {}", found.token), data, found.start_pos, Some(found.end_pos)),
                        None => print_err("Missing type".into(), data, start_pos, None)
                    }
                },
            }
            std::process::exit(0);
        }
    }

    let validated;
    match Validator::new().validate(node) {
        Ok(res) => validated = res,
        Err(err)  => {
            match err {
                ValidateError::UnknownEvent { node } => {
                    print_err(format!("Unknown event '{}'", node.event), data, node.start_pos, Some(node.name_end_pos));
                }
                ValidateError::UnknownAction { name, start_pos, end_pos } => {
                    print_err(format!("Unknown action '{}'", name), data, start_pos, Some(end_pos));
                }
                ValidateError::MissingArgument { name, start_pos, end_pos } => {
                    print_err(format!("Missing argument '{}'", name), data, start_pos, Some(end_pos));
                }
                ValidateError::WrongArgumentType { args, index, name, expected_types, found_type } => {
                    print_err(format!("Wrong argument type for '{}', expected '{:?}' but found '{:?}'", name, expected_types, found_type), data, args.get(index as usize).unwrap().start_pos.clone(), Some(args.get(index as usize).unwrap().end_pos.clone()));
                }
                ValidateError::TooManyArguments { start_pos, end_pos, name } => {
                    print_err(format!("Too many arguments for action '{}'", name), data, start_pos, Some(end_pos));
                }
                ValidateError::InvalidTagOption { tag_name, provided, options, start_pos, end_pos } => {
                    print_err(format!("Invalid option '{}' for tag '{}', expected one of {:?}", provided, tag_name, options), data, start_pos, Some(end_pos));
                }
                ValidateError::UnknownTag { tag_name, available, start_pos, end_pos } => {
                    print_err(format!("Unknown tag '{}', found tags: {:?}", tag_name, available), data, start_pos, Some(end_pos));
                }
                ValidateError::UnknownGameValue { game_value, start_pos, end_pos} => {
                    print_err(format!("Unknown game_value '{game_value}'"), data, start_pos, Some(end_pos));
                }
            }
            std::process::exit(0);
        }
    }

    let compiled = compile(validated, config.debug.compile);
    println!("{}  {}", "Compiled".green(), file.file_name().unwrap().to_string_lossy());
    send(compiled, config);
}

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Compile {
        path: PathBuf,
    },
    Init {
        path: PathBuf,
    },
    Decompile {
        code: String
    },
    LSP {}
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Compile { path }) => {
            if !path.exists() {
                println!("{} File not found", "Error:".bright_red());
                return;
            }
            if path.is_dir() {
                let paths = fs::read_dir(path).unwrap();

                println!("{} {}", "Compiling project".bright_black(), path.file_name().unwrap().to_string_lossy());
                for path in paths {
                    let file = path.unwrap().path();
                    if file.is_file() && file.extension().unwrap() == "dfrs" {
                        compile_cmd(&file);
                    }
                }
            } else {
                println!("f");
                compile_cmd(path);
            }
        }
        Some(Commands::Init { path }) => {
            if !path.exists() {
                println!("{} File not found", "Error:".bright_red());
                return;
            }
            if !path.is_dir() {
                println!("{} Path is not a directory", "Error:".bright_red());
                return;
            }
            println!("{} {}", "Initializing new project in".bright_black(), path.to_string_lossy());
            let new_config = Config::default();
            let mut config_path = path.clone();
            config_path.push("dfrs.toml");
            new_config.save(&config_path);
            println!("{} {}", "Created new config".green(), config_path.to_string_lossy());
        }
        Some(Commands::Decompile { code }) => {
            let mut decompiler = Decompiler::new();
            decompiler.decompile(code);
        }
        Some(Commands::LSP {}) => {
            run_lsp();
        }
        None => {}
    }
}