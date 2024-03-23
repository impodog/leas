use crate::prelude::*;

impl TryInto<toml::Value> for &Value {
    type Error = Error;
    fn try_into(self) -> Result<toml::Value> {
        match self {
            Value::Bool(b) => Ok(toml::Value::Boolean(*b)),
            Value::Float(f) => Ok(toml::Value::Float(*f)),
            Value::Int(i) => Ok(toml::Value::Integer(*i)),
            Value::Uint(u) => Ok(toml::Value::Integer(*u as i64)),
            Value::Null => Ok(toml::Value::Float(f64::NAN)),
            _ => (&self
                .as_res()
                .ok_or_else(|| Error::new("Invalid type for toml::Value", 0))?)
                .try_into(),
        }
    }
}

impl TryInto<toml::Value> for &Resource {
    type Error = Error;
    fn try_into(self) -> Result<toml::Value> {
        if let Some(result) = self.visit(|s: &String| Ok(toml::Value::String(s.clone()))) {
            return result;
        }
        if let Some(result) = self.visit(|v: &VecDeque<Value>| {
            let mut vec = Vec::new();
            for value in v {
                vec.push(value.try_into()?);
            }
            Ok(toml::Value::Array(vec))
        }) {
            return result;
        }
        if let Some(result) = self.visit(|map: &Map| map.try_into()) {
            return result;
        }
        Err(Error::new("Invalid type for toml::Value", 0))
    }
}

impl TryInto<toml::Value> for &Map {
    type Error = Error;
    fn try_into(self) -> Result<toml::Value> {
        let mut map = toml::value::Table::new();
        for (key, value) in self.iter() {
            map.insert(key.clone(), value.try_into()?);
        }
        Ok(toml::Value::Table(map))
    }
}

impl TryFrom<toml::Value> for Value {
    type Error = Error;

    fn try_from(value: toml::Value) -> Result<Value> {
        match value {
            toml::Value::Boolean(b) => Ok(Value::Bool(b)),
            toml::Value::Integer(i) => Ok(Value::Int(i as i64)),
            toml::Value::Float(f) => Ok(Value::Float(f)),
            toml::Value::String(s) => Ok(Value::Res(Resource::new(s))),
            toml::Value::Array(a) => {
                let mut vec = VecDeque::new();
                for value in a {
                    vec.push_back(Value::try_from(value)?);
                }
                Ok(Value::Res(Resource::new(vec)))
            }
            toml::Value::Table(t) => {
                let mut map = Map::new();
                for (key, value) in t {
                    map.forced_set(key, Value::try_from(value)?);
                }
                Ok(Value::Res(Resource::new(map)))
            }
            toml::Value::Datetime(_) => {
                Err(Error::new("Converting from date time is not supported", 0))
            }
        }
    }
}

fn unwrap(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::single("toml");
    matcher.mat_or_err(arg, map.line())?;

    let toml_value = matcher.to_single().unwrap();

    let line = map.line();
    let value = toml_value.visit_res_or_else(
        move |v: &toml::Value| Value::try_from(v.clone()),
        move || {
            Error::new(
                "Non-toml value cannot be converted using this function",
                line,
            )
        },
    )??;

    Ok(value)
}

fn from(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::single("value");
    matcher.mat_or_err(arg, map.line())?;

    let value = matcher.to_single().unwrap();

    let toml_value: toml::Value = (&value).try_into()?;

    Ok(Value::Res(Resource::new(toml_value)))
}

fn to_str(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::single("toml");
    matcher.mat_or_err(arg, map.line())?;

    let toml_value = matcher.to_single().unwrap();

    let line = map.line();
    let value = toml_value.visit_res_or_else(
        |v: &toml::Value| {
            Ok(Value::Res(Resource::new(toml::to_string(v).map_err(
                |err| Error::with_source(err, "When converting to toml string"),
            )?)))
        },
        move || Error::new("Non-toml value cannot be converted to string", line),
    )??;

    Ok(value)
}

fn from_str(map: &mut Map, arg: Value) -> Result<Value> {
    let mut matcher = Matcher::single("str");
    matcher.mat_or_err(arg, map.line())?;

    let s = matcher.to_single().unwrap();

    let line = map.line();
    let value = s.visit_res_or_else(
        |s: &String| {
            let toml_value = toml::from_str::<toml::Value>(s)
                .map_err(|err| Error::with_source(err, "When converting string to toml value"))?;
            Ok(Value::Res(Resource::new(toml_value)))
        },
        move || Error::new("Non-string value cannot be converted to toml value", line),
    )??;

    Ok(value)
}

fn init_module(map: &mut Map) {
    map.register("unwrap", unwrap);
    map.register("from", from);
    map.register("to_str", to_str);
    map.register("from_str", from_str);
}

pub fn init(map: &mut Map) {
    map.register_init("toml", init_module);
}
