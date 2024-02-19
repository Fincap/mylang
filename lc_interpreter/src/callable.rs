use std::{
    fmt::Debug,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use dyn_clone::DynClone;

use crate::*;
use lc_core::*;

#[derive(Clone, Debug)]
pub enum Value {
    Literal(Literal),
    Function(Box<dyn for<'a> Callable<'a>>),
}
impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Literal(lit) => lit.is_truthy(),
            Value::Function(_) => false,
        }
    }

    pub fn as_str(&self) -> String {
        match self {
            Value::Literal(lit) => lit.as_str(),
            Value::Function(func) => func.as_str(),
        }
    }
}
impl Into<Value> for Literal {
    fn into(self) -> Value {
        Value::Literal(self)
    }
}
impl Into<Value> for Function {
    fn into(self) -> Value {
        Value::Function(Box::new(self))
    }
}

#[derive(Clone)]
pub enum Throw {
    Return(Value),
    Error(TokenError),
}
impl From<Literal> for Throw {
    fn from(value: Literal) -> Throw {
        Throw::Return(Value::Literal(value))
    }
}
impl From<Value> for Throw {
    fn from(value: Value) -> Throw {
        Throw::Return(value)
    }
}
impl From<TokenError> for Throw {
    fn from(value: TokenError) -> Throw {
        Throw::Error(value)
    }
}
impl From<(&Token, &str)> for Throw {
    fn from(value: (&Token, &str)) -> Self {
        Throw::Error(TokenError::from(value))
    }
}
impl From<(&Token, String)> for Throw {
    fn from(value: (&Token, String)) -> Self {
        Throw::Error(TokenError::from(value))
    }
}

pub trait Callable<'a>: DynClone + Debug {
    fn call(&mut self, interpreter: &'a mut Interpreter, arguments: &Vec<Value>) -> Throw;
    fn arity(&self) -> usize;
    fn as_str(&self) -> String;
}
dyn_clone::clone_trait_object!(for<'a> Callable<'a>);

#[derive(Clone, Debug)]
pub struct Function {
    name: Token,
    params: Vec<Token>,
    body: Vec<Stmt>,
    closure: Environment,
}
impl<'a> Callable<'a> for Function {
    fn call(&mut self, interpreter: &'a mut Interpreter, arguments: &Vec<Value>) -> Throw {
        if arguments.len() != self.params.len() {
            return (
                &self.name,
                format!(
                    "Function expected {} arguments but was given {}",
                    self.params.len(),
                    arguments.len()
                ),
            )
                .into();
        }
        for i in 0..self.params.len() {
            self.closure
                .define(self.params[i].lexeme.to_owned(), arguments[i].to_owned());
        }

        match interpreter.execute_block(&self.body, &self.closure) {
            Ok(_) => Literal::Null.into(),
            Err(throw) => throw,
        }
    }

    fn arity(&self) -> usize {
        self.params.len()
    }

    fn as_str(&self) -> String {
        format!("<fn {}>", self.name.lexeme)
    }
}
impl Function {
    pub fn new(name: &Token, params: &Vec<Token>, body: &Vec<Stmt>, closure: &Environment) -> Self {
        Self {
            name: name.to_owned(),
            params: params.to_owned(),
            body: body.to_owned(),
            closure: closure.to_owned(),
        }
    }
}

pub fn define_builtins(environment: &mut Environment) {
    environment.define("clock".into(), Value::Function(Box::new(LcClock)));
    environment.define("typeof".into(), Value::Function(Box::new(LcTypeof)));
    environment.define("sleep".into(), Value::Function(Box::new(LcSleep)));
}

#[derive(Clone, Debug)]
pub struct LcClock;
impl<'a> Callable<'a> for LcClock {
    fn call(&mut self, _: &'a mut Interpreter, _: &Vec<Value>) -> Throw {
        Literal::Number(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        )
        .into()
    }

    fn arity(&self) -> usize {
        0
    }

    fn as_str(&self) -> String {
        "<fn clock>".to_string()
    }
}

#[derive(Clone, Debug)]
pub struct LcTypeof;
impl<'a> Callable<'a> for LcTypeof {
    fn call(&mut self, _: &mut Interpreter, arguments: &Vec<Value>) -> Throw {
        if arguments.len() != self.arity() {
            return (
                &Token::new(TokenKind::Fn, self.as_str(), Span::new(0)),
                format!(
                    "Function expected {} arguments but was given {}",
                    self.arity(),
                    arguments.len()
                ),
            )
                .into();
        }
        let res = match &arguments[0] {
            Value::Literal(lit) => match lit {
                Literal::String(_) => "String",
                Literal::Number(_) => "Number",
                Literal::Bool(_) => "Bool",
                Literal::Null => "Null",
            },
            Value::Function(_) => "Function",
        };
        Literal::String(res.to_string()).into()
    }

    fn arity(&self) -> usize {
        1
    }

    fn as_str(&self) -> String {
        "<fn typeof>".to_string()
    }
}

#[derive(Clone, Debug)]
pub struct LcSleep;
impl<'a> Callable<'a> for LcSleep {
    fn call(&mut self, _: &'a mut Interpreter, arguments: &Vec<Value>) -> Throw {
        if arguments.len() != self.arity() {
            return (
                &Token::new(TokenKind::Fn, self.as_str(), Span::new(0)),
                format!(
                    "Function expected {} arguments but was given {}",
                    self.arity(),
                    arguments.len()
                ),
            )
                .into();
        }
        let duration = match &arguments[0] {
            Value::Literal(lit) => match lit {
                Literal::Number(num) => Duration::from_secs_f64(num / 1000.0),
                _ => {
                    return (
                        &Token::new(TokenKind::Fn, self.as_str(), Span::new(0)),
                        "sleep duration must be a number in representing milliseconds",
                    )
                        .into()
                }
            },
            Value::Function(_) => {
                return (
                    &Token::new(TokenKind::Fn, self.as_str(), Span::new(0)),
                    "sleep duration must be a number in representing milliseconds",
                )
                    .into()
            }
        };
        thread::sleep(duration);
        Literal::Null.into()
    }

    fn arity(&self) -> usize {
        1
    }

    fn as_str(&self) -> String {
        "<fn sleep>".to_string()
    }
}
