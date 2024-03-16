use leas::prelude::*;
use rt::Eval;

fn open<T>(result: sys::Result<T>) -> T {
    match result {
        Ok(v) => v,
        Err(err) => {
            println!("{}", err);
            panic!("Error occurred.")
        }
    }
}

fn main() {
    let content = std::fs::read_to_string("tests/main.lea").unwrap();
    let comp = comp::Compilable::new(&content);
    let stream = open(comp.lex());
    //println!("Output stream: {:?}", stream.0);
    let sliced = stream.slice();
    //println!("Output slice: {:?}", sliced.0);
    let cooked = open(sliced.cook());
    //println!("Output cooked: {:?}", cooked.0);
    let mut map = sys::Map::new();
    map.register("print", |map, v| {
        println!("{:?}", v.upgrade().unwrap());
        Ok(map.req("self").unwrap().clone())
    });
    leas::modules::bool::init(&mut map);
    leas::modules::float::init(&mut map);
    leas::modules::int::init(&mut map);
    leas::modules::sys::init(&mut map);
    leas::modules::uint::init(&mut map);
    leas::modules::vec::init(&mut map);

    let result = open(cooked.0.eval(&mut map));
    println!("Output result: {:?}", result);
}
