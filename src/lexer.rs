#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Position {
    line: usize,
    col: usize,
}

impl Position {
    fn to(self, other: Position) -> Span {
        Span {
            from: self,
            to: other,
        }
    }

    pub fn as_string(&self) -> String {
        format!("{}:{}", self.line + 1, self.col)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Span {
    from: Position,
    to: Position,
}

pub struct Source<'a, I>
where
    I: Iterator<Item = char>,
{
    pub name: String,
    chars: &'a mut I,
    peeked: Option<(Option<char>, Position, Position)>,
    cur_pos: Position,
    prev_pos: Position,
}

impl<'a, I> Source<'a, I>
where
    I: Iterator<Item = char>,
{
    pub fn new(name: String, chars: &'a mut I) -> Source<'a, I> {
        Source {
            name,
            chars,
            peeked: None,
            cur_pos: Position::default(),
            prev_pos: Position::default(),
        }
    }

    fn incr_col(&mut self) {
        self.prev_pos = self.cur_pos;
        self.cur_pos.col += 1;
    }

    fn incr_line(&mut self) {
        self.prev_pos = self.cur_pos;
        self.cur_pos.line += 1;
        self.cur_pos.col = 0;
    }

    fn current_pos(&self) -> Position {
        match self.peeked {
            Some((_, pos, _)) => pos,
            None => self.cur_pos,
        }
    }

    fn previous_pos(&self) -> Position {
        match self.peeked {
            Some((_, _, pos)) => pos,
            None => self.prev_pos,
        }
    }

    fn peek(&mut self) -> Option<<Self as Iterator>::Item> {
        match self.peeked {
            None => {
                let cur_pos = self.current_pos();
                let prev_pos = self.previous_pos();
                let next = self.next();
                self.peeked = Some((next, cur_pos, prev_pos));
                next
            }
            Some((p, _, _)) => p,
        }
    }

    pub fn tokenize(self) -> Tokens<'a, I> {
        Tokens(self)
    }
}

impl<'a, I> Iterator for Source<'a, I>
where
    I: Iterator<Item = char>,
{
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.peeked.take() {
            Some((item, _, _)) => item,
            None => {
                let next = self.chars.next();
                match next {
                    Some('\n') => self.incr_line(),
                    Some(_) => self.incr_col(),
                    None => (),
                }
                next
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
    OpeningBrace,
    ClosingBrace,
    Ident(String),
    Num(String),
}

pub struct Tokens<'a, I>(Source<'a, I>)
where
    I: Iterator<Item = char>;

impl<'a, I> Tokens<'a, I>
where
    I: Iterator<Item = char>,
{
    fn discard_while<F>(&mut self, predicate: F)
    where
        F: Fn(char) -> bool + Copy,
    {
        while self.0.peek().map(predicate) == Some(true) {
            self.0.next();
        }
    }

    fn read_while<F>(&mut self, predicate: F, target: &mut String)
    where
        F: Fn(char) -> bool + Copy,
    {
        while self.0.peek().map(predicate) == Some(true) {
            let ch = self.0.next().unwrap();
            target.push(ch);
        }
    }
}

impl<'a, I> Iterator for Tokens<'a, I>
where
    I: Iterator<Item = char>,
{
    type Item = (Token, Span);

    fn next(&mut self) -> Option<Self::Item> {
        self.discard_while(char::is_whitespace);

        let start_pos = self.0.current_pos();
        match self.0.next() {
            None => None,
            Some(ch) => {
                let next = self.0.peek();
                let tok = match (ch, next) {
                    ('(', _) => Token::OpeningBrace,

                    (')', _) => Token::ClosingBrace,

                    ('-', Some(ch)) if ch.is_ascii_digit() => {
                        let mut str = String::new();
                        str.push('-');
                        self.read_while(
                            |ch: char| ch != '(' && ch != ')' && !ch.is_whitespace(),
                            &mut str,
                        );

                        Token::Num(str)
                    }

                    (ch, _) if ch.is_ascii_digit() => {
                        let mut str = String::new();
                        str.push(ch);
                        self.read_while(
                            |ch: char| ch != '(' && ch != ')' && !ch.is_whitespace(),
                            &mut str,
                        );

                        Token::Num(str)
                    }

                    (ch, _) => {
                        let mut str = String::new();
                        str.push(ch);
                        self.read_while(
                            |ch: char| ch != '(' && ch != ')' && !ch.is_whitespace(),
                            &mut str,
                        );

                        Token::Ident(str)
                    }
                };

                let span = start_pos.to(self.0.previous_pos());
                Some((tok, span))
            }
        }
    }
}
