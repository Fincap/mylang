use std::{
    env,
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

use anyhow::{anyhow, Result};

use lc_core::*;
use lc_interpreter::*;

fn run(input: String, context: &mut Interpreter) -> Result<()> {
    let mut scanner = Scanner::new(input);
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse()?;
    //dbg!(&statements);
    let mut resolver = Resolver::new(context);
    resolver.resolve(&statements)?;
    //dbg!(&resolver);
    if resolver.had_error() {
        return Err(anyhow!("Failed to resolve variables"));
    };
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
        run(buffer, &mut context)?;
    }
}

fn main() -> Result<()> {
    if env::args().len() > 2 {
        return Err(anyhow!("Usage: mylang [script]"));
    }
    if env::args().len() == 2 {
        return run_file(env::args().nth(1).unwrap());
    }

    run_prompt()
}
