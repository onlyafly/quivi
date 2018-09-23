mod ast;
mod back;
mod front;
mod loc;

#[allow(unused_imports)]
use back::env::{Env, SmartEnv};
use loc::Loc;

pub fn interpret(filename: &str, input: &str) -> String {
    let parse_result = front::parse(filename, input);

    match parse_result {
        Ok(nodes) => {
            let mut env: SmartEnv;
            let root_env_result = back::create_root_env();
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

            let eval_result = back::eval(&env, nodes);
            match eval_result {
                Ok(output_value) => output_value.display(),
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
            for (loc, syntax_error) in syntax_errors {
                let s = match loc {
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
