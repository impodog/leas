use leas::prelude::*;

fn open<T>(result: sys::Result<T>) -> T {
    match result {
        Ok(v) => v,
        Err(err) => {
            println!("{}", err);
            panic!()
        }
    }
}

fn main() {
    let content = std::fs::read_to_string("tests/main.lea").unwrap();
    let comp = comp::Compilable::new(&content);
    let stream = open(comp.lex());
    println!("Output stream: {:?}", stream.0);
    let sliced = stream.slice();
    println!("Output slice: {:?}", sliced.0);
    let cooked = open(sliced.cook());
    println!("Output cooked: {:?}", cooked.0);
}
