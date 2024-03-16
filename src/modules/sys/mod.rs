use crate::prelude::*;

fn same(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::listed().with_singles(&["a", "b"]);
    matcher.mat_or_err(arg, map.line())?;
    let mut list = matcher.to_values().unwrap();

    let a = list.pop_front().unwrap();
    let b = list.pop_front().unwrap();

    Ok(Value::Bool(a == b))
}

fn is_stop(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::single("a");
    matcher.mat_or_err(arg, map.line())?;

    let a = matcher.to_single().unwrap();

    Ok(Value::Bool(a == Value::Stop))
}

fn init_module(map: &mut Map) {
    map.register("same", same);
    map.register("is_stop", is_stop);
}

pub fn init(map: &mut Map) {
    map.register_init("sys", init_module);
}
