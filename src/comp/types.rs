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
    Mono,
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

    Dot,
    Call,
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
    End(usize),

    Token(Token),
    Block(VecDeque<Stmt>),

    Dot(Box<Stmt>, Box<Stmt>),
    Call(Box<Stmt>, Box<Stmt>),
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
            Self::Call => 10,
            Self::Asn => 200,
            _ => 0,
        }
    }

    pub fn attr(&self) -> Option<Operator> {
        match self {
            Self::Dot | Self::Call => Some(Operator::Left),
            Self::Asn => Some(Operator::Right),
            _ => None,
        }
    }

    pub fn to_stmt_fn(&self) -> fn(Box<Stmt>, Box<Stmt>) -> Stmt {
        match self {
            Self::Dot => Stmt::Dot,
            Self::Call => Stmt::Call,
            Self::Asn => Stmt::Asn,
            _ => panic!("Cannot convert {:?} to stmt function", self),
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
