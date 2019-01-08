use super::{EvalError, Object};
use crate::lexer::Span;

type In = [(Object, Span)];
type Out = Result<Object, EvalError>;
pub type Prim = &'static (Fn(Span, &In) -> Out + Sync);

pub static PRIM_ADD: Prim = &|_, args| {
    let mut acc = 0;
    for (a, span) in args {
        match *a {
            Object::Int(n) => acc += n,
            _ => return Err(EvalError::InvalidType(*span)),
        }
    }
    Ok(Object::Int(acc))
};

pub static PRIM_SUB: Prim = &|span, args| match args {
    &[] => Err(EvalError::WrongArity(span)),
    &[(Object::Int(n), _)] => Ok(Object::Int(-n)),
    &[(_, span)] => Err(EvalError::InvalidType(span)),
    args => {
        let mut acc = 0;
        for (a, span) in args {
            match *a {
                Object::Int(n) => acc -= n,
                _ => return Err(EvalError::InvalidType(*span)),
            }
        }
        Ok(Object::Int(acc))
    }
};

pub static PRIM_MUL: Prim = &|_, args| {
    let mut acc = 1;
    for (a, span) in args {
        match *a {
            Object::Int(n) => acc *= n,
            _ => return Err(EvalError::InvalidType(*span)),
        }
    }
    Ok(Object::Int(acc))
};

pub static PRIM_DIV: Prim = &|span, args| match args.split_first() {
    None => Err(EvalError::WrongArity(span)),
    Some(args) => match args {
        ((Object::Int(_), _), &[]) => Err(EvalError::WrongArity(span)),
        ((Object::Int(numerator), _), denominators) => {
            let mut denominator_acc = 0;
            for (a, span) in denominators {
                match *a {
                    Object::Int(0) => return Err(EvalError::DivisionByZero(*span)),
                    Object::Int(n) => denominator_acc += n,
                    _ => return Err(EvalError::InvalidType(*span)),
                }
            }
            Ok(Object::Int(numerator / denominator_acc))
        }
        ((_, span), _) => Err(EvalError::InvalidType(*span)),
    },
};
