use super::*;

type LineRef = Cell<usize>;

impl Slice {
    fn make_suffix(line: &LineRef, slice: VecDeque<Slice>) -> VecDeque<Slice> {
        let mut ops: VecDeque<Token> = VecDeque::new();
        let mut result = VecDeque::new();
        let mut call_flag = false;

        let mut op_stack = VecDeque::new();
        let mut iter = slice.into_iter();

        let mut get_next = |op_stack: &mut VecDeque<Slice>| {
            if op_stack.is_empty() {
                iter.next()
            } else {
                Some(op_stack.pop_front().unwrap())
            }
        };

        while let Some(slice) = get_next(&mut op_stack) {
            let slice = match slice {
                Slice::Token(token) => {
                    let priority = token.priority();

                    if priority == 0 {
                        Some(Slice::Token(token))
                    } else {
                        call_flag = false;

                        match token.attr().unwrap() {
                            Operator::Left => {
                                while ops.back().map_or(false, |op| op.priority() <= priority) {
                                    result.push_back(Slice::Token(ops.pop_back().unwrap()));
                                }
                                ops.push_back(token);
                            }
                            Operator::Right => {
                                while ops.back().map_or(false, |op| op.priority() < priority) {
                                    result.push_back(Slice::Token(ops.pop_back().unwrap()));
                                }
                                ops.push_back(token);
                            }
                            Operator::Unary => {
                                while ops.back().map_or(false, |op| op.priority() <= priority) {
                                    result.push_back(Slice::Token(ops.pop_back().unwrap()));
                                }
                                ops.push_back(token);
                            }
                        }

                        None
                    }
                }
                Slice::End(num) => {
                    line.set(num);
                    result.push_back(Slice::End(num));
                    None
                }
                _ => Some(slice),
            };

            if let Some(slice) = slice {
                if call_flag {
                    op_stack.push_back(Slice::Token(Token::Call));
                    op_stack.push_back(slice);
                } else {
                    call_flag = true;
                    result.push_back(slice);
                }
            }
        }

        while let Some(op) = ops.pop_back() {
            result.push_back(Slice::Token(op));
        }

        result
    }

    fn cook_line_ops(slice: VecDeque<Slice>, line: &LineRef) -> Result<Stmt> {
        let slice = Self::make_suffix(line, slice);
        let mut nodes = VecDeque::new();

        for slice in slice {
            match slice {
                Slice::Token(token) => match token.attr() {
                    Some(Operator::Unary) => {
                        let operand = nodes.pop_back().ok_or_else(|| {
                            Error::new(
                                format!("Missing operand for unary operator {:?}", token),
                                line.get(),
                            )
                        })?;
                        nodes.push_back(token.to_stmt_unary_fn()(Box::new(operand)));
                    }
                    Some(_) => {
                        let right = nodes.pop_back().ok_or_else(|| {
                            Error::new(
                                format!("Missing operand for binary operator {:?}", token),
                                line.get(),
                            )
                        })?;
                        let left = nodes.pop_back().ok_or_else(|| {
                            Error::new(
                                format!("Missing operand for binary operator {:?}", token),
                                line.get(),
                            )
                        })?;
                        nodes.push_back(token.to_stmt_fn()(Box::new(left), Box::new(right)));
                    }
                    None => {
                        nodes.push_back(Stmt::Token(token, line.get()));
                    }
                },

                Slice::End(num) => {
                    line.set(num);
                }
                _ => nodes.push_back(slice.cook(line)?),
            }
        }

        match nodes.len() {
            1 => Ok(nodes.pop_back().unwrap()),
            _ => Err(Error::new(
                format!(
                    "Incorrect number({}) of nodes are found. Is there an operator unclosed?",
                    nodes.len(),
                ),
                line.get(),
            )),
        }
    }

    fn cook(self, line: &LineRef) -> Result<Stmt> {
        match self {
            Slice::Token(token) => Ok(Stmt::Token(token, line.get())),
            Slice::Line(slice) => Self::cook_line_ops(slice, line),
            Slice::Block(slice) => {
                if slice.is_empty() {
                    Ok(Stmt::Empty)
                } else {
                    Ok(Stmt::Block(
                        slice
                            .into_iter()
                            .map(|slice| slice.cook(line))
                            .collect::<Result<VecDeque<_>>>()?,
                    ))
                }
            }
            Slice::End(num) => Err(Error::new("Unexpected end token".to_string(), num)),
        }
    }
}

impl Sliced {
    pub fn cook(self) -> Result<Cooked> {
        let line = Cell::new(1);
        self.0.cook(&line).map(Cooked)
    }
}
