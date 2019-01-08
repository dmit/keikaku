use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::iter::Peekable;

use crate::lexer::{Span, Token};

#[derive(Debug)]
pub enum ParseError {
    InvalidNumberFormat(Span, String),
    MismatchedClosingBrace(Span),
    UnexpectedEndOfInput,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::ParseError::*;
        match self {
            InvalidNumberFormat(span, msg) => {
                write!(f, "invalid number format: {} at {}", msg, span)
            }
            MismatchedClosingBrace(span) => write!(f, "mismatched closing brace at {}", span),
            UnexpectedEndOfInput => write!(f, "unexpected end of input"),
        }
    }
}

impl Error for ParseError {}

#[derive(Clone, Debug)]
pub enum Ast {
    Def,
    Int(i128),
    List(Vec<(Ast, Span)>),
    Symbol(String),
}

impl Ast {
    fn parse_expr<I>(it: &mut Peekable<I>) -> Result<(Ast, Span), ParseError>
    where
        I: Iterator<Item = (Token, Span)>,
    {
        match it.next() {
            None => Err(ParseError::UnexpectedEndOfInput),

            Some((t, span)) => match t {
                Token::OpeningBrace => match it.peek() {
                    None => Err(ParseError::UnexpectedEndOfInput),

                    Some(_) => {
                        let mut args = Vec::new();
                        while it.peek().map(|(t, _)| t != &Token::ClosingBrace) == Some(true) {
                            let v = Ast::parse_expr(it)?;
                            args.push(v);
                        }

                        // consume the closing brace
                        match it.next() {
                            None => Err(ParseError::UnexpectedEndOfInput),
                            _ => Ok((Ast::List(args), span)),
                        }
                    }
                },

                Token::ClosingBrace => Err(ParseError::MismatchedClosingBrace(span)),

                Token::Ident(s) => {
                    if s == "def" {
                        Ok((Ast::Def, span))
                    } else {
                        Ok((Ast::Symbol(s), span))
                    }
                }

                Token::Num(n) => n
                    .parse::<i128>()
                    .map(|i| (Ast::Int(i), span))
                    .map_err(|e| ParseError::InvalidNumberFormat(span, e.to_string())),
            },
        }
    }

    //TODO: return a lazy iterator instead?
    pub fn parse<I>(it: &mut Peekable<I>) -> Result<Vec<(Ast, Span)>, ParseError>
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
