use super::*;

impl Stmt {
    fn open_list<F>(map: &mut Map, left: &Self, right: &Self, mut f: F) -> Result<()>
    where
        F: FnMut(&mut Map, &Self) -> Result<()>,
    {
        f(map, left)?;
        match right {
            Self::List(left, right) => Self::open_list(map, left, right, f)?,
            Self::Token(Token::Stop, _) => {}
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

    fn eval_list(map: &mut Map, left: &Self, right: &Self) -> Result<Value> {
        let mut result = VecDeque::new();
        Self::open_list(map, left, right, |map, stmt| {
            result.push_back(stmt.eval(map)?);
            Ok(())
        })?;
        Ok(Value::Res(Resource::new(result)))
    }

    fn eval_fn(map: &mut Map, left: &Self, right: &Self, is_proc: bool) -> Result<Value> {
        let mut params = VecDeque::new();
        let mut len = 0;
        match left {
            Self::Empty => {}
            Self::List(left, right) => {
                Self::open_list(map, left, right, |map, stmt| {
                    match stmt {
                        Self::Token(token, line) => {
                            map.set_line(*line);
                            match token {
                                Token::Word(word) => {
                                    params.push_back((word.clone(), None));
                                    len += 1;
                                    Ok(())
                                }
                                _ => {
                                    Err(Error::new(
                                        format!("Cannot evaluate function param {:?}. Only words are accepted", stmt),
                                        map.line(),
                                    )) 
                                }
                            }
                        }
                        Self::Colon(left, right) => {
                            let name = left.as_word(map).ok_or_else(|| Error::new(format!("Cannot use {:?} as a function environment name", left), map.line()))?;
                            let value = right.eval(map)?;
                            params.push_back((name, Some(value)));
                            Ok(())
                        }
                        _ => Err(Error::new(
                            format!("Cannot evaluate function param {:?}. Only words are accepted", stmt),
                            map.line(),
                        )),
                    }
                })?;
            }
            Self::Token(token,line) => {
                map.set_line(*line);
                match token {
                    Token::Word(name) => {
                        params.push_back((name.clone(), None));
                        len += 1;
                    }
                    _ => {
                        return Err(Error::new(format!("Cannot evaluate function param {:?}. Only words are accepted", left), map.line()))
                    }
                }
            }
            _ => {
                return Err(Error::new(
                    format!("Cannot evaluate function params {:?}, please use parentheses enclosed expression", left),
                    map.line(),
                ))
            }
        }

        let body = right.clone();
        let f = move |map: &mut Map, args: VecDeque<Value>| {
            if args.len() != len {
                Err(Error::new(
                    format!(
                        "Incorrect number of arguments are found. Expected {}, but found {}",
                        len,
                        args.len()
                    ),
                    map.line(),
                ))
            } else {
                let old_map = if !is_proc {
                    Some(std::mem::take(map))
                } else {
                    None
                };

                let params = params.iter();
                let mut args = args.iter();

                for (param, default) in params {
                    match default {
                        Some(value) => {
                            map.set(param.clone(), value.clone());
                        }
                        None => {
                            let arg = args.next().unwrap();
                            map.set(param.clone(), arg.clone());
                        }
                    }
                }
                let result = body.eval(map);
                if let Some(mut old_map) = old_map {
                    old_map.set_line(map.line());
                    let _ = std::mem::replace(map, old_map);
                }
                result
            }
        };
        Ok(Value::Res(Resource::new_func(Func::new(f))))
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
                    result = stmt.eval(map)?;
                }
                Ok(result)
            }
            Self::Empty => Ok(Value::Null),
            Self::Dot(_, _) => self.get(map),
            Self::Colon(left, right) => Self::eval_colon(map, left, right),
            Self::Neg(_opd) => {
                todo!()
            }
            Self::Fn(left, right, is_proc) => Self::eval_fn(map, left, right, *is_proc),
            Self::Call(left, right) => {
                let left = left.eval(map)?;
                let mut args = VecDeque::new();
                match right.as_ref() {
                    Self::List(left, right) => {
                        Self::open_list(map, left, right, |map, stmt| {
                            args.push_back(stmt.eval(map)?);
                            Ok(())
                        })?;
                    }
                    Self::Empty => {}
                    _ => {
                        args.push_back(right.eval(map)?);
                    }
                }
                left.call(map, args).ok_or_else(|| {
                    Error::new(format!("Cannot call value {:?}", left), map.line())
                })?
            }
            Self::List(left, right) => Self::eval_list(map, left, right),
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
                left.as_res()
                    .ok_or_else(|| {
                        Error::new(
                            format!("Cannot get value from dot left {:?}", left),
                            map.line(),
                        )
                    })?
                    .visit_mut(|map: &mut Map| right.get(map))
                    .ok_or_else(|| {
                        Error::new(
                            format!("Cannot get value from dot left {:?}", left),
                            map.line(),
                        )
                    })?
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
