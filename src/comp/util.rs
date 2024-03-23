use super::*;
use rt::Eval;

impl Stmt {
    pub fn open_list<F>(map: &mut Map, left: &Self, right: &Self, mut f: F) -> Result<()>
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

    pub fn open_dot<F>(map: &mut Map, left: &Self, right: &Self, mut f: F) -> Result<()>
    where
        F: FnMut(&mut Map, &Self, bool) -> Result<()>,
    {
        f(map, left, false)?;

        match right {
            Self::Dot(left, right) => Self::open_dot(map, left, right, f)?,
            _ => f(map, right, true)?,
        }
        Ok(())
    }

    pub fn as_word(&self, map: &mut Map) -> Option<String> {
        match self {
            Self::Token(token, line) => {
                map.set_line(*line);
                match token {
                    Token::Word(name) => Some(name.to_string()),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn as_word_or_string(&self, map: &mut Map) -> Result<String> {
        if let Stmt::Token(token, line) = self {
            map.set_line(*line);
            if let Token::Word(name) = token {
                return Ok(name.to_string());
            }
        }
        self.eval(map)?
            .as_res()
            .ok_or_else(|| {
                Error::new(
                    format!("Value {:?} cannot be used as name", self),
                    map.line(),
                )
            })?
            .visit(|s: &String| Ok(s.to_string()))
            .ok_or_else(|| {
                Error::new(
                    format!("Value {:?} cannot be used as name", self),
                    map.line(),
                )
            })?
    }

    pub fn open_list_or_single<F>(&self, map: &mut Map, mut f: F) -> Result<()>
    where
        F: FnMut(&mut Map, &Self) -> Result<()>,
    {
        match self {
            Self::List(left, right) => Self::open_list(map, left, right, f)?,
            _ => f(map, self)?,
        }
        Ok(())
    }

    pub fn open_dot_or_single<F>(&self, map: &mut Map, mut f: F) -> Result<()>
    where
        F: FnMut(&mut Map, &Self, bool) -> Result<()>,
    {
        match self {
            Self::Dot(left, right) => Self::open_dot(map, left, right, f)?,
            _ => f(map, self, true)?,
        }
        Ok(())
    }

    pub fn to_fn(map: &mut Map, body: &Rc<Self>) -> Result<Value> {
        let body = body.clone();
        // Downgrading here is unnecessary
        let shared = map.get("shared").cloned();

        let f = move |map: &mut Map, arg: Value| -> Result<Value> {
            map.push_name("arg", arg);
            let use_shared = if let Some(shared) = shared.clone() {
                map.push_name("shared", shared);
                true
            } else {
                false
            };

            map.snapshot();
            let result = body.eval(map);
            map.rollback();

            if use_shared {
                map.pop_name("shared");
            }
            map.pop_name("arg");
            result
        };
        Ok(Value::Res(Resource::new_func(Func::new_unnamed(f))))
    }
}
