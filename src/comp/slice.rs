use super::*;

impl Slice {
    pub fn new_empty() -> Self {
        Self::Block(VecDeque::new())
    }

    fn is_empty_or_end(&self) -> bool {
        match self {
            Self::End(_) => true,
            _ => self.is_empty(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::End(_) | Self::Token(_) => false,
            Self::Line(slice) => slice.iter().all(|slice| slice.is_empty_or_end()),
            Self::Block(slice) => slice.iter().all(|slice| slice.is_empty_or_end()),
        }
    }
}

impl Stream {
    fn slice_with(slice: &mut [Token], is_paren: bool, is_block: bool) -> (Slice, usize) {
        if slice.is_empty() {
            return (Slice::new_empty(), 0);
        }
        if slice.len() == 1 {
            return match &slice[0] {
                Token::Enter(_, _) | Token::End(_) => return (Slice::new_empty(), 1),
                _ => (
                    Slice::Token(std::mem::replace(&mut slice[0], Token::Null)),
                    1,
                ),
            };
        }
        if !is_block {
            match slice.first_mut().unwrap() {
                Token::Enter(len, enclosing) => {
                    let len = *len;
                    match enclosing {
                        Enclosing::Paren => {
                            let result = Self::slice_with(&mut slice[1..len], true, true);
                            return (result.0, len);
                        }
                        Enclosing::Bracket => {
                            panic!("Brackets [] are not currently supported!");
                        }
                        Enclosing::Brace => {
                            let mut pos = 1;
                            let mut result = VecDeque::new();
                            'outer: while pos < len {
                                let mut use_end_token = 0;

                                while let Token::End(_) = slice[pos] {
                                    pos += 1;
                                    use_end_token = 1;
                                    if pos >= len {
                                        break 'outer;
                                    }
                                }
                                let (s, p) = Self::slice_with(
                                    &mut slice[pos - use_end_token..len],
                                    false,
                                    true,
                                );
                                if !s.is_empty() {
                                    result.push_back(s);
                                }
                                pos += p;
                            }
                            return match result.len() {
                                1 => (result.pop_front().unwrap(), len),
                                _ => (Slice::Block(result), len),
                            };
                        }
                    }
                }
                Token::End(_) => {}
                _ => {}
            }
        }

        let mut result = VecDeque::new();
        let mut pos = 0;
        result.push_back(VecDeque::new());
        while pos < slice.len() {
            let next = match slice[pos] {
                Token::End(line) => {
                    if !is_paren && !result.back().unwrap().is_empty() {
                        result.push_back(VecDeque::new());
                    }
                    result.back_mut().unwrap().push_back(Slice::End(line));
                    true
                }
                Token::Enter(len, _) => {
                    let (s, p) = Self::slice_with(&mut slice[pos..pos + len], false, false);
                    result.back_mut().unwrap().push_back(s);
                    pos += p;
                    false
                }
                _ => {
                    result
                        .back_mut()
                        .unwrap()
                        .push_back(Slice::Token(std::mem::replace(
                            &mut slice[pos],
                            Token::Null,
                        )));
                    true
                }
            };
            if next {
                pos += 1;
            }
        }

        while !result.is_empty() {
            if result.back().unwrap().is_empty() {
                result.pop_back();
            } else {
                break;
            }
        }

        let mut result: VecDeque<_> = result
            .into_iter()
            .map(Slice::Line)
            .filter(|slice| !slice.is_empty())
            .collect();

        match result.len() {
            1 => (result.pop_front().unwrap(), slice.len()),
            _ => (Slice::Block(result), slice.len()),
        }
    }

    pub fn slice(mut self) -> Sliced {
        let (sliced, pos) = Self::slice_with(&mut self.0, false, true);
        assert_eq!(pos, self.0.len());
        Sliced(sliced)
    }
}
