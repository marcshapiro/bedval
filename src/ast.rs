use lex;
use std::vec;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Literal(String),
    Column(Vec<Expr>), // KeyColumn LCurl <Expr>* RCurl
    Struct(Vec<Bind>), // KeyStruct LCurl <Bind>* RCurl
    KeyRoot,
    KeySys,
    KeyUp,
    KeyMy,
    From(Vec<Expr>), // KeyFrom <Expr> LCurl <Expr>_ RCurl
    Call{function:Box<Expr>, arguments:Vec<Bind>},
    Error(String),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Expr::Literal(ref s) => write!(f, "lit({})", s),
            Expr::Column(ref c) => write!(f, "col({:?})", c), // FIXME: without Debug
            Expr::Struct(ref s) => write!(f, "str...({:?})",s), // FIXME: recur
            Expr::KeyRoot => write!(f, "@Root"),
            Expr::KeySys => write!(f, "@Sys"),
            Expr::KeyUp => write!(f, "@Up"),
            Expr::KeyMy => write!(f, "@My"),
            Expr::From(ref v) => write!(f, "from({:?})", v), // FIXME: recur / no Debug
            Expr::Call{function: ref ftn, arguments: ref a} => write!(f, "call {:?} ( {:?} )", *ftn, a), // FIXME
            Expr::Error(ref s) => write!(f, "err({})", s),
        }
    }
}


#[derive(Debug, PartialEq)]
pub struct Bind { // KeyBind Literal LCurl Expr RCurl
    name: String,
    value: Expr,
}

impl fmt::Display for Bind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bind {} = {}", self.name, self.value)
    }
}


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

fn parse_bind(it: &mut vec::IntoIter<lex::Tok>) -> Result<Bind, String> {
    let ltok = non_gray(it);
    match ltok {
        Some(lex::Tok::Literal(name)) =>
            match non_gray(it) {
                Some(lex::Tok::CurlL) => {
                    let expr = parse_expr(None, it);
                    match non_gray(it) {
                        Some(lex::Tok::CurlR) => Ok(Bind{name:name, value:expr}),
                        _ => Err("@bind must end with '}'".to_string()),
                    }
                },
                _ => Err("@bind <name> must be followed by '{'".to_string()),
            },
        _ => Err("@bind must be followed by literal".to_string()),
    }
}

fn parse_binds(it: &mut vec::IntoIter<lex::Tok>) -> Result<(Vec<Bind>, Option<lex::Tok>), String> {
    let mut binds = vec![];
    let mut next_tok = non_gray(it);
    while let Some(lex::Tok::Key(lex::Key::Bind)) = next_tok {
        // FIXME: try!
        let rbind = parse_bind(it);
        match rbind {
            Ok(bind) => binds.push(bind),
            Err(s) => return Err(s)
        }
        next_tok = non_gray(it);
    }
    Ok((binds, next_tok))
}

fn parse_bind_list(it: &mut vec::IntoIter<lex::Tok>) -> Result<Vec<Bind>, String> {
    match non_gray(it) {
        Some(lex::Tok::CurlL) => {
            match parse_binds(it) {
                Ok((binds, end_tok)) =>
                    match end_tok {
                        Some(lex::Tok::CurlR) => Ok(binds),
                        _ => Err("bind list must end with '}'".to_string()),
                    },
                Err(e) => Err(e),
            }
        },
        _ => Err("bind list must start with '{'".to_string()),
    }
}

fn parse_expr(ofirst_tok: Option<lex::Tok>, it: &mut vec::IntoIter<lex::Tok>) -> Expr {
    let first_tok = match ofirst_tok {
        Some(ft) => ft,
        None => match non_gray(it) {
            Some(ft) => ft,
            None => return Expr::Error("Expected Expr, got EOF".to_string())
        }
    };
    match first_tok {
        lex::Tok::Key(lex::Key::Struct) =>
            match parse_bind_list(it) {
                Ok(binds) => Expr::Struct(binds),
                Err(e) => Expr::Error(e),
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
        lex::Tok::Key(lex::Key::From) => {
            let otok1 = non_gray(it);
            let head_expr = parse_expr(otok1, it);
            let otok2 = non_gray(it);
            match otok2 {
                Some(lex::Tok::CurlL) => {
                    let (tail_exprs, ttok) = parse_exprs(it);
                    match ttok {
                        Some(lex::Tok::CurlR) => {
                            let mut all_exprs = Vec::with_capacity(1+tail_exprs.len());
                            all_exprs.push(head_expr);
                            all_exprs.extend(tail_exprs);
                            Expr::From(all_exprs)
                        },
                        _ => Expr::Error("@from must end with '}'".to_string()),
                    }
                },
                _ => Expr::Error("@from must have '{' after root struct".to_string()),
            }
        },
        lex::Tok::Key(lex::Key::Call) => {
            let ftn = parse_expr(None, it);
            match parse_bind_list(it) {
                Ok(binds) => Expr::Call{function:Box::new(ftn), arguments:binds},
                Err(e) => Expr::Error(e),
            }
        },
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
                        //println!("parse_exprs literal/key: {:?}", t);
                        exprs.push(parse_expr(Some(t), it));
                    },
                    _ => {
                        //println!("parse_exprs other: {:?}", t);
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
fn is_error(expr: &Expr) -> bool {
    match expr {
        &Expr::Error(_) => true,
        _ => false,
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

#[test]
fn test_from_end() {
    let a = sparse("@from");
    assert!(is_error(&a));
}

#[test]
fn test_from_no_root() {
    let a = sparse("@from { @my }");
    assert!(is_error(&a));
}

#[test]
fn test_from_empty() {
    assert_eq!(sparse("@from @my { }"), Expr::From(vec![Expr::KeyMy]));
}

#[test]
fn test_from() {
    assert_eq!(sparse("@from @my { @my @my }"),
        Expr::From(vec![Expr::KeyMy, Expr::KeyMy, Expr::KeyMy]));
}

#[test]
fn test_struct_empty() {
    assert_eq!(sparse("@struct {}"), Expr::Struct(vec![]));
}

#[test]
fn test_struct_one() {
    assert_eq!(sparse("@struct { @bind a { @my }}"),
        Expr::Struct(vec![Bind{name:"a".to_string(), value: Expr::KeyMy}]))
}

#[test]
fn test_struct_two() {
    assert_eq!(
        sparse("@struct { @bind abc { @my } @bind bcd { @root }}"),
        Expr::Struct(vec![
                Bind{name:"abc".to_string(), value: Expr::KeyMy},
                Bind{name:"bcd".to_string(), value: Expr::KeyRoot},
            ])
        )
}

#[test]
fn test_call_empty() {
    assert_eq!(sparse("@call @my { }"),
        Expr::Call{
            function: Box::new(Expr::KeyMy),
            arguments: vec![]
        })
}

#[test]
fn test_call_args() {
    assert_eq!(sparse("@call @my { @bind x { @root } @bind y { @up } }"),
        Expr::Call{
            function: Box::new(Expr::KeyMy),
            arguments: vec![
                Bind{name:"x".to_string(), value: Expr::KeyRoot},
                Bind{name:"y".to_string(), value: Expr::KeyUp},
            ]
        })
}
