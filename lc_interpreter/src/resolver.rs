use std::collections::HashMap;

use lc_core::*;

use crate::*;

type Scope = HashMap<String, bool>;
type ResolverResult = Result<(), SpannedError>;

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
    errors: Vec<SpannedError>,
}
impl<'a, 'b> Resolver<'a, 'b> {
    pub fn new(interpreter: &'a mut Interpreter<'b>) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionKind::None,
            errors: Vec::new(),
        }
    }

    pub fn resolve(&mut self, statements: &Vec<Stmt>) -> TranslationResult<()> {
        let _ = self.resolve_statements(statements);
        ((), self.errors.clone().into())
    }

    fn resolve_statements(&mut self, statements: &Vec<Stmt>) -> ResolverResult {
        for stmt in statements {
            if let Err(e) = self.resolve_stmt(stmt) {
                self.report_error(e);
            }
        }
        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) -> ResolverResult {
        match stmt {
            Stmt::Block(statements) => self.visit_block_stmt(statements)?,
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
        self.resolve_statements(statements)?;
        self.end_scope();
        Ok(())
    }

    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        st_then: &Stmt,
        st_else: &Option<Box<Stmt>>,
    ) -> ResolverResult {
        self.resolve_expr(condition)?;
        self.resolve_stmt(st_then)?;
        if let Some(st_else) = st_else {
            self.resolve_stmt(st_else)?;
        }
        Ok(())
    }

    fn visit_return_stmt(&mut self, expr: &Expr) -> ResolverResult {
        if self.current_function == FunctionKind::None {
            Err((
                &Token::new(TokenKind::Return, "return".to_string(), Span::default()),
                "Can't return from top-level code",
            )
                .into())
        } else {
            self.resolve_expr(expr)
        }
    }

    fn visit_function_stmt(
        &mut self,
        id: &Ident,
        params: &Vec<Ident>,
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
        self.resolve_statements(body)?;
        self.end_scope();
        self.current_function = enclosing;
        Ok(())
    }

    fn visit_let_stmt(&mut self, id: &Ident, initializer: &Expr) -> ResolverResult {
        self.declare(id)?;
        self.resolve_expr(initializer)?;
        self.define(id);
        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> ResolverResult {
        self.resolve_expr(condition)?;
        self.resolve_stmt(body)?;
        Ok(())
    }

    fn resolve_expr(&mut self, expr: &Expr) -> ResolverResult {
        match &expr.kind {
            ExprKind::Assign(id, initializer) => self.visit_assign_expr(expr, id, initializer),
            ExprKind::Binary(left, _, right) => self.visit_binary_expr(left, right),
            ExprKind::Call(callee, _, args) => self.visit_call_expr(callee, args),
            ExprKind::Grouping(ex) => self.resolve_expr(ex),
            ExprKind::Literal(_) => Ok(()),
            ExprKind::Logical(left, _, right) => self.visit_binary_expr(left, right),
            ExprKind::Unary(_, right) => self.resolve_expr(right),
            ExprKind::Variable(id) => self.visit_var_expr(expr, id),
        }
    }

    fn visit_assign_expr(&mut self, ex: &Expr, id: &Ident, initializer: &Expr) -> ResolverResult {
        self.resolve_expr(initializer)?;
        self.resolve_local(ex, id);
        Ok(())
    }

    fn visit_binary_expr(&mut self, left: &Expr, right: &Expr) -> ResolverResult {
        self.resolve_expr(left)?;
        self.resolve_expr(right)?;
        Ok(())
    }

    fn visit_call_expr(&mut self, callee: &Expr, args: &Vec<Expr>) -> ResolverResult {
        self.resolve_expr(callee)?;
        for arg in args {
            self.resolve_expr(arg)?;
        }
        Ok(())
    }

    fn visit_var_expr(&mut self, ex: &Expr, id: &Ident) -> ResolverResult {
        if let Some(initialized) = self.scopes.last_mut().and_then(|s| s.get(&id.symbol)) {
            if !initialized {
                self.report_error(
                    (id.span, "Can't read local variable in its own initializer.").into(),
                );
            }
        }

        self.resolve_local(ex, id);
        Ok(())
    }

    fn resolve_local(&mut self, ex: &Expr, id: &Ident) {
        for i in (0..self.scopes.len()).rev() {
            if self
                .scopes
                .get(i)
                .is_some_and(|s| s.contains_key(&id.symbol))
            {
                self.interpreter.resolve(ex, self.scopes.len() - 1 - i);
                return;
            }
        }
    }

    fn declare(&mut self, id: &Ident) -> ResolverResult {
        let Some(scope) = self.scopes.last_mut() else {
            return Ok(());
        };
        if scope.contains_key(&id.symbol) {
            return Err((id.span, "Already a variable with this name in this scope.").into());
        }
        scope.insert(id.symbol.to_owned(), false);
        Ok(())
    }

    fn define(&mut self, id: &Ident) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(id.symbol.to_owned(), true);
        };
    }

    fn begin_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn report_error(&mut self, e: SpannedError) {
        self.errors.push(e)
    }
}
