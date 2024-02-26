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
impl From<Literal> for Value {
    fn from(value: Literal) -> Self {
        Value::Literal(value)
    }
}
impl From<Function> for Value {
    fn from(value: Function) -> Self {
        Value::Function(Box::new(value))
    }
}

#[derive(Clone)]
pub enum Throw {
    Return(Value),
    Error(SpannedError),
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
impl From<SpannedError> for Throw {
    fn from(value: SpannedError) -> Throw {
        Throw::Error(value)
    }
}
impl From<(Span, &str)> for Throw {
    fn from(value: (Span, &str)) -> Self {
        Throw::Error(SpannedError::from(value))
    }
}
impl From<(Span, String)> for Throw {
    fn from(value: (Span, String)) -> Self {
        Throw::Error(SpannedError::from(value))
    }
}

pub trait Callable<'a>: DynClone + Debug {
    fn call(&mut self, interpreter: &'a mut Interpreter, arguments: &[Value]) -> Throw;
    fn arity(&self) -> usize;
    fn as_str(&self) -> String;
}
dyn_clone::clone_trait_object!(for<'a> Callable<'a>);

#[derive(Clone, Debug)]
pub struct Function {
    name: Ident,
    params: Vec<Ident>,
    body: Vec<Stmt>,
    closure: Environment,
}
impl<'a> Callable<'a> for Function {
    fn call(&mut self, interpreter: &'a mut Interpreter, arguments: &[Value]) -> Throw {
        if arguments.len() != self.params.len() {
            return (
                self.name.span,
                format!(
                    "Function expected {} arguments but was given {}",
                    self.params.len(),
                    arguments.len()
                ),
            )
                .into();
        }
        for (i, arg) in arguments.iter().enumerate().take(self.params.len()) {
            self.closure.define(self.params[i].symbol, arg.to_owned())
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
        format!("<fn {}>", self.name.symbol)
    }
}
impl Function {
    pub fn new(name: &Ident, params: &Vec<Ident>, body: &Vec<Stmt>, closure: &Environment) -> Self {
        Self {
            name: name.to_owned(),
            params: params.to_owned(),
            body: body.to_owned(),
            closure: closure.to_owned(),
        }
    }
}

pub fn define_builtins(environment: &mut Environment) {
    environment.define_builtin::<LcClock>("clock");
    environment.define_builtin::<LcTypeof>("typeof");
    environment.define_builtin::<LcSleep>("sleep");
}

#[derive(Clone, Debug, Default)]
pub struct LcClock;
impl<'a> Callable<'a> for LcClock {
    fn call(&mut self, _: &'a mut Interpreter, _: &[Value]) -> Throw {
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

#[derive(Clone, Debug, Default)]
pub struct LcTypeof;
impl<'a> Callable<'a> for LcTypeof {
    fn call(&mut self, _: &mut Interpreter, arguments: &[Value]) -> Throw {
        if arguments.len() != self.arity() {
            return (
                Span::default(),
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
        Literal::String(Symbol::string(res.to_string())).into()
    }

    fn arity(&self) -> usize {
        1
    }

    fn as_str(&self) -> String {
        "<fn typeof>".to_string()
    }
}

#[derive(Clone, Debug, Default)]
pub struct LcSleep;
impl<'a> Callable<'a> for LcSleep {
    fn call(&mut self, _: &'a mut Interpreter, arguments: &[Value]) -> Throw {
        if arguments.len() != self.arity() {
            return (
                Span::default(),
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
                        Span::default(),
                        "sleep duration must be a number in representing milliseconds",
                    )
                        .into()
                }
            },
            Value::Function(_) => {
                return (
                    Span::default(),
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
