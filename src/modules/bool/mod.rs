use crate::prelude::*;

fn not(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::single("a");
    matcher.mat_or_err(arg, map.line())?;
    let a = matcher.to_single().unwrap();

    a.as_bool()
        .map(|a| !a)
        .map(Value::Bool)
        .ok_or_else(move || Error::new("Argument a is not a boolean", map.line()))
}

fn and(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::listed().with_singles(&["a", "b"]);
    matcher.mat_or_err(arg, map.line())?;
    let mut list = matcher.to_values().unwrap();

    let a = list.pop_front().unwrap();
    let b = list.pop_front().unwrap();

    a.as_bool()
        .and_then(|a| b.as_bool().map(|b| a && b))
        .map(Value::Bool)
        .ok_or_else(move || Error::new("Arguments a and b are not booleans", map.line()))
}

fn or(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::listed().with_singles(&["a", "b"]);
    matcher.mat_or_err(arg, map.line())?;
    let mut list = matcher.to_values().unwrap();

    let a = list.pop_front().unwrap();
    let b = list.pop_front().unwrap();

    a.as_bool()
        .and_then(|a| b.as_bool().map(|b| a || b))
        .map(Value::Bool)
        .ok_or_else(move || Error::new("Arguments a and b are not booleans", map.line()))
}

fn xor(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::listed().with_singles(&["a", "b"]);
    matcher.mat_or_err(arg, map.line())?;
    let mut list = matcher.to_values().unwrap();

    let a = list.pop_front().unwrap();
    let b = list.pop_front().unwrap();

    a.as_bool()
        .and_then(|a| b.as_bool().map(|b| a ^ b))
        .map(Value::Bool)
        .ok_or_else(move || Error::new("Arguments a and b are not booleans", map.line()))
}

fn init_module(map: &mut Map) {
    map.register("not", not);
    map.register("and", and);
    map.register("or", or);
    map.register("xor", xor);
}

pub fn init(map: &mut Map) {
    map.register_init("bool", init_module);
}
