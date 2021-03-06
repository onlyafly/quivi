use back::env::SmartEnv;
use back::eval::NodeResult;
use back::runtime_error::RuntimeError;
use loc::Loc;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::rc::Rc;

#[derive(PartialEq, Debug, Clone)]
pub enum Val {
    Nil,
    Error(String),
    Number(i32),
    Character(String),
    StringVal(String),
    Symbol(String),
    Boolean(bool),
    Routine(RoutineObj),
    Primitive(PrimitiveObj),
    List(Vec<Node>),
    Writer(WriterObj),
    Reader(ReaderObj),
    Environment(SmartEnv),
    Cell(CellObj),
}

impl Display for Val {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Val::Nil => write!(f, "nil"),
            Val::Error(ref s) => write!(f, "#error<{}>", s),
            Val::Number(n) => write!(f, "{}", n),
            Val::StringVal(ref s) => write!(f, "\"{}\"", s),
            Val::Character(ref s) => match s.as_ref() {
                "\n" => write!(f, r"\newline"),
                _ => write!(f, r"\{}", s),
            },
            Val::Symbol(ref s) => write!(f, "{}", s),
            Val::List(ref children) => {
                let mut v = Vec::new();
                for child in children {
                    v.push(format!("{}", child.val));
                }
                write!(f, "({})", &v.join(" "))
            }
            Val::Boolean(false) => write!(f, "false"),
            Val::Boolean(true) => write!(f, "true"),
            Val::Routine(RoutineObj {
                routine_type: RoutineType::Function,
                name: None,
                ..
            }) => write!(f, "#function"),
            Val::Routine(RoutineObj {
                routine_type: RoutineType::Function,
                name: Some(s),
                ..
            }) => write!(f, "#function<{}>", s),
            Val::Routine(RoutineObj {
                routine_type: RoutineType::Macro,
                name: None,
                ..
            }) => write!(f, "#macro"),
            Val::Routine(RoutineObj {
                routine_type: RoutineType::Macro,
                name: Some(s),
                ..
            }) => write!(f, "#macro<{}>", s),
            Val::Primitive(PrimitiveObj { name, .. }) => write!(f, "#primitive<{}>", name),
            Val::Writer(..) => write!(f, "#writer"),
            Val::Reader(..) => write!(f, "#reader"),
            Val::Environment(env) => write!(f, "#environment<{}>", env.borrow().name),
            Val::Cell(obj) => write!(f, "(cell {})", obj),
        }
    }
}

impl Val {
    pub fn type_name(&self) -> Result<String, RuntimeError> {
        let out = match self {
            Val::Nil => "nil",
            Val::Error(..) => "error",
            Val::Number(..) => "number",
            Val::StringVal(..) => "string",
            Val::Character(..) => "char",
            Val::Symbol(..) => "symbol",
            Val::List(..) => "list",
            Val::Boolean(..) => "boolean",
            Val::Routine(..) => "function",
            Val::Primitive(..) => "primitive",
            Val::Writer(..) => "writer",
            Val::Reader(..) => "reader",
            Val::Environment(..) => "environment",
            Val::Cell(..) => "cell",
        };

        Ok(out.to_string())
    }
}

