use crate::{
    error::parser_error,
    expr::{ExprKind, LIMIT_FN_ARGS},
    stmt::Stmt,
    token::{
        Token, TokenError,
        TokenType::{self, *},
    },
    Expr,
};

type ExprResult = Result<Expr, TokenError>;
type StmtResult = Result<Stmt, TokenError>;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if let Some(statement) = self.declaration() {
                statements.push(statement);
            }
        }
        statements
    }

    fn declaration(&mut self) -> Option<Stmt> {
        let stmt = match self.peek().t_type {
            Let => self.var_declaration(),
            Fn => self.fn_declaration(),
            _ => self.statement(),
        };
        // Handle errors at statement-level
        match stmt {
            Ok(stmt) => Some(stmt),
            Err(e) => {
                self.synchronize();
                parser_error(e);
                None
            }
        }
    }

    fn statement(&mut self) -> StmtResult {
        match self.peek().t_type {
            LeftBrace => self.block(),
            Return => self.return_stmt(),
            Print => self.print_stmt(),
            If => self.if_stmt(),
            While => self.while_stmt(),
            For => self.for_stmt(),
            _ => self.expr_stmt(),
        }
    }

    fn expr_stmt(&mut self) -> StmtResult {
        let ex = self.expression()?;
        self.consume(Semicolon, "Expected ';' after expression.")?;
        Ok(Stmt::Expression(ex))
    }

    fn block(&mut self) -> StmtResult {
        self.advance();
        let mut statements = Vec::new();
        while !self.check(&RightBrace) && !self.is_at_end() {
            if let Some(statement) = self.declaration() {
                statements.push(statement);
            }
        }
        self.consume(RightBrace, "Excepted '}' after block.")?;
        let block = Stmt::Block(statements);
        Ok(block)
    }

    fn return_stmt(&mut self) -> StmtResult {
        self.advance();
        let value = if !self.check(&Semicolon) {
            self.expression()?
        } else {
            Expr::literal_null()
        };
        self.consume(Semicolon, "Expected ';' after return value.")?;
        Ok(Stmt::Return(value))
    }

    fn print_stmt(&mut self) -> StmtResult {
        self.advance();
        let ex = self.expression()?;
        self.consume(Semicolon, "Expected ';' after value.")?;
        Ok(Stmt::Print(ex))
    }

    fn if_stmt(&mut self) -> StmtResult {
        self.advance();
        self.consume(LeftParen, "Expected '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(RightParen, "Expected ')' after if condition.")?;

        let then_branch = self.statement()?;
        let else_branch = if self.match_next(vec![Else]) {
            Some(self.statement()?)
        } else {
            None
        };
        Ok(Stmt::new_if(condition, then_branch, else_branch))
    }

    fn while_stmt(&mut self) -> StmtResult {
        self.advance();
        self.consume(LeftParen, "Expected '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(RightParen, "Expected ')' after while condition.")?;
        let body = self.statement()?;
        Ok(Stmt::new_while(condition, body))
    }

    fn for_stmt(&mut self) -> StmtResult {
        self.advance();
        self.consume(LeftParen, "Expected '(' after 'for'.")?;
        let initializer = match self.peek().t_type {
            Semicolon => {
                self.advance();
                None
            }
            Let => Some(self.var_declaration()?),
            _ => Some(self.expr_stmt()?),
        };

        let condition = if !self.check(&Semicolon) {
            self.expression()?
        } else {
            Expr::literal_bool(true)
        };
        self.consume(Semicolon, "Expected ';' after loop condition.")?;

        let increment = if !self.check(&RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(RightParen, "Expected ')' after for clauses.")?;

        let mut body = self.statement()?;
        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(increment)]);
        }
        body = Stmt::new_while(condition, body);
        if let Some(initializer) = initializer {
            body = Stmt::Block(vec![initializer, body]);
        }

        Ok(body)
    }

    fn var_declaration(&mut self) -> StmtResult {
        self.advance();
        let name = self.consume(Identifier, "Expected variable name.")?;
        let mut initializer = Expr::literal_null();
        if self.match_next(vec![Equal]) {
            initializer = self.expression()?;
        }
        self.consume(Semicolon, "Expect ';' after variable declaration")?;
        Ok(Stmt::Let(name, initializer))
    }

    fn fn_declaration(&mut self) -> StmtResult {
        self.advance();
        let name = self.consume(Identifier, "Expected function name.")?;
        self.consume(LeftParen, "Expected '(' after function name.")?;
        let mut parameters = Vec::new();
        if !self.check(&RightParen) {
            loop {
                if parameters.len() >= LIMIT_FN_ARGS {
                    parser_error(
                        (
                            &self.peek(),
                            format!("Can't have more than {} parameters.", LIMIT_FN_ARGS),
                        )
                            .into(),
                    )
                }
                parameters.push(self.consume(Identifier, "Expected parameter name.")?);
                if !self.match_next(vec![Comma]) {
                    break;
                }
            }
        }
        self.consume(RightParen, "Expected ')' after parameters.")?;
        if !self.check(&LeftBrace) {
            return Err((&self.peek(), "Expected '{' before function body.").into());
        }
        let Stmt::Block(body) = self.block()? else {
            return Err((&self.peek(), "Incomplete function body.").into());
        };
        Ok(Stmt::Function(name, parameters, body))
    }

    fn expression(&mut self) -> ExprResult {
        self.assignment()
    }

    fn assignment(&mut self) -> ExprResult {
        let ex = self.compound_assign()?;
        if self.match_next(vec![Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let ExprKind::Variable(token) = ex.kind {
                return Ok(Expr::assign(token, value));
            }
            // Report error but don't throw because parser isn't in a confused state
            parser_error((&equals, "Invalid assignment target.").into());
        }
        Ok(ex)
    }

    fn compound_assign(&mut self) -> ExprResult {
        let ex = self.logic_or()?;
        if self.match_next(vec![PlusEqual, MinusEqual, StarEqual, SlashEqual]) {
            let op_assign = self.previous();
            let right = self.assignment()?;
            let mut op_arithmetic = op_assign.clone();
            op_arithmetic.t_type = match op_assign.t_type {
                PlusEqual => Plus,
                MinusEqual => Minus,
                StarEqual => Star,
                SlashEqual => Slash,
                _ => unreachable!(),
            };

            let right = Expr::binary(ex.to_owned(), op_arithmetic, right);
            if let ExprKind::Variable(op) = ex.kind {
                return Ok(Expr::assign(op, right));
            }

            parser_error((&op_assign, "Invalid assignment target.").into());
        }
        Ok(ex)
    }

    fn logic_or(&mut self) -> ExprResult {
        let mut ex = self.logic_and()?;
        while self.match_next(vec![Or]) {
            let op = self.previous();
            let right = self.logic_and()?;
            ex = Expr::logical(ex, op, right);
        }
        Ok(ex)
    }

    fn logic_and(&mut self) -> ExprResult {
        let mut ex = self.equality()?;
        while self.match_next(vec![And]) {
            let op = self.previous();
            let right = self.equality()?;
            ex = Expr::logical(ex, op, right);
        }
        Ok(ex)
    }

    fn equality(&mut self) -> ExprResult {
        let mut ex = self.comparison()?;
        while self.match_next(vec![BangEqual, EqualEqual]) {
            let op = self.previous();
            let right = self.comparison()?;
            ex = Expr::binary(ex, op, right);
        }
        Ok(ex)
    }

    fn comparison(&mut self) -> ExprResult {
        let mut ex = self.term()?;
        while self.match_next(vec![Greater, GreaterEqual, Less, LessEqual]) {
            let op = self.previous();
            let right = self.term()?;
            ex = Expr::binary(ex, op, right);
        }
        Ok(ex)
    }

    fn term(&mut self) -> ExprResult {
        let mut ex = self.factor()?;
        while self.match_next(vec![Minus, Plus]) {
            let op = self.previous();
            let right = self.factor()?;
            ex = Expr::binary(ex, op, right);
        }
        Ok(ex)
    }

    fn factor(&mut self) -> ExprResult {
        let mut ex = self.unary()?;
        while self.match_next(vec![Slash, Star]) {
            let op = self.previous();
            let right = self.unary()?;
            ex = Expr::binary(ex, op, right);
        }
        Ok(ex)
    }

    fn unary(&mut self) -> ExprResult {
        if self.match_next(vec![Bang, Minus]) {
            let op = self.previous();
            let ex = self.unary()?;
            return Ok(Expr::unary(op, ex));
        }
        self.inc_dec()
    }

    fn inc_dec(&mut self) -> ExprResult {
        let ex = self.call()?;
        if self.match_next(vec![PlusPlus, MinusMinus]) {
            let op = self.previous();
            let mut op_expanded = op.clone();
            op_expanded.t_type = match op.t_type {
                PlusPlus => Plus,
                MinusMinus => Minus,
                _ => unreachable!(),
            };
            let right = Expr::binary(
                ex.to_owned(),
                op_expanded.to_owned(),
                Expr::literal_number(1.0),
            );
            if let ExprKind::Variable(op) = ex.kind {
                return Ok(Expr::assign(op, right));
            }
            parser_error((&op_expanded, "Invalid increment/decrement target.").into());
        }
        Ok(ex)
    }

    fn call(&mut self) -> ExprResult {
        let mut ex = self.primary()?;
        loop {
            if self.match_next(vec![LeftParen]) {
                ex = self.finish_call(&ex)?;
            } else {
                break;
            }
        }
        Ok(ex)
    }

    fn finish_call(&mut self, ex: &Expr) -> ExprResult {
        let mut arguments = Vec::new();
        if !self.check(&RightParen) {
            loop {
                if arguments.len() >= LIMIT_FN_ARGS {
                    parser_error(
                        (
                            &self.peek(),
                            format!("Can't have more than {} arguments.", LIMIT_FN_ARGS),
                        )
                            .into(),
                    )
                }
                arguments.push(self.expression()?);
                if !self.match_next(vec![Comma]) {
                    break;
                }
            }
        }
        let paren = self.consume(RightParen, "Expected ')' after arguments.")?;
        Ok(Expr::call(ex.to_owned(), paren, arguments))
    }

    fn primary(&mut self) -> ExprResult {
        let token = self.peek();
        match token.t_type {
            False => {
                self.advance();
                Ok(Expr::literal_bool(false))
            }
            True => {
                self.advance();
                Ok(Expr::literal_bool(true))
            }
            Null => {
                self.advance();
                Ok(Expr::literal_null())
            }
            Number(num) => {
                self.advance();
                Ok(Expr::literal_number(num))
            }
            String(str) => {
                self.advance();
                Ok(Expr::literal_string(str))
            }
            LeftParen => {
                self.advance();
                let ex = self.expression()?;
                self.consume(RightParen, "Expected ')' after expression.")?;
                Ok(Expr::grouping(ex))
            }
            Identifier => {
                self.advance();
                Ok(Expr::var(token))
            }
            BangEqual | EqualEqual | Greater | GreaterEqual | Less | LessEqual | Plus | Slash
            | Star => {
                self.advance();
                Err((
                    &token,
                    format!("Binary operator '{}' missing operand(s)", token.lexeme),
                )
                    .into())
            }
            _ => Err((&token, "Expected expression.").into()),
        }
    }

    fn match_next(&mut self, types: Vec<TokenType>) -> bool {
        for t_type in &types {
            if self.check(t_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&mut self, t_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().t_type == *t_type
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().t_type == EOF
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].to_owned()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].to_owned()
    }

    fn consume(&mut self, t_type: TokenType, message: &'static str) -> Result<Token, TokenError> {
        if self.check(&t_type) {
            Ok(self.advance())
        } else {
            Err((&self.peek(), message.to_string()).into())
        }
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().t_type == Semicolon {
                return;
            }
            match self.peek().t_type {
                Class | Fn | Let | For | If | While | Print | Return => {
                    return;
                }
                _ => (),
            }
            self.advance();
        }
    }
}
