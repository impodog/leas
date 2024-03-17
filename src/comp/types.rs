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
    Use,
    Import,
    Include,
    Extern,
    Map,
    Fn,
    Neg,
    Move,
    Acq,
    Return,
    Call,
    Do,
    List,
    Expose,
    Then,
    Else,
    Repeat,
    Colon,
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
    Import(Box<Stmt>),
    Include(Box<Stmt>),
    Extern(Box<Stmt>),
    Map(Box<Stmt>),
    Fn(Rc<Stmt>),
    Neg(Box<Stmt>),
    Move(Box<Stmt>),
    Acq(Box<Stmt>),
    Return(Box<Stmt>),
    Call(Box<Stmt>, Box<Stmt>),
    Do(Box<Stmt>, Box<Stmt>),
    List(Box<Stmt>, Box<Stmt>),
    Use(Box<Stmt>),
    Expose(Box<Stmt>),
    Then(Box<Stmt>, Box<Stmt>),
    Else(Box<Stmt>, Box<Stmt>),
    Repeat(Box<Stmt>, Box<Stmt>),
    Colon(Box<Stmt>, Box<Stmt>),
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
            Self::Import | Self::Include | Self::Extern => 3,
            Self::Map => 4,
            Self::Fn => 5,
            Self::Neg => 10,
            Self::Move | Self::Acq | Self::Return => 15,
            Self::Call => 20,
            Self::Do => 30,
            Self::List => 50,
            Self::Use | Self::Expose => 60,
            Self::Then => 100,
            Self::Else => 101,
            Self::Repeat => 102,
            Self::Colon => 103,
            Self::Asn => 200,
            _ => 0,
        }
    }

    pub fn attr(&self) -> Option<Operator> {
        match self {
            Self::Dot | Self::Call | Self::Colon => Some(Operator::Left),
            Self::Do | Self::List | Self::Then | Self::Else | Self::Repeat | Self::Asn => {
                Some(Operator::Right)
            }

            Self::Import
            | Self::Include
            | Self::Extern
            | Self::Fn
            | Self::Map
            | Self::Neg
            | Self::Move
            | Self::Acq
            | Self::Return
            | Self::Use
            | Self::Expose => Some(Operator::Unary),
            _ => None,
        }
    }

    pub fn to_stmt_fn(&self) -> fn(Box<Stmt>, Box<Stmt>) -> Stmt {
        match self {
            Self::Dot => Stmt::Dot,

            Self::Call => Stmt::Call,
            Self::Do => Stmt::Do,
            Self::List => Stmt::List,
            Self::Then => Stmt::Then,
            Self::Else => Stmt::Else,
            Self::Repeat => Stmt::Repeat,
            Self::Colon => Stmt::Colon,
            Self::Asn => Stmt::Asn,
            _ => panic!("Cannot convert {:?} to binary stmt function", self),
        }
    }

    pub fn to_stmt_unary_fn(&self) -> fn(Box<Stmt>) -> Stmt {
        fn fn_fn(stmt: Box<Stmt>) -> Stmt {
            Stmt::Fn(Rc::new(*stmt))
        }
        match self {
            Self::Import => Stmt::Import,
            Self::Include => Stmt::Include,
            Self::Extern => Stmt::Extern,
            Self::Fn => fn_fn,
            Self::Map => Stmt::Map,
            Self::Neg => Stmt::Neg,
            Self::Move => Stmt::Move,
            Self::Acq => Stmt::Acq,
            Self::Return => Stmt::Return,
            Self::Use => Stmt::Use,
            Self::Expose => Stmt::Expose,
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

impl<'s> Compilable<'s> {
    pub fn new(str: &'s str) -> Self {
        Self(str)
    }

    pub fn compile(self) -> Result<Stmt> {
        Ok(self.lex()?.slice().cook()?.0)
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
