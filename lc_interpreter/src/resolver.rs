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
pub struct Resolver<'a, 'b> {
    interpreter: &'a mut Interpreter<'b>,
    scopes: Vec<Scope>,
    current_function: FunctionKind,
    had_error: bool,
}
impl<'a, 'b> Resolver<'a, 'b> {
    pub fn new(interpreter: &'a mut Interpreter<'b>) -> Self {
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
            Stmt::Block(statements) => self.visit_block_stmt(statements)?,
            Stmt::Expression(ex) => self.resolve_expr(&ex)?,
            Stmt::Function(id, params, body) => {
                self.visit_function_stmt(id, params, body, FunctionKind::Function)?
            }
            Stmt::If(condition, st_then, st_else) => {
                self.visit_if_stmt(&condition, st_then, st_else)?
            }
            Stmt::Print(ex) => self.resolve_expr(&ex)?,
            Stmt::Return(ex) => self.visit_return_stmt(&ex)?,
            Stmt::Let(id, initializer) => self.visit_let_stmt(id, &initializer)?,
            Stmt::While(condition, body) => self.visit_while_stmt(&condition, body)?,
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
        condition: &Expr,
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

    fn visit_return_stmt(&mut self, expr: &Expr) -> ResolverResult {
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

    fn visit_let_stmt(&mut self, id: &Token, initializer: &Expr) -> ResolverResult {
        self.declare(id)?;
        self.resolve_expr(initializer)?;
        self.define(id);
        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Box<Stmt>) -> ResolverResult {
        self.resolve_expr(condition)?;
        self.resolve_stmt(body)?;
        Ok(())
    }

    fn resolve_expr(&mut self, expr: &Expr) -> ResolverResult {
        match &expr.kind {
            ExprKind::Assign(id, ex) => self.visit_assign_expr(expr, id, ex),
            ExprKind::Binary(left, _, right) => self.visit_binary_expr(left, right),
            ExprKind::Call(callee, _, args) => self.visit_call_expr(callee, args),
            ExprKind::Grouping(ex) => self.resolve_expr(ex),
            ExprKind::Literal(_) => Ok(()),
            ExprKind::Logical(left, _, right) => self.visit_binary_expr(left, right),
            ExprKind::Unary(_, right) => self.resolve_expr(right),
            ExprKind::Variable(id) => self.visit_var_expr(expr, id),
        }
    }

    fn visit_assign_expr(&mut self, ex: &Expr, id: &Token, expr: &Box<Expr>) -> ResolverResult {
        self.resolve_expr(expr)?;
        self.resolve_local(ex, id);
        Ok(())
    }

    fn visit_binary_expr(&mut self, left: &Box<Expr>, right: &Box<Expr>) -> ResolverResult {
        self.resolve_expr(left)?;
        self.resolve_expr(right)?;
        Ok(())
    }

    fn visit_call_expr(&mut self, callee: &Box<Expr>, args: &Vec<Expr>) -> ResolverResult {
        self.resolve_expr(callee)?;
        for arg in args {
            self.resolve_expr(arg)?;
        }
        Ok(())
    }

    fn visit_var_expr(&mut self, ex: &Expr, id: &Token) -> ResolverResult {
        if let Some(initialized) = self.scopes.last_mut().and_then(|s| s.get(&id.lexeme)) {
            if !initialized {
                parser_error((id, "Can't read local variable in its own initializer.").into());
                self.had_error = true;
            }
        }

        self.resolve_local(ex, id);
        Ok(())
    }

    fn resolve_local(&mut self, ex: &Expr, id: &Token) {
        for i in (0..self.scopes.len()).rev() {
            if self
                .scopes
                .get(i)
                .is_some_and(|s| s.contains_key(&id.lexeme))
            {
                self.interpreter.resolve(ex, self.scopes.len() - 1 - i);
                return;
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
