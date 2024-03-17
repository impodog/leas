use super::*;

impl Eval for Token {
    fn eval(&self, map: &mut Map) -> Result<Value> {
        match self {
            Self::Int(value) => Ok(Value::Int(*value)),
            Self::Float(value) => Ok(Value::Float(*value)),
            Self::Uint(value) => Ok(Value::Uint(*value)),
            Self::Bool(value) => Ok(Value::Bool(*value)),
            Self::Null => Ok(Value::Null),
            Self::Str(value) => Ok(Value::Res(Resource::new(value.clone()))),
            Self::Word(_) => self.get(map),
            Self::Stop => Ok(Value::Stop),
            _ => Err(Error::new(
                format!(
                    "Cannot evaluate token {:?}. It is only used when compiling",
                    self
                ),
                map.line(),
            )),
        }
    }

    fn get(&self, map: &mut Map) -> Result<Value> {
        match self {
            Self::Word(key) | Self::Str(key) => map
                .get(key)
                .map(|value| value.clone().downgrade())
                .ok_or_else(|| {
                    Error::new(format!("Cannot find value named {:?}", key), map.line())
                }),
            _ => self.eval(map),
        }
    }

    fn set(&self, map: &mut Map, value: Value) -> Result<Value> {
        match self {
            Self::Word(key) | Self::Str(key) => {
                map.set(key.clone(), value.clone());
                value.as_res().and_then(|res| {
                    res.visit_mut_func(|func| {
                        if func.name.is_none() {
                            func.name = Some(key.clone())
                        }
                    })
                });
                Ok(value)
            }
            Self::Stop => Ok(Value::Stop),
            _ => Err(Error::new(
                format!("Cannot set value to token {:?}", self),
                map.line(),
            )),
        }
    }
}
