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
            Self::Word(value) => map.get(value).cloned().ok_or_else(|| {
                Error::new(format!("Cannot find value named {:?}", value), map.line())
            }),
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
            Self::Word(key) | Self::Str(key) => map.get(key).cloned().ok_or_else(|| {
                Error::new(format!("Cannot find value named {:?}", key), map.line())
            }),
            _ => self.eval(map),
        }
    }

    fn set(&self, map: &mut Map, value: Value) -> Result<Value> {
        match self {
            Self::Word(key) | Self::Str(key) => {
                map.set(key.clone(), value.clone());
                Ok(value)
            }
            _ => Err(Error::new(
                format!("Cannot set value to token {:?}", self),
                map.line(),
            )),
        }
    }
}