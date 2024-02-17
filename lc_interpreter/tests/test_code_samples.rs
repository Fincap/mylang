mod common;

use anyhow::Result;
use common::execute_sample;

#[test]
fn closure_scope() -> Result<()> {
    let source = "\
    let a = \"global\";
    {
      fn showA() {
        print a;
      }
    
      showA();
      let a = \"block\";
      showA();
    }";
    let mut output: Vec<u8> = Vec::new();
    execute_sample(source, &mut output)?;
    dbg!(&output);
    let expect = "\
global
global
"
    .as_bytes()
    .to_vec();
    assert_eq!(output, expect);
    Ok(())
}

#[test]
fn while_loop() -> Result<()> {
    let source = "\
    let x = 0;
    while (x < 5) {
        print x;
        x++;
    }
    ";
    let mut output: Vec<u8> = Vec::new();
    execute_sample(source, &mut output)?;
    dbg!(&output);
    let expect = "\
0
1
2
3
4
"
    .as_bytes()
    .to_vec();
    assert_eq!(output, expect);
    Ok(())
}

#[test]
fn for_loop() -> Result<()> {
    let source = "\
    for (let x = 0; x < 5; x++) {
        print x;
    }
    ";
    let mut output: Vec<u8> = Vec::new();
    execute_sample(source, &mut output)?;
    dbg!(&output);
    let expect = "\
0
1
2
3
4
"
    .as_bytes()
    .to_vec();
    assert_eq!(output, expect);
    Ok(())
}

#[test]
#[should_panic]
fn undefined_variable() {
    let source = "\
    print a;
    ";
    let mut output: Vec<u8> = Vec::new();
    execute_sample(source, &mut output).unwrap();
    dbg!(&output);
}

#[test]
#[should_panic]
fn self_initializer() {
    let source = "\
    let a = a;
    ";
    let mut output: Vec<u8> = Vec::new();
    let _ = execute_sample(source, &mut output).unwrap();
    dbg!(&output);
}

#[test]
#[should_panic]
fn top_level_return() {
    let source = "\
    return;
    ";
    let mut output: Vec<u8> = Vec::new();
    let _ = execute_sample(source, &mut output).unwrap();
    dbg!(&output);
}
