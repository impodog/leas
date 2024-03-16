use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Status {
    Normal,
    Int,
    Float,
    Str(bool),
    Word,
}

impl<'s> Compilable<'s> {
    pub fn lex(&self) -> Result<Stream> {
        let mut stream = Vec::new();

        let mut chars = self.0.chars().chain("\n".chars()).peekable();

        let mut status = Status::Normal;
        let mut buffer = String::new();
        let mut line = 1;
        let mut unclosed = Vec::new();

        while let Some(&c) = chars.peek() {
            let used = match status {
                Status::Normal => match c {
                    ' ' | '\t' => true,
                    '\n' => {
                        line += 1;
                        stream.push(Token::End(line));
                        true
                    }
                    '(' | '[' | '{' => {
                        unclosed.push((stream.len(), Enclosing::from(c)));
                        stream.push(Token::Null);
                        true
                    }
                    ')' | ']' | '}' => {
                        if let Some((index, left)) = unclosed.pop() {
                            if left != Enclosing::from(c) {
                                return Err(Error::new(
                                    format!(
                                        "Unmatched previous deliminator {:?} and {:?}",
                                        left.to_left(),
                                        c
                                    ),
                                    line,
                                ));
                            }
                            *stream.get_mut(index).unwrap() =
                                Token::Enter(stream.len() - index, left);
                            true
                        } else {
                            return Err(Error::new(format!("Unclosed deliminator {:?}", c), line));
                        }
                    }
                    '0'..='9' | '-' => {
                        status = Status::Int;
                        buffer.push(c);
                        true
                    }
                    '\"' => {
                        status = Status::Str(false);
                        true
                    }
                    'A'..='Z' | 'a'..='z' | '_' => {
                        status = Status::Word;
                        buffer.push(c);
                        true
                    }
                    '.' => {
                        stream.push(Token::Dot);
                        true
                    }
                    ':' => {
                        stream.push(Token::Colon);
                        true
                    }
                    ',' => {
                        stream.push(Token::List);
                        true
                    }
                    '=' => {
                        stream.push(Token::Asn);
                        true
                    }
                    _ => {
                        return Err(Error::new(
                            format!("Unexpected stray character when lexing: '{}'", c),
                            line,
                        ))
                    }
                },
                Status::Int => match c {
                    '0'..='9' => {
                        buffer.push(c);
                        true
                    }
                    '.' => {
                        buffer.push(c);
                        status = Status::Float;
                        true
                    }
                    'u' => {
                        let num: Uint = buffer.parse().map_err(|err| {
                            Error::with_source(
                                err,
                                format!("When parsing unsigned integer {:?}", buffer),
                            )
                        })?;
                        stream.push(Token::Uint(num));
                        buffer.clear();
                        status = Status::Normal;
                        true
                    }
                    _ => {
                        if buffer.ends_with('-') {
                            stream.push(Token::Neg);
                        } else {
                            let num: Int = buffer.parse().map_err(|err| {
                                Error::with_source(
                                    err,
                                    format!("When parsing integer {:?}", buffer),
                                )
                            })?;
                            stream.push(Token::Int(num));
                        }
                        status = Status::Normal;
                        buffer.clear();
                        false
                    }
                },
                Status::Float => match c {
                    '0'..='9' => {
                        buffer.push(c);
                        true
                    }
                    _ => {
                        if buffer.ends_with('.') {
                            let num: Int = buffer[0..buffer.len() - 1].parse().map_err(|err| {
                                Error::with_source(
                                    err,
                                    format!("When parsing integer {:?}", buffer),
                                )
                            })?;
                            stream.push(Token::Int(num));
                            stream.push(Token::Dot);
                        } else {
                            let num: Float = buffer.parse().map_err(|err| {
                                Error::with_source(
                                    err,
                                    format!("When parsing floating number {:?}", buffer),
                                )
                            })?;
                            stream.push(Token::Float(num));
                        }
                        buffer.clear();
                        status = Status::Normal;
                        false
                    }
                },
                Status::Str(is_escape) => {
                    if is_escape {
                        let c = match c {
                            'n' => '\n',
                            'r' => '\r',
                            't' => '\t',
                            _ => c,
                        };
                        buffer.push(c);
                        status = Status::Str(false);
                        true
                    } else {
                        match c {
                            '\\' => {
                                status = Status::Str(true);
                            }
                            '\"' => {
                                stream.push(Token::Str(std::mem::take(&mut buffer)));
                                status = Status::Normal;
                            }
                            _ => {
                                buffer.push(c);
                            }
                        }
                        true
                    }
                }
                Status::Word => match c {
                    'A'..='Z' | 'a'..='z' | '_' => {
                        buffer.push(c);
                        true
                    }
                    _ => {
                        match buffer.as_str() {
                            "true" => stream.push(Token::Bool(true)),
                            "false" => stream.push(Token::Bool(false)),
                            "null" => stream.push(Token::Null),
                            "stop" => stream.push(Token::Stop),
                            "import" => stream.push(Token::Import),
                            "include" => stream.push(Token::Include),
                            "ext" => stream.push(Token::Extern),
                            "map" => stream.push(Token::Map),
                            "fn" => stream.push(Token::Fn),
                            "move" => stream.push(Token::Move),
                            "acq" => stream.push(Token::Acq),
                            "return" => stream.push(Token::Return),
                            "then" => stream.push(Token::Then),
                            "else" => stream.push(Token::Else),
                            "repeat" => stream.push(Token::Repeat),
                            _ => {
                                stream.push(Token::Word(std::mem::take(&mut buffer)));
                            }
                        }
                        buffer.clear();
                        status = Status::Normal;
                        false
                    }
                },
            };

            if used {
                chars.next();
            }
        }

        Ok(Stream(stream))
    }
}
