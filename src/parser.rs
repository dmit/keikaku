use std::iter::Peekable;

use crate::lexer::{Span, Token};

#[derive(Debug)]
pub enum ParseError {
    InvalidNumberFormat(Span, String),
    MismatchedClosingBrace(Span),
    UnexpectedEndOfInput,
}

#[derive(Debug)]
pub enum Ast {
    Int(i128),
    List(Vec<Ast>),
    Nil,
    Symbol(String),
}

impl Ast {
    fn parse_expr<I>(it: &mut Peekable<I>) -> Result<Ast, ParseError>
    where
        I: Iterator<Item = (Token, Span)>,
    {
        match it.next() {
            None => Err(ParseError::UnexpectedEndOfInput),

            Some((t, span)) => match t {
                Token::OpeningBrace => match it.peek() {
                    None => Err(ParseError::UnexpectedEndOfInput),

                    Some((Token::ClosingBrace, _)) => {
                        it.next();
                        Ok(Ast::Nil)
                    }

                    Some(_) => {
                        let mut args = Vec::new();
                        while it.peek().map(|(t, _)| t != &Token::ClosingBrace) == Some(true) {
                            let v = Ast::parse_expr(it)?;
                            args.push(v);
                        }

                        // consume the closing brace
                        match it.next() {
                            None => Err(ParseError::UnexpectedEndOfInput),
                            _ => Ok(Ast::List(args)),
                        }
                    }
                },

                Token::ClosingBrace => Err(ParseError::MismatchedClosingBrace(span)),

                Token::Ident(s) => Ok(Ast::Symbol(s)),

                Token::Num(n) => n
                    .parse::<i128>()
                    .map(Ast::Int)
                    .map_err(|e| ParseError::InvalidNumberFormat(span, e.to_string())),
            },
        }
    }

    pub fn parse<I>(it: &mut Peekable<I>) -> Result<Vec<Ast>, ParseError>
    where
        I: Iterator<Item = (Token, Span)>,
    {
        let mut exprs = Vec::new();
        while it.peek().is_some() {
            let expr = Ast::parse_expr(it)?;
            exprs.push(expr);
        }
        Ok(exprs)
    }
}
