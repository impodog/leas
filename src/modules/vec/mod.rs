use crate::prelude::*;

fn empty(_: &mut Map, _: Value) -> Result<Value> {
    Ok(Value::Res(Resource::new(VecDeque::<Value>::new())))
}

fn push_back(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::listed().with_singles(&["vec", "arg"]);
    matcher.mat_or_err(arg, map.line())?;
    let mut list = matcher.to_values().unwrap();

    let vec = list.pop_front().unwrap();
    let arg = list.pop_front().unwrap();

    let line = map.line();
    vec.visit_mut_res_or_else(
        |vec: &mut VecDeque<Value>| {
            vec.push_back(arg);
        },
        move || Error::new("Argument vec is not a vector", line),
    )?;

    Ok(Value::Null)
}

fn push_front(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::listed().with_singles(&["vec", "arg"]);
    matcher.mat_or_err(arg, map.line())?;
    let mut list = matcher.to_values().unwrap();

    let vec = list.pop_front().unwrap();
    let arg = list.pop_front().unwrap();

    let line = map.line();
    vec.visit_mut_res_or_else(
        |vec: &mut VecDeque<Value>| {
            vec.push_front(arg);
        },
        move || Error::new("Argument vec is not a vector", line),
    )?;

    Ok(Value::Null)
}

fn pop_back(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher: Matcher = Matcher::single("vec");
    matcher.mat_or_err(arg, map.line())?;

    let vec = matcher.to_single().unwrap();

    let line = map.line();
    let result = vec
        .visit_mut_res_or_else(
            |vec: &mut VecDeque<Value>| vec.pop_back(),
            move || Error::new("Argument vec is not a vector", line),
        )?
        .unwrap_or(Value::Stop);

    Ok(result)
}

fn pop_front(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher: Matcher = Matcher::single("vec");
    matcher.mat_or_err(arg, map.line())?;

    let vec = matcher.to_single().unwrap();

    let line = map.line();
    let result = vec
        .visit_mut_res_or_else(
            |vec: &mut VecDeque<Value>| vec.pop_front(),
            move || Error::new("Argument vec is not a vector", line),
        )?
        .unwrap_or(Value::Stop);

    Ok(result)
}

fn get(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::listed().with_singles(&["vec", "index"]);
    matcher.mat_or_err(arg, map.line())?;
    let mut list = matcher.to_values().unwrap();

    let vec = list.pop_front().unwrap();
    let index = list.pop_front().unwrap();

    let line = map.line();
    let index = index
        .as_uint()
        .ok_or_else(|| Error::new("Argument index is not an unsigned integer", line))?
        as usize;
    let result = vec
        .visit_res_or_else(
            move |vec: &VecDeque<Value>| vec.get(index).cloned().unwrap_or(Value::Stop),
            move || Error::new("Argument vec is not a vector", line),
        )?
        .clone();

    Ok(result)
}

fn set(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::listed().with_singles(&["vec", "index", "value"]);
    matcher.mat_or_err(arg, map.line())?;
    let mut list = matcher.to_values().unwrap();

    let vec = list.pop_front().unwrap();
    let index = list.pop_front().unwrap();
    let value = list.pop_front().unwrap();

    let line = map.line();
    let index = index
        .as_uint()
        .ok_or_else(|| Error::new("Argument index is not an unsigned integer", line))?
        as usize;
    vec.visit_mut_res_or_else(
        move |vec: &mut VecDeque<Value>| {
            if index > vec.len() {
                vec.resize(index + 1, Value::Stop);
            }
            vec[index] = value;
        },
        move || Error::new("Argument vec is not a vector", line),
    )?;

    Ok(Value::Null)
}

fn clone(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher: Matcher = Matcher::single("vec");
    matcher.mat_or_err(arg, map.line())?;

    let vec = matcher.to_single().unwrap();

    let line = map.line();
    let result = vec.visit_res_or_else(
        |vec: &VecDeque<Value>| Value::Res(Resource::new(vec.clone())),
        move || Error::new("Argument vec is not a vector", line),
    )?;

    Ok(result)
}

fn extend(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::listed().with_singles(&["vec", "extend"]);
    matcher.mat_or_err(arg, map.line())?;
    let mut list = matcher.to_values().unwrap();

    let vec = list.pop_front().unwrap();
    let extend = list.pop_front().unwrap();

    let line = map.line();
    let q = extend.visit_mut_res_or_else(
        |extend: &mut VecDeque<Value>| std::mem::take(extend),
        move || Error::new("Argument extend is not a vector", line),
    )?;
    let line = map.line();
    vec.visit_mut_res_or_else(
        move |vec: &mut VecDeque<Value>| vec.extend(q.into_iter()),
        move || Error::new("Argument vec is not a vector", line),
    )?;

    Ok(Value::Null)
}

fn length(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher: Matcher = Matcher::single("vec");
    matcher.mat_or_err(arg, map.line())?;

    let vec = matcher.to_single().unwrap();

    let line = map.line();
    let result = vec.visit_res_or_else(
        |vec: &VecDeque<Value>| Value::Uint(vec.len() as Uint),
        move || Error::new("Argument vec is not a vector", line),
    )?;

    Ok(result)
}

fn init_module(map: &mut Map) {
    map.register("empty", empty);
    map.register("push_back", push_back);
    map.register("push_front", push_front);
    map.register("pop_back", pop_back);
    map.register("pop_front", pop_front);
    map.register("get", get);
    map.register("set", set);
    map.register("clone", clone);
    map.register("extend", extend);
    map.register("length", length);
}

pub fn init(map: &mut Map) {
    map.register_init("vec", init_module);
}
