use std;
use std::str::FromStr;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Key {
    Bind,
    Column,
    From,
    My,
    Root,
    Struct,
    Sys,
    Up,
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            Key::Bind => "bind",
            Key::Column => "column",
            Key::From => "from",
            Key::My => "my",
            Key::Root => "root",
            Key::Struct => "struct",
            Key::Sys => "sys",
            Key::Up => "up",
        })
    }
}

// token enum list
#[derive(Debug)]
pub enum Tok {
    // single char tokens
    CurlL,
    CurlR,

    // keywords (@)
    Key(Key),

    // literals
    Literal(String), // 'text'

    // whitespace, comments, pragmas
    Whitespace(String), //  space, newline, tab
    Comment(String), // # string\n
    // Pragma

    // oops
    Error(String),
}

impl fmt::Display for Tok {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Tok::CurlL => write!(f, "Curl Start"),
            Tok::CurlR => write!(f, "Curl End"),
            Tok::Key(ref k) => write!(f, "@{}", k),
            Tok::Literal(ref q) => write!(f, "q {}", q),
            Tok::Whitespace(ref w) => write!(f, "White {}", w.len()),
            Tok::Comment(ref c) => write!(f, "Comment {}", c),
            Tok::Error(ref e) => write!(f, "Error {}", e),
        }
    }
}

fn empty_literal() -> String {
    String::with_capacity(20)
}

fn empty_comment() -> String {
    String::with_capacity(20)
}

fn lex_white(first_char: char, chars: &mut std::str::Chars)
    -> (Option<char>, Tok) {
  let (oc, word) = read_to(chars, |c| !c.is_whitespace());
  (oc, Tok::Whitespace(first_char.to_string()+&word))
}

impl std::str::FromStr for Key {
    type Err = Tok;
    fn from_str(word: &str) -> Result<Key, Tok> {
        match word {
            "bind" => Ok(Key::Bind),
            "column" => Ok(Key::Column),
            "from" => Ok(Key::From),
            "my" => Ok(Key::My),
            "root" => Ok(Key::Root),
            "struct" => Ok(Key::Struct),
            "sys" => Ok(Key::Sys),
            "up" => Ok(Key::Up),
            _ => Err(Tok::Error(format!("Invalid key: @{}", word))),
        }
    }
}

fn at_tok(word: String) -> Tok {
    match Key::from_str(&word) {
        Ok(t) => Tok::Key(t),
        Err(t) => t,
    }
}

fn lex_at(chars: &mut std::str::Chars) -> (Option<char>, Tok) {
    let (oc, word) = read_to(chars, |c| !c.is_alphanumeric());
    (oc, at_tok(word))
}

// TODO: inline
fn lex_q(chars: &mut std::str::Chars) -> Tok {
    //println!("*lex_q*");
    lex_lit(false, true, '\'', chars)
}

fn lex_hash(chars: &mut std::str::Chars, pass_comment: bool) -> Option<Tok> {
    let mut n_hash = 1;
    while let Some(c) = chars.next() {
        //println!("*lex_hash c* {:?}",c);
        if c.is_whitespace() {
            if c == '\n' { // empty comment
                if 1 == n_hash {
                    return if pass_comment {
                        Some(Tok::Comment("".to_string()))
                    } else {
                        None
                    };
                } else {
                    // ....
                    return Some(Tok::Error("Empty multihash NYI".to_string()))
                }
            } else {
                if 1 == n_hash {
                    let tok = lex_comment_eol(chars);
                    return if pass_comment { Some(tok) } else { None };
                } else {
                    return Some(Tok::Error("multihash NYI".to_string()));
                }
            };
        } else if c == '#' {
            n_hash += 1;
        } else if c.is_alphanumeric() {
            return Some(Tok::Error("pragma NYI".to_string()));
        } else if "{([<`'\"|".contains(c) {
            return Some(Tok::Error("Inline comment NYI".to_string()));
        } else if "+-".contains(c) {
            return Some(Tok::Error("On/off pragma NYI".to_string()));
        } else {
            return Some(Tok::Error("Bad char after #".to_string()));
        }
    }
    None
}

