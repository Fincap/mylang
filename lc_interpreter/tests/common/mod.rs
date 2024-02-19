use anyhow::Result;
use lc_core::*;
use lc_interpreter::*;

pub fn execute_sample(source: &str, output: &mut Vec<u8>) -> Result<()> {
    let mut context = Interpreter::new(output);
    let mut issues = TranslationErrors::new();

    // Lexing
    let mut scanner = Scanner::new(source.to_string());
    let (tokens, mut errs) = scanner.scan_tokens();
    issues.merge(&mut errs);

    // Parsing
    let mut parser = Parser::new(tokens);
    let (statements, mut errs) = parser.parse();
    issues.merge(&mut errs);

    // Resolving and binding
    let mut resolver = Resolver::new(&mut context);
    let (_, mut errs) = resolver.resolve(&statements);
    issues.merge(&mut errs);

    // Execution
    issues.check()?;
    context.interpret(statements)?;
    // dbg!(String::from_utf8_lossy(output));
    Ok(())
}
