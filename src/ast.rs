
//mod lex;

extern crate core;

use self::core::slice;
use lex;
//use std;
use std::fmt;

#[derive(Debug)]
pub enum Expr {
    Literal(String),
    Column(Vec<Expr>), // KeyColumn LCurl <Expr>* RCurl
//    Struct(Vec<Bind>), // KeyStruct LCurl <Bind>* RCurl
//    KeyRoot,
//    KeySys,
//    KeyUp,
//    KeyMy,
//    From(Vec<Expr>), // KeyFrom <Expr> LCurl <Expr>_ RCurl
    Error(String),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Expr::Literal(ref s) => write!(f, "lit({})", s),
            Expr::Column(ref c) => write!(f, "col({:?})", c), // FIXME: without Debug
//            Expr::Struct(ref s) => write!(f, "str..."), // FIXME: recur
//            Expr::KeyRoot => write!(f, "@Root"),
//            Expr::KeySys => write!(f, "@Sys"),
//            Expr::KeyUp => write!(f, "@Up"),
//            Expr::KeyMy => write!(f, "@My"),
//            Expr::From(ref v) => write!(f, "from..."), // FIXME: recur
            Expr::Error(ref s) => write!(f, "err({})", s),
        }
    }
}

/*
#[derive(Debug)]
pub struct Bind { // KeyBind Literal LCurl Expr RCurl
    name: String,
    value: Expr,
}

impl fmt::Display for Bind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bind {} = {}", self.name, self.value)
    }
}
*/


pub fn parse(toks: &Vec<lex::BvToken>) -> Expr {
    let mut it = toks.iter();
    parse_expr(None, &mut it)
}

fn parse_expr(ofirst_tok: Option<&lex::BvToken>, it: &mut slice::Iter<lex::BvToken>) -> Expr {
    let first_tok = match ofirst_tok {
        Some(ft) => ft,
        None => match it.next() {
            Some(ft) => ft,
            None => return Expr::Error("Expected Expr, got EOF".to_string())
        }
    };
    match first_tok.value {
        lex::BvTokE::KeyStruct => {
            // ...
            Expr::Error("Struct NYI".to_string())
        },
        lex::BvTokE::KeyFrom => {
            // ...
            Expr::Error("From NYI".to_string())
        },
        lex::BvTokE::KeyColumn => {
            let otok = it.next();
            match otok {
                Some(& lex::BvToken { value: lex::BvTokE::CurlL }) => {
                    let (exprs, ttok) = parse_exprs(it);
                    match ttok {
                        Some(& lex::BvToken { value: lex::BvTokE::CurlR }) => Expr::Column(exprs),
                        _ => Expr::Error("Column must end with '}'".to_string())
                    }
                },
                _ => Expr::Error("@Column must be followed by '{'".to_string())
            }
        },
//        lex::BvTokE::KeyRoot => Expr::KeyRoot,
        lex::BvTokE::Literal(ref s) => Expr::Literal(s.clone()),
        _ => Expr::Error("Unexpected token".to_string())
    }
}

fn parse_exprs<'a>(it: &mut slice::Iter<'a, lex::BvToken>) -> (Vec<Expr>, Option<&'a lex::BvToken>) {
    let mut exprs: Vec<Expr> = vec![];
    loop {
        let otok = it.next();
        match otok {
            Some(t) => match t.value {
                lex::BvTokE::KeyStruct | lex::BvTokE::KeyFrom | lex::BvTokE::KeyColumn |
                lex::BvTokE::Literal(_) => {
                    exprs.push(parse_expr(Some(t), it));
                },
                _ => {
                    return (exprs, Some(t))
                },
            },
            None => {
                exprs.push(Expr::Error("Expected Expr, got EOF".to_string()));
                return (exprs, None)
            }
        }
    }
}