fn lex_comment_eol(chars: &mut std::str::Chars) -> Tok {
    let mut word = empty_comment();
    while let Some(c) = chars.next() {
        if c == '\n' {
            return Tok::Comment(word.clone());
        } else {
            word = word + &c.to_string();
        }
    }
    Tok::Error("Comment unreachable".to_string())
}

fn lex_esc(chars: &mut std::str::Chars, digit_esc: bool, quote_esc: bool,
        tab_esc: bool) -> Option<char> {
    let nc = chars.next();
    //println!("*lex_esc nc* {:?}",nc);
    match nc {
        None => None,
        Some(c) => {
            //println!("*lex_esc c* {:?}", c);
            if c == '\\' {
                Some('\\')
            } else if digit_esc && c == '{' {
                Some('?') // FIXME
            } else if tab_esc && c == 't' {
                Some('\t')
            } else if tab_esc && c == 'n' {
                Some('\n')
            } else if tab_esc && c == '0' {
                Some('\0')
            } else if quote_esc && c == '"' {
                Some('\"')
            } else {
                None
            }
        }
    }
}

fn lex_lit(allow_newline: bool, allow_backslash: bool,
        qc: char, chars: &mut std::str::Chars) -> Tok {
    let mut lit = String::with_capacity(20);
    loop {
        let (oc, word) = read_to(chars, |c| {
            c == qc
            || (!allow_newline && c == '\n')
            || (allow_backslash && c == '\\')
        });
        lit = lit + &word;
        match oc {
            Some('"') | Some('\'') => return Tok::Literal(lit),
            Some('\n') => return Tok::Error("Newline in Literal".to_string()),
            None => return Tok::Error("EOF in Literal".to_string()),
            Some('\\') => {
                let xc = lex_esc(chars, true, true, true); // FIXME
                //println!("*lex_lit lex_esc xc oc word* {:?} {:?} {:?}", xc, oc, word);
                match xc {
                    Some(c) => lit.push(c),
                    None => return Tok::Error("Invalid backslash escape sequence in Literal".to_string()),
                };
                //println!("*lex_lit lit* {:?}", lit)
            },
            Some(_) => {
                return Tok::Error("Unexpected character in Literal".to_string())
            }
        }
    }
}

fn lex_lits(flags : String, qc: char, chars: &mut std::str::Chars) -> Tok {
    assert!(qc == '"' || qc == '\'');
    let allow_newline = flags == "n";
    let allow_backslash = qc == '"';

    if flags != "n" && flags != "" {
        return Tok::Error(format!("Invalid quote prefix: {}",flags))
    }
    lex_lit(allow_newline, allow_backslash, qc, chars)
}

fn lex_bare(first_char: char, chars: &mut std::str::Chars) -> (Option<char>, Tok) {
    let mut word = empty_literal();
    word.push(first_char);
    while let Some(c) = chars.next() {
        if c.is_alphanumeric() {
            word.push(c);
        } else if c == '"' || c == '\'' {
            return (None, lex_lits(word, c, chars));
            //return (None, lex_qq(word, chars));
        } else {
            return (Some(c), Tok::Literal(word));
        }
    }
    (None, Tok::Literal(word))
}

fn read_to<F>(chars: &mut std::str::Chars, is_end: F)
    -> (Option<char>, String)
    where F: Fn(char) -> bool {
  let mut word = empty_literal(); // ??
  while let Some(c) = chars.next() {
      //println!("*read_to c* {:?}",c);
      if is_end(c) {
          return (Some(c), word)
      } else {
          word.push(c)
      }
  }
  (None, word)
}

fn some_tok(pair: (Option<char>, Tok)) -> (Option<char>, Option<Tok>) {
    let (oc, tok) = pair;
    (oc, Some(tok))
}

