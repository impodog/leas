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

fn compile(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::single("str");
    matcher.mat_or_err(arg, map.line())?;

    let a = matcher.to_single().unwrap();

    let line = map.line();
    let result = a.visit_res_or_else(
        |s: &String| {
            let stmt = Compilable::new(s)
                .compile()
                .map_err(|err| err.with(format!("When compiling:\n{}", s)))?;
            let body = Rc::new(stmt);
            Stmt::to_fn(map, &body)
                .map_err(|err| err.with(format!("When making function with:\n{}", s)))
        },
        move || Error::new("Non-string value cannot be compiled", line),
    )??;

    Ok(result)
}

fn init_module(map: &mut Map) {
    map.register("same", same);
    map.register("is_stop", is_stop);
    map.register("compile", compile);
}

pub fn init(map: &mut Map) {
    map.register_init("sys", init_module);
}
