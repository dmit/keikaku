mod prim;

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

use crate::lexer::Span;
use crate::parser::Ast;

#[derive(Debug)]
pub enum EvalError {
    DivisionByZero(Span),
    EmptyDef(Span),
    NotAFunction(Span, Symbol),
    InvalidType(Span), //TODO: include expected and actual type info
    UnknownDef(Span, Symbol),
    WrongArity(Span), //TODO: include expected arity info
}

impl Display for EvalError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::EvalError::*;
        match self {
            DivisionByZero(span) => write!(f, "division by zero at {}", span),
            EmptyDef(span) => write!(f, "empty def at {}", span),
            InvalidType(span) => write!(f, "unexpected argument type at {}", span),
            NotAFunction(span, sym) => write!(f, "`{}` is not a function at {}", sym.0, span),
            UnknownDef(span, sym) => write!(f, "undefined symbol: `{}` at {}", sym.0, span),
            WrongArity(span) => write!(f, "wrong arity at {}", span),
        }
    }
}

impl Error for EvalError {}

#[derive(Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Symbol(String);

#[derive(Clone)]
pub enum Object {
    Int(i128),
    Lambda(Vec<Symbol>, Vec<Ast>),
    Nil,
    PrimOp(prim::Prim),
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Object::Int(n) => write!(f, "{}", n),
            //Object::Lambda(args, _) => write!(f, "#lambda({})#", args.iter().map(|arg| arg.0.clone()).collect::<Vec<String>>().join(" ")),
            Object::Lambda(args, _) => {
                write!(f, "#lambda(")?;
                for (i, arg) in args.iter().enumerate() {
                    write!(f, "{}", arg.0)?;
                    if i != args.len() - 1 {
                        write!(f, " ")?;
                    }
                }
                write!(f, ")#")
            }
            Object::Nil => write!(f, "()"),
            Object::PrimOp(_) => write!(f, "#primop#"),
        }
    }
}

impl Debug for Object {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        (self as &Display).fmt(f)
    }
}

#[derive(Debug)]
pub struct Env {
    defines: HashMap<Symbol, Object>,
}

impl Default for Env {
    fn default() -> Self {
        let mut defines = HashMap::new();
        defines.insert(Symbol("+".to_string()), Object::PrimOp(&prim::PRIM_ADD));
        defines.insert(Symbol("*".to_string()), Object::PrimOp(&prim::PRIM_MUL));
        defines.insert(Symbol("-".to_string()), Object::PrimOp(&prim::PRIM_SUB));
        defines.insert(Symbol("/".to_string()), Object::PrimOp(&prim::PRIM_DIV));
        Env { defines }
    }
}

pub struct Eval<'a> {
    env: &'a mut Env,
}

impl<'a> Eval<'a> {
    pub fn new(env: &mut Env) -> Eval {
        Eval { env }
    }

    pub fn eval(&mut self, ast: &Ast, span: Span) -> Result<Object, EvalError> {
        match ast {
            Ast::List(children) => {
                match children.as_slice() {
                    [] => Ok(Object::Nil),
                    [(Ast::Def, span)] => Err(EvalError::EmptyDef(*span)),
                    [(Ast::Def, _), (Ast::Symbol(sym), _), (expr, span)] => {
                        let key = Symbol(sym.clone());
                        let val = self.eval(expr, *span)?;
                        self.env.defines.insert(key, val);
                        Ok(Object::Nil)
                    }
                    [(Ast::Symbol(sym), span)] => {
                        let sym = Symbol(sym.clone());
                        match self.env.defines.get(&sym) {
                            Some(f) => unimplemented!(),
                            None => Err(EvalError::UnknownDef(*span, sym)),
                        }
                    }
                    [(Ast::Symbol(sym), span), args..] => {
                        let sym = Symbol(sym.clone());
                        match self.env.defines.get(&sym) {
                            Some(Object::Lambda(args, body)) => unimplemented!(),
                            Some(Object::PrimOp(p)) => {
                                let p = p.clone(); //FIXME can we avoid this clone?
                                let mut evaluated_args = Vec::new();
                                for (ast, span) in args {
                                    let res = self.eval(ast, *span)?;
                                    evaluated_args.push((res, *span));
                                }
                                p(*span, evaluated_args.as_slice())
                            }
                            Some(_) => Err(EvalError::NotAFunction(*span, sym)),
                            None => Err(EvalError::UnknownDef(*span, sym)),
                        }
                    }
                    other => unimplemented!("don't know how to eval {:?}", other),
                }
            }
            Ast::Def => Err(EvalError::EmptyDef(span)),
            Ast::Int(n) => Ok(Object::Int(*n)),
            Ast::Symbol(sym) => {
                let sym = Symbol(sym.clone());
                match self.env.defines.get_mut(&sym) {
                    Some(obj) => Ok(obj.clone()),
                    None => Err(EvalError::UnknownDef(span, sym)),
                }
            }
        }
    }
}
