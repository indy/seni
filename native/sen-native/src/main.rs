use sen_core::*;

fn main() {
    println!("Hello, world!");

    let s = "hello world".to_string();
    println!("the size of {} is {}", s, sen_parse(&s));

    let program = Compiler::compile_preamble().unwrap();
//    println!("{:?}", program);

    println!("{}", program);
}
