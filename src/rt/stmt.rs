use super::*;

impl Stmt {
    fn open_list<F>(map: &mut Map, left: &Self, right: &Self, mut f: F) -> Result<()>
    where
        F: FnMut(&mut Map, &Self) -> Result<()>,
    {
        match left {
            Self::Token(Token::Stop, line) => {
                map.set_line(*line);
                return Ok(());
            }
            _ => {
                f(map, left)?;
            }
        }
        match right {
            Self::List(left, right) => Self::open_list(map, left, right, f)?,
            Self::Token(Token::Stop, line) => {
                map.set_line(*line);
            }
            _ => f(map, right)?,
        }
        Ok(())
    }

    fn open_dot<F>(map: &mut Map, left: &Self, right: &Self, mut f: F) -> Result<()>
    where
        F: FnMut(&mut Map, &Self) -> Result<()>,
    {
        match left {
            Self::Token(Token::Stop, line) => {
                map.set_line(*line);
                return Ok(());
            }
            _ => {
                f(map, left)?;
            }
        }
        match right {
            Self::Dot(left, right) => Self::open_dot(map, left, right, f)?,
            Self::Token(Token::Stop, line) => {
                map.set_line(*line);
            }
            _ => f(map, right)?,
        }
        Ok(())
    }

    fn eval_colon(map: &mut Map, _left: &Self, _right: &Self) -> Result<Value> {
        Err(Error::new(
            "Colon operator should not be used here",
            map.line(),
        ))
    }

    fn eval_import(map: &mut Map, opd: &Self) -> Result<Value> {
        let mut stack = Vec::new();
        let name = if let Stmt::Dot(left, right) = opd {
            let mut result = String::new();
            Self::open_dot(map, left, right, |map, stmt| {
                let str = stmt.as_word_or_string(map)?;
                result.push_str(&str);
                result.push('/');
                stack.push(str);
                Ok(())
            })
            .map_err(|err| err.with("When importing"))?;
            result.pop();
            result
        } else {
            let name = opd
                .as_word_or_string(map)
                .map_err(|err| err.with("When importing"))?;
            stack.push(name.clone());
            name
        };

        let path = map
            .env()
            .find_module(&name)
            .ok_or_else(|| Error::new(format!("Module {:?} not found", name), map.line()))?;

        let (mut res_map, result) = match map.env().get_import(&path) {
            Some(res) => (res, None),
            None => {
                let content = std::fs::read_to_string(&path).map_err(|err| {
                    Error::with_source(err, format!("When reading module {:?}", path))
                })?;

                let stmt = Compilable::new(&content)
                    .compile()
                    .map_err(|err| err.with(format!("When compiling module {:?}", path)))?;

                let mut new_map = Map::new_under(map);
                new_map.link(std::mem::take(map));
                new_map.env().forward_base(path.clone());
                let result = stmt
                    .eval(&mut new_map)
                    .map_err(|err| err.with(format!("When evaluating module {:?}", path)))?;
                new_map.env().backward_base();
                new_map.unlink_to(map);

                let res_map = Resource::new(new_map);
                map.env().set_import(path, &res_map);

                (res_map, Some(result))
            }
        };

        while stack.len() > 1 {
            let name = stack.pop().unwrap();
            let new_map = Resource::new(Map::new_under(map));
            new_map.visit_mut(|map: &mut Map| map.set(name, Value::Res(res_map.clone())));
            res_map = new_map;
        }

        map.set(stack.pop().unwrap(), Value::Res(res_map.clone()));

        Ok(result.unwrap_or_else(|| Value::Null))
    }

    fn eval_include(map: &mut Map, opd: &Self) -> Result<Value> {
        let name = if let Stmt::Dot(left, right) = opd {
            let mut result = String::new();
            Self::open_dot(map, left, right, |map, stmt| {
                result.push_str(&stmt.as_word_or_string(map)?);
                result.push('/');
                Ok(())
            })
            .map_err(|err| err.with("When including"))?;
            result.pop();
            result
        } else {
            opd.as_word_or_string(map)
                .map_err(|err| err.with("When including"))?
        };

        let path = map
            .env()
            .find_module(&name)
            .ok_or_else(|| Error::new(format!("Module {:?} not found", name), map.line()))?;
        let content = std::fs::read_to_string(&path)
            .map_err(|err| Error::with_source(err, format!("When reading module {:?}", path)))?;

        Compilable::new(&content)
            .compile()
            .map_err(|err| err.with(format!("When compiling module {:?}", path)))?
            .eval(map)
    }

    fn eval_map(map: &mut Map, opd: &Self) -> Result<Value> {
        let mut new_map = Map::new_under(map);

        new_map.link(std::mem::take(map));
        let result = opd.eval(&mut new_map);
        new_map.unlink_to(map);

        result?;

        Ok(Value::Res(Resource::new(new_map)))
    }

