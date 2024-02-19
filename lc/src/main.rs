use std::{
    env,
    fs::File,
    io::{self, Read, Write},
    path::Path,
    process::ExitCode,
};

use anyhow::Result;

use lc_core::*;
use lc_interpreter::*;

fn run(input: String, context: &mut Interpreter) -> Result<()> {
    let mut issues = TranslationErrors::new();

    // Lexing
    let mut scanner = Scanner::new(input);
    let (tokens, mut errs) = scanner.scan_tokens();
    issues.merge(&mut errs);

    // Parsing
    let mut parser = Parser::new(tokens);
    let (statements, mut errs) = parser.parse();
    issues.merge(&mut errs);

    // Resolving and binding
    let mut resolver = Resolver::new(context);
    let (_, mut errs) = resolver.resolve(&statements);
    issues.merge(&mut errs);

    // Execution
    issues.check()?;
    context.interpret(statements)?;
    Ok(())
}

fn run_file(filename: String) -> Result<()> {
    let path = Path::new(filename.as_str());
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let output = &mut io::stdout();
    run(contents, &mut Interpreter::new(output))
}

fn run_prompt() -> Result<()> {
    let output = &mut io::stdout();
    let mut context = Interpreter::new(output);
    loop {
        let mut buffer = String::new();
        print!("> ");
        io::stdout().flush()?;
        let input_size = io::stdin().read_line(&mut buffer)?;
        if input_size == 0 {
            // Windows: Ctrl+Z, Unix: Ctrl+D
            return Ok(());
        }
        if let Err(e) = run(buffer, &mut context) {
            eprint!("{}", e);
        }
    }
}

fn main() -> ExitCode {
    if env::args().len() > 2 {
        eprintln!("Usage: mylang [script]");
        return ExitCode::FAILURE;
    }
    let result = if env::args().len() == 2 {
        run_file(env::args().nth(1).unwrap())
    } else {
        run_prompt()
    };
    if let Err(e) = result {
        eprint!("{e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
