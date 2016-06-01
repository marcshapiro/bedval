use lex;
use std::vec;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Literal(String),
    Column(Vec<Expr>), // KeyColumn LCurl <Expr>* RCurl
//    Struct(Vec<Bind>), // KeyStruct LCurl <Bind>* RCurl
    KeyRoot,
    KeySys,
    KeyUp,
    KeyMy,
//    From(Vec<Expr>), // KeyFrom <Expr> LCurl <Expr>_ RCurl
    Error(String),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Expr::Literal(ref s) => write!(f, "lit({})", s),
            Expr::Column(ref c) => write!(f, "col({:?})", c), // FIXME: without Debug
//            Expr::Struct(ref s) => write!(f, "str..."), // FIXME: recur
            Expr::KeyRoot => write!(f, "@Root"),
            Expr::KeySys => write!(f, "@Sys"),
            Expr::KeyUp => write!(f, "@Up"),
            Expr::KeyMy => write!(f, "@My"),
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


pub fn parse(toks: Vec<lex::Tok>) -> Expr {
    let mut it = toks.into_iter();
    parse_expr(None, &mut it)
}

// skip whitespace and similar
pub fn non_gray(it: &mut vec::IntoIter<lex::Tok>) -> Option<lex::Tok> {
    let ot = it.next();
    match ot {
        Some(lex::Tok::Whitespace(_))
        | Some(lex::Tok::Comment(_))
        | Some(lex::Tok::Error(_))
            => non_gray(it),
        _ => ot
    }
}

fn parse_expr(ofirst_tok: Option<lex::Tok>, it: &mut vec::IntoIter<lex::Tok>) -> Expr {
    let first_tok = match ofirst_tok {
        Some(ft) => ft,
        None => match it.next() {
            Some(ft) => ft,
            None => return Expr::Error("Expected Expr, got EOF".to_string())
        }
    };
    match first_tok {
        lex::Tok::Key(lex::Key::Struct) => {
            // ...
            Expr::Error("Struct NYI".to_string())
        },
        lex::Tok::Key(lex::Key::From) => {
            // ...
            Expr::Error("From NYI".to_string())
        },
        lex::Tok::Key(lex::Key::Column) => {
            let otok = non_gray(it);
            match otok {
                Some(lex::Tok::CurlL) => {
                    let (exprs, ttok) = parse_exprs(it);
                    match ttok {
                        Some(lex::Tok::CurlR) => Expr::Column(exprs),
                        _ => Expr::Error("Column must end with '}'".to_string())
                    }
                },
                _ => Expr::Error("@Column must be followed by '{'".to_string())
            }
        },
        lex::Tok::Key(lex::Key::Root) => Expr::KeyRoot,
        lex::Tok::Key(lex::Key::Up) => Expr::KeyUp,
        lex::Tok::Key(lex::Key::Sys) => Expr::KeySys,
        lex::Tok::Key(lex::Key::My) => Expr::KeyMy,
        lex::Tok::Literal(s) => Expr::Literal(s),
        _ => Expr::Error("Unexpected token".to_string())
    }
}

fn parse_exprs(it: &mut vec::IntoIter<lex::Tok>) -> (Vec<Expr>, Option<lex::Tok>) {
    let mut exprs: Vec<Expr> = vec![];
    loop {
        let otok = non_gray(it);
        match otok {
            Some(t) => {
                match t {
                    lex::Tok::Key(lex::Key::Struct)
                    | lex::Tok::Key(lex::Key::From)
                    | lex::Tok::Key(lex::Key::Column)
                    | lex::Tok::Key(lex::Key::My)
                    | lex::Tok::Literal(_) => {
                        println!("parse_exprs literal/key: {:?}", t);
                        exprs.push(parse_expr(Some(t), it));
                    },
                    _ => {
                        println!("parse_exprs other: {:?}", t);
                        return (exprs, Some(t))
                    },
                }
            },
            None => {
                exprs.push(Expr::Error("Expected Expr, got EOF".to_string()));
                return (exprs, None);
            }
        }
    }
}

#[cfg(test)]
fn assert_no_lex_errors(toks: &Vec<lex::Tok>) {
    for tok in toks {
        match *tok {
            lex::Tok::Error(_) => assert!(false),
            _ => {}
        }
    }
}

#[cfg(test)]
fn sparse(s: &str) -> Expr {
    let toks = lex::slex(s);
    assert_no_lex_errors(&toks);
    parse(toks)
}

#[test]
fn test_literal() {
    assert_eq!(sparse("'abc'"), Expr::Literal("abc".to_string()));
}

#[test]
fn test_key_my() {
    assert_eq!(sparse("@my"), Expr::KeyMy);
}

#[test]
fn test_column_empty() {
    assert_eq!(sparse("@column { }"), Expr::Column(vec![]));
}

#[test]
fn test_column_some() {
    assert_eq!(sparse("@column { @my @my @my }"),
        Expr::Column(vec![Expr::KeyMy, Expr::KeyMy, Expr::KeyMy]));
}