fn lex_tok(first_char: Option<char>, chars: &mut std::str::Chars)
        -> (Option<char>, Option<Tok>) {
    let c = match first_char {
        Some(fc) => fc,
        None => match chars.next() {
            Some(nc) => nc,
            None => return (None, None), // do I need an EOF token?
        }
    };
    if c.is_whitespace() {
        some_tok(lex_white(c, chars))
    } else if c.is_alphanumeric() {
        some_tok(lex_bare(c, chars))
    } else if c == '{' {
        (None, Some(Tok::CurlL))
    } else if c == '}' {
        (None, Some(Tok::CurlR))
    } else if c == '\'' {
        //println!("*lex_tok {:?}",c);
        (None, Some(lex_q(chars)))
    } else if c == '@' {
        some_tok(lex_at(chars))
    } else if c == '#' {
        (None, lex_hash(chars, true))
    } else {
        (None,Some(Tok::Error(format!("Bad char: {}",c))))
    }
}

pub fn lex(text: String) -> Vec<Tok> {
    let mut tokens : Vec<Tok> = vec![];
    let mut chars = text.chars();
    let mut oc: Option<char> = None;
    loop {
        let (noc, otok) = lex_tok(oc, &mut chars);
        match otok {
            Some(tok) => tokens.push(tok),
            None => break,
        }
        oc = noc;
    }
    tokens
}

///////////////////////////////////////////////////////////// tests

#[cfg(test)]
fn slex(text: &str) -> Vec<Tok> {
    //println!("*slex text* {:?} {:?} {:?}",text.len(), text.to_string().len(),text);
    lex(text.to_string())
}

#[test]
fn test_empty_string() {
    let a = slex("");
    assert!(0 == a.len())
}

#[test]
fn test_white() {
    let w = "  \t  \n  ";
    let a = lex(w.to_string());
    assert!(1 == a.len());
    match a[0] {
        Tok::Whitespace(ref x) => assert!(x.clone() == w),
        _ => assert!(false),
    }
}

#[test]
fn test_bare() {
    let w = "abc123";
    let a = slex(w);
    assert!(1 == a.len());
    match a[0] {
        Tok::Literal(ref x) => assert!(x.clone() == w.to_string()),
        _ => assert!(false),
    }
}

#[test]
fn test_bare_q() {
    let w = "abc'xxx'";
    let a = slex(w);
    assert!(2 == a.len());
    match a[0] {
        Tok::Error(_) => {},
        _ => assert!(false)
    }
    match a[1] {
        Tok::Error(_) => {},
        _ => assert!(false)
    }
}

#[test]
fn test_err() {
    let w = "_";
    let a = lex(w.to_string());
    assert!(1 == a.len());
    match a[0] {
        Tok::Error(_) => {},
        _ => assert!(false)
    }
}

#[test]
fn test_q() {
    let a = lex("'abc'".to_string());
    assert!(1 == a.len());
    match a[0] {
        Tok::Literal(ref w) => assert!(w == "abc"),
        _ => assert!(false)
    }
}

#[test]
fn test_q_nonl() {
    let a = lex("'a\nc'".to_string());
    assert!(2 == a.len());
    match a[0] {
        Tok::Error(_) => {},
        _ => assert!(false),
    }
    match a[1] {
        Tok::Error(_) => {},
        _ => assert!(false),
    }
}

#[test]
fn test_q_escnl() {
    let a = slex("'a\\nb'");
    //println!("*q_noesc* {:?}",a);
    assert!(1 == a.len());
    match a[0] {
        Tok::Literal(ref w) => {
            assert!(w == "a\nb");
        },
        _ => assert!(false),
    }
}

#[test]
fn test_nq() {
    let a = slex("n'abc'");
    assert!(1 == a.len());
    match a[0] {
        Tok::Literal(ref w) => assert!(w == "abc"),
        _ => assert!(false)
    }
}

#[test]
fn test_nq_nl() {
    let a = slex("n'a\nc'");
    assert!(1 == a.len());
    match a[0] {
        Tok::Literal(ref w) => assert!(w == "a\nc"),
        _ => assert!(false),
    }
}

#[test]
fn test_nq_noesc() {
    let a = slex("n'a\\nb'");
    assert!(1 == a.len());
    match a[0] {
        Tok::Literal(ref w) => assert!(w == "a\\nb"),
        _ => assert!(false),
    }
}
