use std::collections::HashMap;

use crate::*;
use lc_core::*;

#[derive(Clone, Debug)]
pub struct EnvironmentStack {
    stack: Vec<Environment>,
}
impl EnvironmentStack {
    pub fn new(globals: Environment) -> Self {
        Self {
            stack: vec![globals],
        }
    }

    pub fn top(&self) -> Environment {
        self.stack.last().unwrap().clone()
    }

    pub fn begin_scope(&mut self, environment: Environment) {
        self.stack.push(environment);
    }

    pub fn end_scope(&mut self) {
        self.stack.pop();
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.stack.last_mut().unwrap().define(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Value, SpannedError> {
        for env in self.stack.iter().rev() {
            if let Ok(value) = env.get(name) {
                return Ok(value);
            }
        }
        Err((name, format!("Undefined variable '{}'", name.lexeme)).into())
    }

    pub fn get_at(&self, name: &Token, depth: usize) -> Result<Value, SpannedError> {
        self.stack
            .get(self.stack.len() - 1 - depth)
            .unwrap()
            .get(name)
    }

    pub fn global_get(&self, name: &Token) -> Result<Value, SpannedError> {
        self.stack.first().unwrap().get(name)
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> Result<(), SpannedError> {
        for env in self.stack.iter_mut().rev() {
            if env.contains(name) {
                if env.assign(name, value.to_owned()).is_ok() {
                    return Ok(());
                }
            }
        }
        Err((name, format!("Undefined variable '{}'", name.lexeme)).into())
    }

    pub fn assign_at(
        &mut self,
        name: &Token,
        value: Value,
        depth: usize,
    ) -> Result<(), SpannedError> {
        let index = self.stack.len() - 1 - depth;
        self.stack.get_mut(index).unwrap().assign(name, value)
    }

    pub fn global_assign(&mut self, name: &Token, value: Value) -> Result<(), SpannedError> {
        self.stack.first_mut().unwrap().assign(name, value)
    }
}

#[derive(Clone, Debug)]
pub struct Environment {
    values: HashMap<String, Value>,
}
impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Value, SpannedError> {
        if let Some(value) = self.values.get(&name.lexeme) {
            Ok(value.clone())
        } else {
            Err((name, format!("Undefined variable '{}'", name.lexeme)).into())
        }
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> Result<(), SpannedError> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.to_owned(), value);
            Ok(())
        } else {
            Err((name, format!("Undefined variable '{}'", name.lexeme)).into())
        }
    }

    pub fn contains(&self, name: &Token) -> bool {
        self.values.contains_key(&name.lexeme)
    }
}
