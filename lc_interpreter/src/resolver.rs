use std::collections::HashMap;

use lc_core::*;

use crate::*;

type Scope = HashMap<String, bool>;
type ResolverResult = Result<(), TokenError>;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FunctionKind {
    None,
    Function,
}

#[derive(Debug)]
pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<Scope>,
    current_function: FunctionKind,
    had_error: bool,
}
impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionKind::None,
            had_error: false,
        }
    }

    pub fn had_error(&self) -> bool {
        self.had_error
    }

    pub fn resolve(&mut self, statements: &Vec<Stmt>) -> ResolverResult {
        for stmt in statements {
            if let Err(e) = self.resolve_stmt(stmt) {
                parser_error(e);
                self.had_error = true;
            }
        }
        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) -> ResolverResult {
        match stmt {
            Stmt::Block(ex) => self.visit_block_stmt(ex)?,
            Stmt::Expression(ex) => self.resolve_expr(ex)?,
            Stmt::Function(id, params, body) => {
                self.visit_function_stmt(id, params, body, FunctionKind::Function)?
            }
            Stmt::If(condition, st_then, st_else) => {
                self.visit_if_stmt(condition, st_then, st_else)?
            }
            Stmt::Print(ex) => self.resolve_expr(ex)?,
            Stmt::Return(ex) => self.visit_return_stmt(ex)?,
            Stmt::Let(id, initializer) => self.visit_let_stmt(id, initializer)?,
            Stmt::While(condition, body) => self.visit_while_stmt(condition, body)?,
        };
        Ok(())
    }

    fn visit_block_stmt(&mut self, statements: &Vec<Stmt>) -> ResolverResult {
        self.begin_scope();
        self.resolve(statements)?;
        self.end_scope();
        Ok(())
    }

    fn visit_if_stmt(
        &mut self,
        condition: &ExprKind,
        st_then: &Box<Stmt>,
        st_else: &Option<Box<Stmt>>,
    ) -> ResolverResult {
        self.resolve_expr(condition)?;
        self.resolve_stmt(&st_then)?;
        if let Some(st_else) = st_else {
            self.resolve_stmt(&st_else)?;
        }
        Ok(())
    }

    fn visit_return_stmt(&mut self, expr: &ExprKind) -> ResolverResult {
        if self.current_function == FunctionKind::None {
            Err((
                &Token::new(TokenType::Return, "return".to_string(), 0),
                "Can't return from top-level code",
            )
                .into())
        } else {
            self.resolve_expr(expr)
        }
    }

    fn visit_function_stmt(
        &mut self,
        id: &Token,
        params: &Vec<Token>,
        body: &Vec<Stmt>,
        kind: FunctionKind,
    ) -> ResolverResult {
        self.declare(id)?;
        self.define(id);

        let enclosing = self.current_function;
        self.current_function = kind;
        self.begin_scope();
        for param in params {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve(body)?;
        self.end_scope();
        self.current_function = enclosing;
        Ok(())
    }

    fn visit_let_stmt(&mut self, id: &Token, initializer: &ExprKind) -> ResolverResult {
        self.declare(id)?;
        self.resolve_expr(initializer)?;
        self.define(id);
        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: &ExprKind, body: &Box<Stmt>) -> ResolverResult {
        self.resolve_expr(condition)?;
        self.resolve_stmt(body)?;
        Ok(())
    }

    fn resolve_expr(&mut self, expr: &ExprKind) -> ResolverResult {
        match expr {
            ExprKind::Assign(id, ex) => self.visit_assign_expr(id, ex),
            ExprKind::Binary(left, _, right) => self.visit_binary_expr(left, right),
            ExprKind::Call(callee, _, args) => self.visit_call_expr(callee, args),
            ExprKind::Grouping(ex) => self.resolve_expr(ex),
            ExprKind::Literal(_) => Ok(()),
            ExprKind::Logical(left, _, right) => self.visit_binary_expr(left, right),
            ExprKind::Unary(_, right) => self.resolve_expr(right),
            ExprKind::Variable(id) => self.visit_var_expr(id),
        }
    }

    fn visit_assign_expr(&mut self, id: &Token, expr: &Box<ExprKind>) -> ResolverResult {
        self.resolve_expr(expr)?;
        self.resolve_local(id);
        Ok(())
    }

    fn visit_binary_expr(&mut self, left: &Box<ExprKind>, right: &Box<ExprKind>) -> ResolverResult {
        self.resolve_expr(left)?;
        self.resolve_expr(right)?;
        Ok(())
    }

    fn visit_call_expr(&mut self, callee: &Box<ExprKind>, args: &Vec<ExprKind>) -> ResolverResult {
        self.resolve_expr(callee)?;
        for arg in args {
            self.resolve_expr(arg)?;
        }
        Ok(())
    }

    fn visit_var_expr(&mut self, id: &Token) -> ResolverResult {
        if let Some(initialized) = self.scopes.last_mut().and_then(|s| s.get(&id.lexeme)) {
            if !initialized {
                parser_error((id, "Can't read local variable in its own initializer.").into());
                self.had_error = true;
            }
        }

        self.resolve_local(id);
        Ok(())
    }

    fn resolve_local(&mut self, id: &Token) {
        for i in (0..self.scopes.len()).rev() {
            if self
                .scopes
                .get(i)
                .is_some_and(|s| s.contains_key(&id.lexeme))
            {
                self.interpreter.resolve(id, self.scopes.len() - 1 - i);
            }
        }
    }

    fn declare(&mut self, id: &Token) -> ResolverResult {
        let Some(scope) = self.scopes.last_mut() else {
            return Ok(());
        };
        if scope.contains_key(&id.lexeme) {
            return Err((id, "Already a variable with this name in this scope.").into());
        }
        scope.insert(id.lexeme.to_owned(), false);
        Ok(())
    }

    fn define(&mut self, id: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(id.lexeme.to_owned(), true);
        };
    }

    fn begin_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }
}
