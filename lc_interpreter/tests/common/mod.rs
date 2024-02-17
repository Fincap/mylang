use anyhow::{anyhow, Result};
use lc_core::*;
use lc_interpreter::*;

pub fn execute_sample(source: &str, output: &mut Vec<u8>) -> Result<()> {
    let mut context = Interpreter::new(output);
    let mut scanner = Scanner::new(source.to_string());
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse();
    dbg!(&statements);
    let mut resolver = Resolver::new(&mut context);
    resolver.resolve(&statements)?;
    dbg!(&resolver);
    if resolver.had_error() {
        return Err(anyhow!("Failed to resolve variables"));
    };
    context.interpret(statements)
}
