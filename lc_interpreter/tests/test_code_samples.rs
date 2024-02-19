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
fn block_scope() -> Result<()> {
    let source = "\
let x = \"outside\";
{
    let x = \"first\";
    print x;
}
{
    let x = \"second\";
    print x;
}
print x;
    ";
    let mut output: Vec<u8> = Vec::new();
    execute_sample(source, &mut output)?;
    let expect = "\
first
second
outside
"
    .as_bytes()
    .to_vec();
    assert_eq!(output, expect);
    Ok(())
}

#[test]
fn mixed_scope() -> Result<()> {
    let source = "\
let x = \"outside\";
{
    let y = \"inside\";
    print x + y;
}
    ";
    let mut output: Vec<u8> = Vec::new();
    execute_sample(source, &mut output)?;
    let expect = "\
outsideinside
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
fn shadowing() -> Result<()> {
    let source = "\
let a = \"global a\";
let b = \"global b\";
let c = \"global c\";
{
    let a = \"outer a\";
    let b = \"outer b\";
    {
        let a = \"inner a\";
        print a;
        print b;
        print c;
    }
    print a;
    print b;
    print c;
}
print a;
print b;
print c;
    ";
    let mut output: Vec<u8> = Vec::new();
    execute_sample(source, &mut output)?;
    let expect = "\
inner a
outer b
global c
outer a
outer b
global c
global a
global b
global c
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
fn redefine_var() -> Result<()> {
    let source = "\
let x = \"before\";
print x;
let x = \"after\";
print x;
    ";
    let mut output: Vec<u8> = Vec::new();
    execute_sample(source, &mut output)?;
    let expect = "\
before
after
"
    .as_bytes()
    .to_vec();
    assert_eq!(output, expect);
    Ok(())
}

#[test]
fn evaluate_ver_expr() -> Result<()> {
    let source = "\
let x = 1;
let y = 2;
print x + y;
    ";
    let mut output: Vec<u8> = Vec::new();
    execute_sample(source, &mut output)?;
    let expect = "\
3
"
    .as_bytes()
    .to_vec();
    assert_eq!(output, expect);
    Ok(())
}

#[test]
fn single_recursion() -> Result<()> {
    let source = "\
fn toZero(n) {
    if (n <= 0) return;
    print n;
    toZero(n-1);
}
toZero(5);
    ";
    let mut output: Vec<u8> = Vec::new();
    execute_sample(source, &mut output)?;
    let expect = "\
5
4
3
2
1
"
    .as_bytes()
    .to_vec();
    assert_eq!(output, expect);
    Ok(())
}

#[test]
fn mutual_recursion() -> Result<()> {
    let source = "\
fn isOdd(n) {
    if (n == 0) return false;
    return isEven(n-1);
}

fn isEven(n) {
    if (n == 0) return true;
    return isOdd(n-1);
}

print isEven(2);
print isEven(5.0);
print isOdd(3);
print isOdd(2);
    ";
    let mut output: Vec<u8> = Vec::new();
    execute_sample(source, &mut output)?;
    let expect = "\
true
false
true
false
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
}

#[test]
#[should_panic]
fn self_initializer() {
    let source = "\
let a = a;
    ";
    let mut output: Vec<u8> = Vec::new();
    let _ = execute_sample(source, &mut output).unwrap();
}

#[test]
#[should_panic]
fn top_level_return() {
    let source = "\
return;
    ";
    let mut output: Vec<u8> = Vec::new();
    let _ = execute_sample(source, &mut output).unwrap();
}

#[test]
#[should_panic]
fn uncallable_identifers_string() {
    let source = "\
let x = \"not_a_function\"();
print x;
    ";
    let mut output: Vec<u8> = Vec::new();
    let _ = execute_sample(source, &mut output).unwrap();
}

#[test]
#[should_panic]
fn uncallable_identifers_number() {
    let source = "\
let x = 14+0.001();
print x
    ";
    let mut output: Vec<u8> = Vec::new();
    let _ = execute_sample(source, &mut output).unwrap();
}

#[test]
#[should_panic]
fn uncallable_identifers_bool() {
    let source = "\
let x = false();
    ";
    let mut output: Vec<u8> = Vec::new();
    let _ = execute_sample(source, &mut output).unwrap();
}

#[test]
#[should_panic]
fn uncallable_identifers_null() {
    let source = "\
let x = null();
    ";
    let mut output: Vec<u8> = Vec::new();
    let _ = execute_sample(source, &mut output).unwrap();
}

#[test]
#[should_panic]
fn use_keyword_as_identifier() {
    let source = "\
fn let() {
    print \"test\";
}
let();
    ";
    let mut output: Vec<u8> = Vec::new();
    let _ = execute_sample(source, &mut output).unwrap();
}

#[test]
#[should_panic]
fn use_token_as_identifier() {
    let source = "\
fn *() {
    print \"test\";
}
*();
    ";
    let mut output: Vec<u8> = Vec::new();
    let _ = execute_sample(source, &mut output).unwrap();
}

#[test]
#[should_panic]
fn invalid_binary_ops() {
    let source = "\
let a = + 5;
    ";
    let mut output: Vec<u8> = Vec::new();
    let _ = execute_sample(source, &mut output).unwrap();
}

#[test]
#[should_panic]
fn invalid_unary_op() {
    let source = "\
let !a = -!5;
    ";
    let mut output: Vec<u8> = Vec::new();
    let _ = execute_sample(source, &mut output).unwrap();
}

#[test]
#[should_panic]
fn var_decl_as_if_body() {
    let source = "\
let x = 1;
if (x == 1) var y = 2;
    ";
    let mut output: Vec<u8> = Vec::new();
    let _ = execute_sample(source, &mut output).unwrap();
}