impl PartialOrd for Val {
    fn partial_cmp(&self, other: &Val) -> Option<Ordering> {
        use self::Val::*;
        match (self, other) {
            (Number(a), Number(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub val: Val,
    pub loc: Loc,
}

impl Node {
    pub fn new(val: Val, loc: Loc) -> Self {
        Node { val, loc }
    }

    pub fn as_print_friendly_string(&self) -> String {
        match self.val {
            Val::StringVal(ref s) => format!("{}", s),
            Val::Character(ref s) => format!("{}", s),
            ref v => format!("{}", v), // Use Display's fmt for everything else
        }
    }

    pub fn as_host_number(&self) -> Result<i32, RuntimeError> {
        match self.val {
            Val::Number(i) => Ok(i),
            _ => Err(RuntimeError::UnexpectedValue(
                "number".to_string(),
                self.val.clone(),
                self.loc.clone(),
            )),
        }
    }

    pub fn as_host_boolean(&self) -> Result<bool, RuntimeError> {
        match self.val {
            Val::Nil => Ok(false),
            Val::Boolean(b) => Ok(b),
            _ => Ok(true),
        }
    }

    /* TODO
    pub fn as_host_vector(&self) -> Result<bool, RuntimeError> {
        match self {
            &Val::Nil => Ok(false),
            &Val::Boolean(b) => Ok(b),
            _ => Ok(true),
        }
    } */

    pub fn coll_len(self) -> Result<usize, RuntimeError> {
        match self.val {
            Val::Nil => Ok(0),
            Val::StringVal(s) => Ok(s.chars().count()),
            Val::List(children) => Ok(children.len()),
            v => Err(RuntimeError::CannotGetLengthOfNonCollection(v, self.loc)),
        }
    }

    pub fn coll_cons(self, elem: Node) -> Result<Node, RuntimeError> {
        let loc = self.loc;
        match self.val {
            Val::Nil => Ok(Node::new(Val::List(vec![elem]), loc)),
            Val::StringVal(s) => {
                let loc = elem.loc;
                let out = match elem.val {
                    Val::Character(c) => format!("{}{}", c, s),
                    v => return Err(RuntimeError::CannotConsNonCharacterOntoString(v, loc)),
                };
                Ok(Node::new(Val::StringVal(out), loc))
            }
            Val::List(mut children) => {
                children.insert(0, elem);
                Ok(Node::new(Val::List(children), loc))
            }
            v => Err(RuntimeError::CannotConsOntoNonCollection(v, loc)),
        }
    }

    fn coll_children(self) -> Result<Vec<Node>, RuntimeError> {
        let loc = self.loc;
        match self.val {
            Val::Nil => Ok(Vec::new()),
            Val::StringVal(s) => Ok(s
                .chars()
                .map(|c| Node::new(Val::Character(format!("{}", c)), loc.clone()))
                .collect()),
            Val::List(children) => Ok(children),
            v => Err(RuntimeError::CannotGetChildrenOfNonCollection(
                "append".to_string(),
                v,
                loc,
            )),
        }
    }

    pub fn coll_append(self, other: Node) -> Result<Node, RuntimeError> {
        match self.val {
            Val::Nil => Ok(other),
            Val::StringVal(s) => {
                let output = format!("{}{}", s, other.as_print_friendly_string());
                Ok(Node::new(Val::StringVal(output), self.loc))
            }
            Val::List(mut children) => {
                let mut other_children = other.coll_children()?;
                children.append(&mut other_children);
                Ok(Node::new(Val::List(children), self.loc))
            }
            v => Err(RuntimeError::CannotAppendOnto(v, self.loc)),
        }
    }
}

// By implementing manually, we ensure that the metadata (loc, etc) is not checked
// for equality
impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.val == other.val
    }
}

impl Deref for Node {
    type Target = Val;

    fn deref(&self) -> &Val {
        &self.val
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum RoutineType {
    Function,
    Macro,
}

#[derive(PartialEq, Debug, Clone)]
pub struct RoutineObj {
    pub name: Option<String>,
    pub params: Vec<Node>,
    pub body: Box<Node>,
    pub lexical_env: SmartEnv,
    pub routine_type: RoutineType,
}

pub type PrimitiveFnPointer = fn(SmartEnv, Node, Vec<Node>) -> NodeResult;

#[derive(PartialEq, Debug, Clone)]
pub struct PrimitiveObj {
    pub name: String,
    pub f: PrimitiveFnPointer,
    pub min_arity: isize,
    pub max_arity: isize,
}

#[derive(PartialEq, Debug, Clone)]
pub enum WriterObj {
    Sink,
    Standard,
    Buffer(Rc<RefCell<Vec<u8>>>),
}

#[derive(PartialEq, Debug, Clone)]
pub struct ReaderObj {
    pub reader_function: fn() -> Result<String, String>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct CellObj {
    pub contents: Rc<RefCell<Node>>,
}

impl CellObj {
    pub fn new(n: Node) -> Self {
        CellObj {
            contents: Rc::new(RefCell::new(n)),
        }
    }
}

impl Display for CellObj {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.contents.borrow_mut().val)
    }
}