    fn eval_fn(map: &mut Map, body: &Rc<Self>) -> Result<Value> {
        let body = body.clone();
        let shared = map.get("shared").map(|value| value.clone().downgrade());

        let f = move |map: &mut Map, arg: Value| -> Result<Value> {
            map.push("arg", arg);
            let use_shared = if let Some(shared) = shared.clone() {
                map.push("shared", shared);
                true
            } else {
                false
            };

            let result = body.eval(map);

            if use_shared {
                map.pop("shared");
            }
            map.pop("arg");
            result
        };
        Ok(Value::Res(Resource::new_func(Func::new(f))))
    }

    fn eval_list(map: &mut Map, left: &Self, right: &Self) -> Result<Value> {
        let mut result = VecDeque::new();
        Self::open_list(map, left, right, |map, stmt| {
            result.push_back(stmt.eval(map)?);
            Ok(())
        })?;
        Ok(Value::Res(Resource::new(result)))
    }

    fn eval_move(map: &mut Map, opd: &Self) -> Result<Value> {
        let name = opd
            .as_word_or_string(map)
            .map_err(|err| err.with("When moving"))?;
        Ok(map.rem(&name).unwrap_or_else(|| Value::Stop))
    }

    fn eval_else(map: &mut Map, left: &Self, right: &Self) -> Result<Value> {
        match left {
            Self::Then(cond, first) => {
                let cond = cond
                    .eval(map)
                    .map_err(|err| err.with("When evaluating condition"))?;
                let cond = cond.as_bool().ok_or_else(|| {
                    Error::new(format!("Condition {:?} is not a boolean", cond), map.line())
                })?;
                if cond {
                    first.eval(map)
                } else {
                    right.eval(map)
                }
            }
            _ => Err(Error::new(
                "Else keyword should be used after then keyword",
                map.line(),
            )),
        }
    }

    fn eval_then(map: &mut Map, left: &Self, right: &Self) -> Result<Value> {
        let cond = left
            .eval(map)
            .map_err(|err| err.with("When evaluating condition"))?;
        let cond = cond.as_bool().ok_or_else(|| {
            Error::new(format!("Condition {:?} is not a boolean", cond), map.line())
        })?;
        if cond {
            right.eval(map)
        } else {
            Ok(Value::Stop)
        }
    }

    fn eval_repeat(map: &mut Map, left: &Self, right: &Self) -> Result<Value> {
        let mut result = Value::Stop;
        loop {
            let cond = left.eval(map)?;
            let cond = cond.as_bool().ok_or_else(|| {
                Error::new(format!("Condition {:?} is not a boolean", cond), map.line())
            })?;
            if !cond {
                break;
            }
            result = right.eval(map)?;
        }
        Ok(result)
    }
}

impl Eval for Stmt {
    fn eval(&self, map: &mut Map) -> Result<Value> {
        match self {
            Self::Token(token, line) => {
                map.set_line(*line);
                token.eval(map)
            }
            Self::Block(block) => {
                let mut result = Value::Null;
                for stmt in block {
                    match stmt {
                        Self::Return(value) => {
                            return Ok(value.eval(map)?);
                        }
                        _ => {
                            result = stmt.eval(map)?;
                        }
                    }
                }
                Ok(result)
            }
            Self::Empty => Ok(Value::Null),
            Self::Dot(_, _) => self.get(map),
            Self::Colon(left, right) => Self::eval_colon(map, left, right),
            Self::Import(opd) => Self::eval_import(map, opd),
            Self::Include(opd) => Self::eval_include(map, opd),
            Self::Extern(opd) => {
                let line = map.line();
                opd.eval(map.parent_mut().ok_or_else(|| {
                    Error::new(
                        "Extern keyword is used(right value), but no parent map is found",
                        line,
                    )
                })?)
            }
            Self::Map(opd) => Self::eval_map(map, opd),
            Self::Fn(body) => Self::eval_fn(map, body),
            Self::Neg(_opd) => {
                todo!()
            }
            Self::Move(opd) => Self::eval_move(map, opd),
            Self::Acq(opd) => Ok(opd
                .eval(map)?
                .upgrade()
                .ok_or_else(|| Error::new("Attempted to acquire deleted value", map.line()))?),
            Self::Return(opd) => Ok(opd
                .eval(map)?
                .upgrade()
                .ok_or_else(|| Error::new("Attempted to return deleted value", map.line()))?),
            Self::Call(left, right) => {
                let left = left.eval(map)?;
                let right = right.eval(map)?;
                left.call(map, right).ok_or_else(|| {
                    Error::new(format!("Cannot call value {:?}", left), map.line())
                })?
            }
            Self::List(left, right) => Self::eval_list(map, left, right),
            Self::Else(left, right) => Self::eval_else(map, left, right),
            Self::Then(left, right) => Self::eval_then(map, left, right),
            Self::Repeat(left, right) => Self::eval_repeat(map, left, right),
            Self::Asn(left, right) => {
                let right = right.eval(map)?;
                left.set(map, right)
            }
        }
    }

