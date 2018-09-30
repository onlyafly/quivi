mod ast;
mod back;
mod front;
mod loc;

use ast::WriterObj;
use back::env::SmartEnv;
use loc::Loc;
use std::io;

fn get_stdout() -> Box<io::Write> {
    Box::new(io::stdout())
}

pub fn interpret(filename: &str, input: &str) -> String {
    let parse_result = front::parse(filename, input);

    let writer_obj = WriterObj {
        name: "stdout".to_string(),
        get_writer: get_stdout,
    };

    match parse_result {
        Ok(nodes) => {
            let mut env: SmartEnv;
            let root_env_result = back::create_root_env(writer_obj);
            match root_env_result {
                Ok(root_env) => env = root_env,
                Err(runtime_error) => {
                    return format!(
                        "Error creating root environment: {}",
                        runtime_error.display()
                    )
                }
            }

            /* DEBUG
            for Value in &Values {
                println!("{}", Value.display())
            }
            */

            let eval_result = back::eval(env, nodes);
            match eval_result {
                Ok(output_node) => format!("{}", output_node.val),
                Err(runtime_error) => match runtime_error.loc() {
                    Loc::File { filename, line, .. } => format!(
                        "Runtime error ({}:{}): {}\n\n",
                        filename,
                        line,
                        runtime_error.display()
                    ),
                    Loc::Unknown => format!("Runtime error: {}\n\n", runtime_error.display()),
                },
            }
        }
        Err(syntax_errors) => {
            let mut output = String::new();
            for syntax_error in syntax_errors {
                let s = match syntax_error.loc() {
                    Loc::File { filename, line, .. } => format!(
                        "Syntax error ({}:{}): {}\n\n",
                        filename,
                        line,
                        syntax_error.display()
                    ),
                    Loc::Unknown => format!("Syntax error: {}\n\n", syntax_error.display(),),
                };
                output.push_str(&s);
            }
            output
        }
    }
}
