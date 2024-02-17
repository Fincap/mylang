use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::*;
use lc_core::*;

type ParentEnv = Rc<RefCell<Environment>>;

#[derive(Clone, Debug)]
pub struct Environment {
    enclosing: Option<ParentEnv>,
    values: HashMap<String, Value>,
}
impl Environment {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn with_parent(enclosing: ParentEnv) -> Self {
        Self {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Value, TokenError> {
        if let Some(value) = self.values.get(&name.lexeme) {
            Ok(value.clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow().get(name)
        } else {
            Err((name, format!("Undefined variable '{}'", name.lexeme)).into())
        }
    }

    pub fn get_at(&self, name: &Token, depth: usize) -> Result<Value, TokenError> {
        Ok(self
            .ancestor(depth)
            .borrow()
            .values
            .get(&name.lexeme)
            .unwrap()
            .to_owned())
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> Result<(), TokenError> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.to_owned(), value);
            Ok(())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().assign(name, value)
        } else {
            Err((name, format!("Undefined variable '{}'", name.lexeme)).into())
        }
    }

    pub fn assign_at(&self, name: &Token, value: Value, depth: usize) -> Result<(), TokenError> {
        dbg!(&name);
        dbg!(&depth);
        self.ancestor(depth)
            .borrow_mut()
            .values
            .insert(name.lexeme.to_owned(), value);
        Ok(())
    }

    fn ancestor(&self, depth: usize) -> ParentEnv {
        let mut environment = self.to_owned();
        for _ in 0..depth {
            environment = environment.enclosing.unwrap().borrow().to_owned()
        }
        Rc::new(RefCell::new(environment))
    }
}