    fn get(&self, map: &mut Map) -> Result<Value> {
        match self {
            Self::Token(token, _) => token.get(map),
            Self::Dot(left, right) => {
                let left = left.eval(map)?;
                let res = left.as_res().ok_or_else(|| {
                    Error::new(
                        format!("Cannot get value from dot left {:?}", left),
                        map.line(),
                    )
                })?;
                match res
                    .visit_mut(|map: &mut Map| right.get(map))
                    .ok_or_else(|| {
                        Error::new(
                            format!("Cannot get value from dot left {:?}", left),
                            map.line(),
                        )
                    })? {
                    Ok(value) => Ok(value),
                    Err(err) => {
                        res.visit_mut(|map: &mut Map| {
                            map.get("meta")
                                .ok_or_else(|| err)?
                                .as_res()
                                .ok_or_else(|| {
                                    Error::new("\"meta\" is found, but is not a map", map.line())
                                })?
                                .visit_mut(|map: &mut Map| right.get(map))
                                .ok_or_else(|| {
                                    Error::new(
                                        format!("Cannot get value from dot left(although \"meta\" map found) {:?}", left),
                                        map.line(),
                                    )
                                })?
                        })
                        .unwrap()
                        // Here we are sure that res is a map
                    }
                }
            }
            Self::Extern(opd) => {
                let line = map.line();
                opd.get(map.parent_mut().ok_or_else(|| {
                    Error::new(
                        "Extern keyword is used(right value), but no parent map is found",
                        line,
                    )
                })?)
            }
            _ => {
                let value = self.eval(map)?;
                match value {
                    Value::Stop => Ok(Value::Stop),
                    _ => {
                        value
                            .as_res()
                            .ok_or_else(|| {
                                Error::new(
                                    format!("Cannot get value from name {:?}", value),
                                    map.line(),
                                )
                            })?
                            .visit(|s: &String| map.get(s).cloned())
                            .ok_or_else(|| {
                                Error::new(
                                    format!("Cannot get value from name {:?}", value),
                                    map.line(),
                                )
                            })?;
                        Ok(value)
                    }
                }
            }
        }
    }

    fn set(&self, map: &mut Map, value: Value) -> Result<Value> {
        match self {
            Self::Token(token, line) => {
                map.set_line(*line);
                token.set(map, value)
            }
            Self::Dot(left, right) => {
                let left = left.eval(map)?;
                left.as_res()
                    .ok_or_else(|| {
                        Error::new(
                            format!("Cannot set value to dot left {:?}", left),
                            map.line(),
                        )
                    })?
                    .visit_mut(|map: &mut Map| right.set(map, value))
                    .ok_or_else(|| {
                        Error::new(
                            format!("Cannot set value to dot left {:?}", left),
                            map.line(),
                        )
                    })?
            }
            Self::Extern(opd) => {
                let line = map.line();
                opd.set(
                    map.parent_mut().ok_or_else(|| {
                        Error::new(
                            "Extern keyword is used(left value), but no parent map is found",
                            line,
                        )
                    })?,
                    value,
                )
            }
            Self::List(left, right) => {
                value
                    .as_res()
                    .ok_or_else(|| {
                        Error::new(format!("Cannot unwrap non-list {:?}", value), map.line())
                    })?
                    .visit(|list: &VecDeque<Value>| {
                        let mut iter = list.iter();
                        Self::open_list(map, left, right, |map, stmt| {
                            stmt.set(
                                map,
                                iter.next()
                                    .ok_or_else(|| Error::new("", map.line()))?
                                    .clone(),
                            )?;
                            Ok(())
                        })
                    })
                    .ok_or_else(|| {
                        Error::new(format!("Cannot unwrap non-list {:?}", value), map.line())
                    })??;
                Ok(value)
            }
            _ => {
                let left = self.eval(map)?;
                left.as_res()
                    .ok_or_else(|| {
                        Error::new(
                            format!("Cannot set value to name {:?}", left.clone()),
                            map.line(),
                        )
                    })?
                    .visit(|s: &String| map.set(s.to_string(), value.clone()))
                    .ok_or_else(|| {
                        Error::new(
                            format!("Cannot set value to name {:?}", left.clone()),
                            map.line(),
                        )
                    })?;
                Ok(value)
            }
        }
    }
}
