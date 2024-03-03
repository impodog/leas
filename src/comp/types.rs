use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Enclosing {
    Paren,
    Bracket,
    Brace,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Operator {
    Left,
    Right,
    Unary,
}

#[derive(Debug, Clone)]
pub enum Token {
    End(usize),
    Enter(usize, Enclosing),

    Word(String),
    Int(Int),
    Float(Float),
    Uint(Uint),
    Bool(Bool),
    Null,
    Str(String),

    Stop,

    Dot,
    Colon,
    Fn,
    Pc,
    Neg,
    Call,
    List,
    Asn,
}

#[derive(Debug, Clone)]
pub enum Slice {
    End(usize),

    Token(Token),
    Line(VecDeque<Slice>),
    Block(VecDeque<Slice>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Token(Token, usize),
    Block(VecDeque<Stmt>),
    Empty,

    Dot(Box<Stmt>, Box<Stmt>),
    Colon(Box<Stmt>, Box<Stmt>),
    Fn(Box<Stmt>, Rc<Stmt>, bool),
    Neg(Box<Stmt>),
    Call(Box<Stmt>, Box<Stmt>),
    List(Box<Stmt>, Box<Stmt>),
    Asn(Box<Stmt>, Box<Stmt>),
}

pub struct Compilable<'s>(pub &'s str);

pub struct Stream(pub Vec<Token>);

pub struct Sliced(pub Slice);

pub struct Cooked(pub Stmt);

impl From<char> for Enclosing {
    fn from(value: char) -> Self {
        match value {
            '(' | ')' => Enclosing::Paren,
            '[' | ']' => Enclosing::Bracket,
            '{' | '}' => Enclosing::Brace,
            _ => panic!("Cannot convert {:?} to enclosing pairs", value),
        }
    }
}

impl Token {
    pub fn priority(&self) -> u8 {
        match self {
            Self::Dot => 1,
            Self::Colon => 2,
            Self::Fn | Self::Pc => 5,
            Self::Neg => 10,
            Self::List => 50,
            Self::Call => 20,
            Self::Asn => 200,
            _ => 0,
        }
    }

    pub fn attr(&self) -> Option<Operator> {
        match self {
            Self::Dot | Self::Colon | Self::Fn | Self::Pc | Self::Call => Some(Operator::Left),
            Self::List | Self::Asn => Some(Operator::Right),
            Self::Neg => Some(Operator::Unary),
            _ => None,
        }
    }

    pub fn to_stmt_fn(&self) -> fn(Box<Stmt>, Box<Stmt>) -> Stmt {
        fn fn_fn(left: Box<Stmt>, right: Box<Stmt>) -> Stmt {
            Stmt::Fn(left, Rc::new(*right), false)
        }

        fn pc_fn(left: Box<Stmt>, right: Box<Stmt>) -> Stmt {
            Stmt::Fn(left, Rc::new(*right), true)
        }

        match self {
            Self::Dot => Stmt::Dot,
            Self::Colon => Stmt::Colon,
            Self::Fn => fn_fn,
            Self::Pc => pc_fn,
            Self::Call => Stmt::Call,
            Self::List => Stmt::List,
            Self::Asn => Stmt::Asn,
            _ => panic!("Cannot convert {:?} to binary stmt function", self),
        }
    }

    pub fn to_stmt_unary_fn(&self) -> fn(Box<Stmt>) -> Stmt {
        match self {
            Self::Neg => Stmt::Neg,
            _ => panic!("Cannot convert {:?} to unary stmt function", self),
        }
    }

    pub fn is_op(&self) -> bool {
        self.priority() != 0
    }

    pub fn modify_line(&self, line: &Cell<usize>) -> bool {
        match self {
            Self::End(l) => {
                line.set(*l);
                true
            }
            _ => false,
        }
    }
}

impl Enclosing {
    pub fn to_left(self) -> char {
        match self {
            Enclosing::Paren => '(',
            Enclosing::Bracket => '[',
            Enclosing::Brace => '{',
        }
    }

    pub fn to_right(self) -> char {
        match self {
            Enclosing::Paren => ')',
            Enclosing::Bracket => ']',
            Enclosing::Brace => '}',
        }
    }
}

impl Stmt {
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
}

impl<'s> Compilable<'s> {
    pub fn new(str: &'s str) -> Self {
        Self(str)
    }
}

impl Stream {
    pub fn new(stream: Vec<Token>) -> Self {
        Self(stream)
    }
}

impl Sliced {
    pub fn new(slice: Slice) -> Self {
        Self(slice)
    }
}

impl Cooked {
    pub fn new(stmt: Stmt) -> Self {
        Self(stmt)
    }
}
