use ast::{Node, Val};
use back::runtime_error::RuntimeError;
use loc::Loc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type SmartEnv = Rc<RefCell<Env>>;

#[derive(PartialEq, Debug)]
pub struct Env {
    pub name: String,
    pub map: HashMap<String, Node>,
    pub parent: Option<SmartEnv>,
}

impl Env {
    pub fn new(parent: Option<SmartEnv>) -> SmartEnv {
        let name = match parent {
            None => "TopLevel",
            Some(..) => "Local",
        };
        let e = Env {
            name: name.to_string(),
            map: HashMap::new(),
            parent,
        };
        Rc::new(RefCell::new(e))
    }

    // Define a new variable, or update an existing one
    pub fn define(&mut self, k: &str, v: Node) -> Result<(), RuntimeError> {
        self.map.insert(k.to_string(), v);
        Ok(())
    }

    pub fn update(&mut self, k: &str, v: Node) -> Result<(), RuntimeError> {
        let kstring = k.to_string();

        if self.map.contains_key(k) {
            self.map.insert(kstring, v);
            Ok(())
        } else {
            match self.parent {
                Some(ref parent_env) => parent_env.borrow_mut().update(k, v),
                None => Err(RuntimeError::CannotUpdateUndefinedName(kstring, v.loc)),
            }
        }
    }

    #[allow(dead_code)]
    pub fn exists(&mut self, k: &str) -> bool {
        self.map.contains_key(k)
    }

    pub fn get(&self, name: &str) -> Option<Node> {
        match self.map.get(name) {
            Some(node) => Some(node.clone()),
            None => match self.parent {
                Some(ref parent_env) => parent_env.borrow().get(name),
                None => None,
            },
        }
    }

    pub fn remove(&mut self, k: &str) -> Option<Node> {
        let val = self.map.remove(k);
        // Reinsert nil here so that a later update will update the correct hashmap
        self.map
            .insert(k.to_string(), Node::new(Val::Number(0), Loc::Unknown)); //TODO: should be nil
        val
    }
}
