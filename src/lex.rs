use std;
use std::str::FromStr;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum Key {
    Bind,
    Call,
    Column,
    From,
    My,
    Root,
    Struct,
    Sys,
    Up,
}

#[test]
fn test_peq_bind() {
    assert!(Key::Bind == Key::Bind);
    assert!(Key::Column == Key::Column);
    assert!(Key::Bind != Key::Column);
}

#[test]
fn text_peq_vec_key() {
    assert!(vec![Key::Bind] == vec![Key::Bind])
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            Key::Bind => "bind",
            Key::Call => "call",
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
#[derive(Debug, PartialEq, Eq)]
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

#[test]
fn test_peq_tok() {
    assert!(Tok::CurlL == Tok::CurlL);
    assert!(Tok::CurlL != Tok::CurlR);
    assert!(Tok::CurlR == Tok::CurlR);
}

#[test]
fn test_peq_tok_key() {
    assert!(Tok::Key(Key::Bind) == Tok::Key(Key::Bind));
    assert!(Tok::Key(Key::My) != Tok::Key(Key::Bind));
}

#[test]
fn test_peq_tok_lit() {
    assert!(Tok::Literal("aaa".to_string()) == Tok::Literal("aaa".to_string()));
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
            "call" => Ok(Key::Call),
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

fn lex_hash(chars: &mut std::str::Chars) -> Tok {
    let (_, word) = read_to(chars, |c| c == '\n');
    return Tok::Comment(word);
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
  let mut word = empty_literal();
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
        (None, Some(lex_lit(false, true, '\'', chars)))
    } else if c == '@' {
        some_tok(lex_at(chars))
    } else if c == '#' {
        (None, Some(lex_hash(chars)))
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
pub fn slex(text: &str) -> Vec<Tok> {
    lex(text.to_string())
}

#[test]
fn test_empty_string() {
    let a = slex("");
    assert_eq!(0, a.len())
}

#[test]
fn test_white() {
    let w = "  \t  \n  ";
    assert_eq!(slex(w), vec![Tok::Whitespace(w.to_string())]);
}

#[test]
fn test_bare() {
    let w = "abc123";
    assert_eq!(slex(w), vec![Tok::Literal(w.to_string())]);
}

#[cfg(test)]
fn is_error(t: &Tok) -> bool {
    match t {
        &Tok::Error(_) => true,
        _ => false,
    }
}

#[cfg(test)]
fn vec_err(vt: &Vec<Tok>) -> usize {
    for t in vt {
        assert!(is_error(t))
    }
    vt.len()
}

#[test]
fn test_bare_q() {
    let a = slex("abc'xxx'");
    assert!(2 == vec_err(&a));
}

#[test]
fn test_err() {
    let a = slex("_");
    assert!(1 == vec_err(&a));
}

#[test]
fn test_q() {
    assert_eq!(slex("'abc'"), vec![Tok::Literal("abc".to_string())]);
}

#[test]
fn test_q_nonl() {
    let a = slex("'a\nc'");
    assert!(2 == vec_err(&a));
}

#[test]
fn test_q_escnl() {
    assert_eq!(slex("'a\\nb'"), vec![Tok::Literal("a\nb".to_string())]);
}

#[test]
fn test_nq() {
    assert_eq!(slex("n'abc'"), vec![Tok::Literal("abc".to_string())]);
}

#[test]
fn test_nq_nl() {
    assert_eq!(slex("n'a\nc'"), vec![Tok::Literal("a\nc".to_string())]);
}

#[test]
fn test_nq_noesc() {
    assert_eq!(slex("n'a\\nb'"), vec![Tok::Literal("a\\nb".to_string())]);
}

#[test]
fn test_key_bind() {
    assert_eq!(slex("@bind"), vec![Tok::Key(Key::Bind)]);
}

#[test]
fn test_key_column() {
    assert_eq!(slex("@column"), vec![Tok::Key(Key::Column)]);
}

#[test]
fn test_key_from() {
    assert_eq!(slex("@from"), vec![Tok::Key(Key::From)]);
}

#[test]
fn test_key_my() {
    assert_eq!(slex("@my"), vec![Tok::Key(Key::My)]);
}

#[test]
fn test_key_root() {
    assert_eq!(slex("@root"), vec![Tok::Key(Key::Root)]);
}

#[test]
fn test_key_struct() {
    assert_eq!(slex("@struct"), vec![Tok::Key(Key::Struct)]);
}

#[test]
fn test_key_sys() {
    assert_eq!(slex("@sys"), vec![Tok::Key(Key::Sys)]);
}

#[test]
fn test_key_up() {
    assert_eq!(slex("@bind"), vec![Tok::Key(Key::Bind)]);
}

#[test]
fn test_key_call() {
    assert_eq!(slex("@call"), vec![Tok::Key(Key::Call)]);
}

#[test]
fn test_key_bad_empty() {
    let a = slex("@");
    assert!(1 == vec_err(&a));
}

#[test]
fn test_key_bad_name() {
    let a = slex("@xxx");
    assert!(1 == vec_err(&a));
}

#[test]
fn test_key_bad_char() {
    let a = slex("@@");
    assert!(2 == vec_err(&a));
}

#[test]
fn test_curl_l() {
    assert_eq!(slex("{"), vec![Tok::CurlL]);
}

#[test]
fn test_curl_r() {
    assert_eq!(slex("}"), vec![Tok::CurlR]);
}
