use super::*;

#[derive(Debug, Clone)]
pub enum MatcherEntry {
    Matched(Value),
    MatchedRest(VecDeque<Value>),
    Vacant,
}

#[derive(Debug, Clone)]
pub enum Matcher {
    Single(MatcherEntry, &'static str),
    Listed(VecDeque<Matcher>),
    Rest(MatcherEntry),
}

impl MatcherEntry {
    pub fn new() -> Self {
        Self::Vacant
    }

    pub fn set(&mut self, value: Value) {
        *self = MatcherEntry::Matched(value);
    }

    pub fn set_rest(&mut self, vec: VecDeque<Value>) {
        *self = MatcherEntry::MatchedRest(vec);
    }
}

impl Matcher {
    fn new() -> Self {
        Self::Single(MatcherEntry::new(), "arg")
    }

    pub fn single(name: &'static str) -> Self {
        Self::Single(MatcherEntry::new(), name)
    }

    pub fn listed() -> Self {
        Self::Listed(VecDeque::new())
    }

    pub fn with(self, matcher: Matcher) -> Self {
        match self {
            Self::Listed(mut list) => {
                list.push_back(matcher);
                Self::Listed(list)
            }
            _ => self,
        }
    }

    pub fn with_singles(mut self, names: &'static [&'static str]) -> Self {
        for name in names.iter() {
            self = self.with(Self::single(name));
        }
        self
    }

    pub fn with_rest(self) -> Self {
        self.with(Matcher::rest())
    }

    pub fn rest() -> Self {
        Self::Rest(MatcherEntry::new())
    }

    pub fn to_single(self) -> Option<Value> {
        if let Self::Single(MatcherEntry::Matched(value), _) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn to_list(self) -> Option<VecDeque<Matcher>> {
        if let Self::Listed(list) = self {
            Some(list)
        } else {
            None
        }
    }

    pub fn to_values(self) -> Option<VecDeque<Value>> {
        if let Self::Listed(list) = self {
            Some(list.into_iter().filter_map(|m| m.to_single()).collect())
        } else {
            None
        }
    }

    pub fn to_rest(self) -> Option<VecDeque<Value>> {
        if let Self::Rest(MatcherEntry::MatchedRest(vec)) = self {
            Some(vec)
        } else {
            None
        }
    }

    pub fn mat(&mut self, value: Value) -> Option<()> {
        match self {
            Self::Single(entry, _) => {
                entry.set(value);
                Some(())
            }
            Self::Listed(list) => value.as_res()?.visit_mut(|values: &mut VecDeque<Value>| {
                let len = list.len();
                for (i, matcher) in list.iter_mut().enumerate() {
                    match matcher {
                        Matcher::Rest(entry) => {
                            let mut vec = VecDeque::new();
                            while values.len() > len - i - 1 {
                                vec.push_back(values.pop_front()?);
                            }
                            entry.set_rest(vec);
                        }
                        _ => {
                            matcher.mat(values.pop_front()?)?;
                        }
                    }
                }
                if values.is_empty() {
                    Some(())
                } else {
                    None
                }
            })?,
            Self::Rest(list) => {
                let mut vec = VecDeque::new();
                vec.push_back(value);
                list.set_rest(vec);
                Some(())
            }
        }
    }

    pub fn mat_or_err(&mut self, value: Value, line: usize) -> Result<()> {
        self.mat(value).ok_or_else(|| {
            Error::new(
                format!("Unable to match with argument shape {}", self),
                line,
            )
        })
    }
}

impl fmt::Display for Matcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Single(_, name) => write!(f, "{}", name),
            Self::Listed(list) => write!(
                f,
                "({})",
                list.iter()
                    .map(|m| format!("{}", m))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Rest(_) => write!(f, "..."),
        }
    }
}

impl Default for Matcher {
    fn default() -> Self {
        Self::new()
    }
}
