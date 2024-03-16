use crate::prelude::*;

fn add(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::listed().with_singles(&["a", "b"]);
    matcher.mat_or_err(arg, map.line())?;
    let mut list = matcher.to_values().unwrap();

    let a = list.pop_front().unwrap();
    let b = list.pop_front().unwrap();

    a.as_float()
        .and_then(|a| b.as_float().map(|b| a + b))
        .map(Value::Float)
        .ok_or_else(move || Error::new("Arguments a and b are not floats", map.line()))
}

fn sub(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::listed().with_singles(&["a", "b"]);
    matcher.mat_or_err(arg, map.line())?;
    let mut list = matcher.to_values().unwrap();

    let a = list.pop_front().unwrap();
    let b = list.pop_front().unwrap();

    a.as_float()
        .and_then(|a| b.as_float().map(|b| a - b))
        .map(Value::Float)
        .ok_or_else(move || Error::new("Arguments a and b are not floats", map.line()))
}

fn mul(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::listed().with_singles(&["a", "b"]);
    matcher.mat_or_err(arg, map.line())?;
    let mut list = matcher.to_values().unwrap();

    let a = list.pop_front().unwrap();
    let b = list.pop_front().unwrap();

    a.as_float()
        .and_then(|a| b.as_float().map(|b| a * b))
        .map(Value::Float)
        .ok_or_else(move || Error::new("Arguments a and b are not floats", map.line()))
}

fn div(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::listed().with_singles(&["a", "b"]);
    matcher.mat_or_err(arg, map.line())?;
    let mut list = matcher.to_values().unwrap();

    let a = list.pop_front().unwrap();
    let b = list.pop_front().unwrap();

    a.as_float()
        .and_then(|a| b.as_float().map(|b| a / b))
        .map(Value::Float)
        .ok_or_else(move || Error::new("Arguments a and b are not floats", map.line()))
}

fn eq(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::listed().with_singles(&["a", "b"]);
    matcher.mat_or_err(arg, map.line())?;
    let mut list = matcher.to_values().unwrap();

    let a = list.pop_front().unwrap();
    let b = list.pop_front().unwrap();

    a.as_float()
        .and_then(|a| b.as_float().map(|b| a == b))
        .map(Value::Bool)
        .ok_or_else(move || Error::new("Arguments a and b are not floats", map.line()))
}

fn lt(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::listed().with_singles(&["a", "b"]);
    matcher.mat_or_err(arg, map.line())?;
    let mut list = matcher.to_values().unwrap();

    let a = list.pop_front().unwrap();
    let b = list.pop_front().unwrap();

    a.as_float()
        .and_then(|a| b.as_float().map(|b| a < b))
        .map(Value::Bool)
        .ok_or_else(move || Error::new("Arguments a and b are not floats", map.line()))
}

fn to_int(map: &mut Map, arg: Value) -> Result<Value> {
    arg.as_float()
        .map(|n| Value::Int(n as Int))
        .ok_or_else(|| Error::new("Argument is not a float", map.line()))
}

fn to_uint(map: &mut Map, arg: Value) -> Result<Value> {
    arg.as_float()
        .map(|n| Value::Uint(n as Uint))
        .ok_or_else(|| Error::new("Argument is not a float", map.line()))
}

fn init_module(map: &mut Map) {
    map.register("add", add);
    map.register("sub", sub);
    map.register("mul", mul);
    map.register("div", div);
    map.register("eq", eq);
    map.register("lt", lt);
    map.register("to_int", to_int);
    map.register("to_uint", to_uint);
}

pub fn init(map: &mut Map) {
    map.register_init("float", init_module);
}
