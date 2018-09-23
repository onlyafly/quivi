use ast::{Node, PrimitiveObj, Value};
#[allow(unused_imports)]
use back::env::{Env, SmartEnv};
use back::primitives::eval_primitive;
use back::runtime_error::{check_args, RuntimeError};
use back::specials;
use loc::Loc;

type EvalResult = Result<Node, RuntimeError>;

pub fn eval_node(env: &SmartEnv, node: Node) -> EvalResult {
    let loc = node.loc;
    match node.value {
        Value::List { children } => eval_list(env, children, loc),
        Value::Symbol(name) => match env.borrow_mut().get(&name) {
            Some(node) => Ok(node),
            None => Err(RuntimeError::UndefinedName(name, loc)),
        },
        n @ Value::Number(_) => Ok(Node::new(n, loc)),
        n => Err(RuntimeError::UnableToEvalValue(n, loc)),
    }
}

fn eval_each_node(env: &SmartEnv, nodes: Vec<Node>) -> Result<Vec<Node>, RuntimeError> {
    let mut outputs = Vec::new();
    for node in nodes {
        let output = eval_node(env, node)?;
        outputs.push(output);
    }
    Ok(outputs)
}

fn eval_list(env: &SmartEnv, mut args: Vec<Node>, loc: Loc) -> EvalResult {
    if args.len() == 0 {
        return Err(RuntimeError::CannotEvalEmptyList(loc));
    }

    let head_node = args.remove(0);
    let head_value = head_node.value;
    let loc = head_node.loc;

    match head_value {
        Value::Symbol(ref name) => match name.as_ref() {
            "list" => {
                check_args("list", &loc, &args, 0, -1)?;
                return specials::eval_special_list(env, loc, args);
            }
            "quote" => {
                check_args("quote", &loc, &args, 1, -1)?;
                return specials::eval_special_quote(args);
            }
            "def" => {
                check_args("def", &loc, &args, 2, 2)?;
                return specials::eval_special_def(env, args);
            }
            "fn" => {
                check_args("fn", &loc, &args, 2, 2)?;
                return specials::eval_special_fn(env, args);
            }
            "update!" => {
                check_args("update!", &loc, &args, 2, 2)?;
                return specials::eval_special_update(env, args);
            }
            "update-element!" => {
                check_args("update-element!", &loc, &args, 3, 3)?;
                return specials::eval_special_update_element(env, args);
            }
            "if" => {
                check_args("if", &loc, &args, 3, 3)?;
                return specials::eval_special_if(env, args);
            }
            "let" => {
                check_args("let", &loc, &args, 2, -1)?;
                return specials::eval_special_let(env, args);
            }
            _ => {}
        },
        _ => {}
    }

    let evaled_head = eval_node(env, Node::new(head_value, loc.clone()))?;

    match evaled_head.value {
        Value::Function { .. } => eval_invoke_proc(env, evaled_head, args),
        Value::Primitive(obj) => eval_invoke_primitive(obj, env, args, loc),
        _ => Err(RuntimeError::UnableToEvalListStartingWith(
            evaled_head.display(),
            loc,
        )),
    }
}

fn eval_invoke_primitive(
    obj: PrimitiveObj,
    dynamic_env: &SmartEnv,
    unevaled_args: Vec<Node>,
    loc: Loc,
) -> EvalResult {
    let evaled_args = eval_each_node(dynamic_env, unevaled_args)?;
    eval_primitive(obj, dynamic_env, evaled_args, loc)
}

fn eval_invoke_proc(dynamic_env: &SmartEnv, proc: Node, unevaled_args: Vec<Node>) -> EvalResult {
    let loc = proc.loc;
    match proc.value {
        Value::Function {
            params,
            body,
            lexical_env: parent_lexical_env,
        } => {
            // Validate params
            if unevaled_args.len() != params.len() {
                return Err(RuntimeError::ProcArgsDoNotMatchParams(String::new(), loc));
            }

            // Create the lexical environment based on the procedure's lexical parent
            let lexical_env = Env::new(Some(parent_lexical_env));

            // Prepare the arguments for evaluation
            let mut evaled_args = eval_each_node(dynamic_env, unevaled_args)?;

            // Map arguments to parameters
            for param in params {
                let evaled_arg = match evaled_args.pop() {
                    None => return Err(RuntimeError::Unknown("not enough args".to_string(), loc)),
                    Some(n) => n,
                };

                match param.value {
                    Value::Symbol(name) => {
                        lexical_env.borrow_mut().define(&name, evaled_arg)?;
                    }
                    _ => return Err(RuntimeError::Unknown("param not a symbol".to_string(), loc)),
                }
            }

            // Evaluate the application of the procedure
            eval_node(&lexical_env, *body)
        }
        _ => panic!("Cannot invoke a non-procedure"),
    }

    /* TODO
    defer func() {
		if e := recover(); e != nil {
			switch errorValue := e.(type) {
			case *EvalError:
				fmt.Printf("TRACE: (%v: %v): call to %v\n", head.Loc().Filename, head.Loc().Line, f.Name)
				panic(errorValue)
			default:
				panic(errorValue)
			}
		}
	}()
    */
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_args() {
        // Arrange
        //let args = vec![Node::new(Value::Number(42), Loc::Unknown)];
        let args = Vec::<Node>::new();

        // Act
        let r = check_args("list", &Loc::Unknown, &args, 1, -1);

        // Assert
        assert_eq!(
            r,
            Err(RuntimeError::NotEnoughArgs(
                "list".to_string(),
                1,
                0,
                Loc::Unknown
            ))
        );
    }
}
