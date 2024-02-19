use core::fmt;
use std::{collections::HashMap, io};

use crate::*;
use lc_core::*;

type ExprResult = Result<Value, Throw>;
type StmtResult = Result<(), Throw>;

pub struct Interpreter<'a> {
    pub environment: EnvironmentStack,
    locals: HashMap<Expr, usize>,
    output: &'a mut dyn io::Write,
}
impl<'a> fmt::Debug for Interpreter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Interpreter")
            .field("environment", &self.environment)
            .field("locals", &self.locals)
            .finish()
    }
}
impl<'a> Interpreter<'a> {
    pub fn new(output: &'a mut dyn io::Write) -> Self {
        let mut globals = Environment::new();
        define_builtins(&mut globals);
        let environment = EnvironmentStack::new(globals);
        Self {
            environment,
            locals: HashMap::new(),
            output,
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<(), RuntimeError> {
        for statement in &statements {
            if let Err(e) = self.execute(statement) {
                if let Throw::Error(e) = e {
                    return Err(e.into());
                }
                break;
            }
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> StmtResult {
        self.visit_stmt(stmt)
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> StmtResult {
        match stmt {
            Stmt::Block(statements) => self.visit_block_stmt(statements),
            Stmt::Expression(ex) => self.visit_expr_stmt(ex),
            Stmt::Function(name, params, body) => self.visit_fn_stmt(name, params, body),
            Stmt::If(condition, st_then, st_else) => {
                self.visit_if_stmt(condition, st_then, st_else)
            }
            Stmt::Print(ex) => self.visit_print_stmt(ex),
            Stmt::Return(ex) => self.visit_return_stmt(ex),
            Stmt::Let(id, initializer) => self.visit_let_stmt(id, initializer),
            Stmt::While(condition, body) => self.visit_while_stmt(condition, body),
        }
    }

    pub fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        environment: &Environment,
    ) -> StmtResult {
        self.environment.begin_scope(environment.to_owned());
        for statement in statements {
            if let Err(e) = self.execute(statement) {
                self.environment.end_scope();
                return Err(e);
            }
        }
        self.environment.end_scope();
        Ok(())
    }

    fn visit_block_stmt(&mut self, statements: &Vec<Stmt>) -> StmtResult {
        self.execute_block(statements, &Environment::new())
    }

    fn visit_expr_stmt(&mut self, ex: &Expr) -> StmtResult {
        match self.evaluate(ex) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    fn visit_fn_stmt(&mut self, name: &Token, params: &Vec<Token>, body: &Vec<Stmt>) -> StmtResult {
        let function = Function::new(name, params, body, &self.environment.top());
        self.environment
            .define(name.lexeme.to_owned(), function.into());
        Ok(())
    }

    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        st_then: &Box<Stmt>,
        st_else: &Option<Box<Stmt>>,
    ) -> StmtResult {
        if self.evaluate(condition)?.is_truthy() {
            self.execute(&st_then)?;
        } else if let Some(st_else) = st_else {
            self.execute(&st_else)?;
        }
        Ok(())
    }

    fn visit_print_stmt(&mut self, ex: &Expr) -> StmtResult {
        match self.evaluate(ex) {
            Ok(lit) => {
                writeln!(self.output, "{}", lit.as_str()).unwrap();
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn visit_return_stmt(&mut self, ex: &Expr) -> StmtResult {
        let value = self.evaluate(ex)?;
        Err(value.into())
    }

    fn visit_let_stmt(&mut self, id: &Token, initializer: &Expr) -> StmtResult {
        let value = self.evaluate(initializer)?;
        self.environment.define(id.lexeme.to_owned(), value.into());
        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Box<Stmt>) -> StmtResult {
        while self.evaluate(condition)?.is_truthy() {
            self.execute(&body)?;
        }
        Ok(())
    }

    fn evaluate(&mut self, ex: &Expr) -> ExprResult {
        self.visit_expr(ex)
    }

    fn visit_expr(&mut self, expr: &Expr) -> ExprResult {
        match &expr.kind {
            ExprKind::Assign(id, right) => self.visit_assign_expr(expr, id, right),
            ExprKind::Binary(left, op, right) => self.visit_binary_expr(left, op, right),
            ExprKind::Call(callee, paren, args) => self.visit_call_expr(callee, paren, args),
            ExprKind::Grouping(ex) => self.evaluate(ex),
            ExprKind::Literal(lit) => Ok(lit.to_owned().into()),
            ExprKind::Logical(left, op, right) => self.visit_logical_expr(left, op, right),
            ExprKind::Unary(op, right) => self.visit_unary_expr(op, right),
            ExprKind::Variable(id) => self.visit_var_expr(expr, &id),
        }
    }

    fn visit_assign_expr(&mut self, ex: &Expr, id: &Token, right: &Box<Expr>) -> ExprResult {
        let value = self.evaluate(right)?;
        if let Some(distance) = self.locals.get(ex) {
            self.environment
                .assign_at(id, value.to_owned(), *distance)?;
        } else {
            self.environment
                .global_assign(id, value.to_owned().into())?;
        }
        Ok(value)
    }

    fn visit_binary_expr(&mut self, left: &Box<Expr>, op: &Token, right: &Box<Expr>) -> ExprResult {
        let Value::Literal(left) = self.evaluate(&left)? else {
            return Err((
                op,
                "Operands must be two numbers or two strings. Did you forget to call the function?",
            )
                .into());
        };
        let Value::Literal(right) = self.evaluate(&right)? else {
            return Err((
                op,
                "Operands must be two numbers or two strings. Did you forget to call the function?",
            )
                .into());
        };
        match op.kind {
            TokenKind::Minus => {
                let (left, right) = self.get_number_ops(&left, op, &right)?;
                Ok(Literal::Number(left - right).into())
            }
            TokenKind::Slash => {
                let (left, right) = self.get_number_ops(&left, op, &right)?;
                Ok(Literal::Number(left / right).into())
            }
            TokenKind::Star => {
                let (left, right) = self.get_number_ops(&left, op, &right)?;
                Ok(Literal::Number(left * right).into())
            }
            TokenKind::Plus => match left {
                Literal::Number(_) => {
                    let (left, right) = self.get_number_ops(&left, op, &right)?;
                    Ok(Literal::Number(left + right).into())
                }
                Literal::String(str) => {
                    let Literal::String(right) = right else {
                        return Err((op, "Cannot concatenate non-string value.").into());
                    };
                    Ok(Literal::String(str.to_owned() + &right).into())
                }
                _ => Err((op, "Operands must be two numbers or two strings.").into()),
            },
            TokenKind::Greater => {
                let (left, right) = self.get_number_ops(&left, op, &right)?;
                Ok(Literal::Bool(left > right).into())
            }
            TokenKind::GreaterEqual => {
                let (left, right) = self.get_number_ops(&left, op, &right)?;
                Ok(Literal::Bool(left >= right).into())
            }
            TokenKind::Less => {
                let (left, right) = self.get_number_ops(&left, op, &right)?;
                Ok(Literal::Bool(left < right).into())
            }
            TokenKind::LessEqual => {
                let (left, right) = self.get_number_ops(&left, op, &right)?;
                Ok(Literal::Bool(left <= right).into())
            }
            TokenKind::BangEqual => Ok(Literal::Bool(left != right).into()),
            TokenKind::EqualEqual => Ok(Literal::Bool(left == right).into()),
            _ => Err((
                op,
                "Interpreter data corruption, binary expression has invalid operator",
            )
                .into()),
        }
    }

    fn visit_call_expr(
        &mut self,
        callee: &Box<Expr>,
        paren: &Token,
        args: &Vec<Expr>,
    ) -> ExprResult {
        let ExprKind::Variable(identifier) = &callee.kind else {
            return Err((paren, "Not a valid function call.").into());
        };
        let mut arguments = Vec::new();
        for arg in args {
            arguments.push(self.evaluate(arg)?);
        }
        let value = self.environment.get(&identifier)?;
        match value {
            Value::Literal(_) => Err((identifier, "Not a valid function call.").into()),
            Value::Function(mut func) => match func.call(self, &arguments) {
                Throw::Return(value) => Ok(value),
                Throw::Error(err) => Err(err.into()), // only keep propagating up call stack if it was an *actual* error
            },
        }
    }

    fn visit_logical_expr(
        &mut self,
        left: &Box<Expr>,
        op: &Token,
        right: &Box<Expr>,
    ) -> ExprResult {
        let left = self.evaluate(&left)?;
        if op.kind == TokenKind::Or {
            if left.is_truthy() {
                return Ok(left);
            }
        } else {
            if !left.is_truthy() {
                return Ok(left);
            }
        }
        self.evaluate(right)
    }

    fn visit_unary_expr(&mut self, op: &Token, right: &Box<Expr>) -> ExprResult {
        let Value::Literal(right) = self.evaluate(right)? else {
            return Err((
                op,
                "Unary operand must be numeric. Did you forget to call the function?",
            )
                .into());
        };
        match op.kind {
            TokenKind::Minus => match right {
                Literal::Number(num) => Ok(Literal::Number(-num).into()),
                _ => Err((op, "Unary operand must be numeric.").into()),
            },
            TokenKind::Bang => Ok(Literal::Bool(!right.is_truthy()).into()),
            _ => Err((
                op,
                "Interpreter data corruption, unary expression has invalid operator",
            )
                .into()),
        }
    }

    fn visit_var_expr(&mut self, ex: &Expr, id: &Token) -> ExprResult {
        self.look_up_variable(ex, id)
    }

    pub fn resolve(&mut self, ex: &Expr, depth: usize) {
        self.locals.insert(ex.to_owned(), depth);
    }

    fn look_up_variable(&self, ex: &Expr, id: &Token) -> ExprResult {
        match self.locals.get(ex) {
            Some(distance) => Ok(self.environment.get_at(&id, *distance)?),
            None => Ok(self.environment.global_get(&id)?),
        }
    }

    fn get_number_ops(
        &self,
        left: &Literal,
        op: &Token,
        right: &Literal,
    ) -> Result<(f64, f64), TokenError> {
        let Literal::Number(left) = *left else {
            return Err((op, "Left operand must be a number.").into());
        };
        let Literal::Number(right) = *right else {
            return Err((op, "Right operand must be a number.").into());
        };
        Ok((left, right))
    }
}
