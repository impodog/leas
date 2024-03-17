use crate::prelude::*;

fn empty(_: &mut Map, _: Value) -> Result<Value> {
    Ok(Value::Res(Resource::new(String::new())))
}

fn push(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::listed().with_singles(&["str", "arg"]);
    matcher.mat_or_err(arg, map.line())?;
    let mut list = matcher.to_values().unwrap();

    let s = list.pop_front().unwrap();
    let arg = list.pop_front().unwrap();

    let line = map.line();
    let str = arg.visit_res_or_else(
        |arg: &String| arg.clone(),
        move || Error::new("Argument arg is not a string", line),
    )?;
    s.visit_mut_res_or_else(
        move |s: &mut String| {
            s.push_str(&str);
        },
        move || Error::new("Argument str is not a string", line),
    )?;

    Ok(Value::Null)
}

fn pop(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher: Matcher = Matcher::single("str");
    matcher.mat_or_err(arg, map.line())?;

    let s = matcher.to_single().unwrap();

    let line = map.line();
    let result = s
        .visit_mut_res_or_else(
            |s: &mut String| s.pop().map(|c| Value::Uint(c as Uint)),
            move || Error::new("Argument str is not a string", line),
        )?
        .unwrap_or(Value::Stop);

    Ok(result)
}

fn get(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::listed().with_singles(&["str", "index"]);
    matcher.mat_or_err(arg, map.line())?;
    let mut list = matcher.to_values().unwrap();

    let s = list.pop_front().unwrap();
    let i = list.pop_front().unwrap();

    let line = map.line();
    let result = s
        .visit_res_or_else(
            |s: &String| {
                let i = i.as_uint()?;
                let c = s.chars().nth(i as usize)?;
                Some(Value::Uint(c as Uint))
            },
            move || Error::new("Argument str is not a string", line),
        )?
        .unwrap_or(Value::Stop);

    Ok(result)
}

fn set(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::listed().with_singles(&["str", "index", "uint"]);
    matcher.mat_or_err(arg, map.line())?;
    let mut list = matcher.to_values().unwrap();

    let s = list.pop_front().unwrap();
    let i = list.pop_front().unwrap();
    let c = list.pop_front().unwrap();

    let line = map.line();
    s.visit_mut_res_or_else(
        move |s: &mut String| {
            let i = i.as_uint()?;
            let c = char::from_u32(c.as_uint()? as u32)?;
            let mut chars = s.chars().collect::<Vec<_>>();
            chars[i as usize] = c;
            *s = chars.into_iter().collect();
            Some(())
        },
        move || Error::new("Argument str is not a string", line),
    )?
    .map_or_else(|| Ok(Value::Stop), |_| Ok(Value::Null))
}

fn clone(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher: Matcher = Matcher::single("str");
    matcher.mat_or_err(arg, map.line())?;

    let s = matcher.to_single().unwrap();

    let line = map.line();
    let result = s
        .visit_res_or_else(
            |s: &String| Some(Value::Res(Resource::new(s.clone()))),
            move || Error::new("Argument str is not a string", line),
        )?
        .unwrap_or(Value::Stop);

    Ok(result)
}

fn length(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher: Matcher = Matcher::single("str");
    matcher.mat_or_err(arg, map.line())?;

    let s = matcher.to_single().unwrap();

    let line = map.line();
    let result = s
        .visit_res_or_else(
            |s: &String| Some(Value::Uint(s.len() as Uint)),
            move || Error::new("Argument str is not a string", line),
        )?
        .unwrap_or(Value::Stop);

    Ok(result)
}

fn to_chars(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher: Matcher = Matcher::single("str");
    matcher.mat_or_err(arg, map.line())?;

    let s = matcher.to_single().unwrap();

    let line = map.line();
    let result = s
        .visit_res_or_else(
            |s: &String| {
                let mut vec = VecDeque::new();
                for c in s.chars() {
                    vec.push_back(Value::Uint(c as Uint));
                }
                Some(Value::Res(Resource::new(vec)))
            },
            move || Error::new("Argument str is not a string", line),
        )?
        .unwrap_or(Value::Stop);

    Ok(result)
}

fn from_chars(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::listed().with_singles(&["vec"]);
    matcher.mat_or_err(arg, map.line())?;
    let mut list = matcher.to_values().unwrap();

    let vec = list.pop_front().unwrap();

    let line = map.line();
    let result = vec
        .visit_res_or_else(
            |vec: &VecDeque<Value>| {
                let mut s = String::new();
                for c in vec {
                    s.push(char::from_u32(c.as_uint()? as u32)?);
                }
                Some(Value::Res(Resource::new(s)))
            },
            move || Error::new("Argument vec is not a vector", line),
        )?
        .unwrap_or(Value::Stop);

    Ok(result)
}

fn from_char(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher: Matcher = Matcher::single("uint");
    matcher.mat_or_err(arg, map.line())?;

    let c = matcher.to_single().unwrap();

    let line = map.line();
    let result = c
        .visit_res_or_else(
            |c: &Uint| {
                let c = char::from_u32(*c as u32)?;
                Some(Value::Res(Resource::new(c.to_string())))
            },
            move || Error::new("Argument uint is not a uint", line),
        )?
        .unwrap_or(Value::Stop);

    Ok(result)
}

fn print(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher: Matcher = Matcher::single("str");
    matcher.mat_or_err(arg, map.line())?;

    let s = matcher.to_single().unwrap();

    let line = map.line();
    s.visit_res_or_else(
        |s: &String| {
            print!("{}", s);
            Some(Value::Null)
        },
        move || Error::new("Argument str is not a string", line),
    )?;

    map.req("self")
}

fn from(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher: Matcher = Matcher::single("str");
    matcher.mat_or_err(arg, map.line())?;

    let s = matcher.to_single().unwrap();

    Ok(Value::Res(Resource::new(s.to_string())))
}

fn init_module(map: &mut Map) {
    map.register("empty", empty);
    map.register("push", push);
    map.register("pop", pop);
    map.register("get", get);
    map.register("set", set);
    map.register("clone", clone);
    map.register("length", length);
    map.register("to_chars", to_chars);
    map.register("from_chars", from_chars);
    map.register("from_char", from_char);
    map.register("print", print);
    map.register("from", from);
}

pub fn init(map: &mut Map) {
    map.register_init("str", init_module);
}
