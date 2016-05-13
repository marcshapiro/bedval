use std;
use std::fmt;

// token enum list
#[derive(Debug)]
pub enum Tok {
    // single char tokens
    CurlL,
    CurlR,

    // keywords (@)
    KeyStruct,
    KeyBind,
    KeyFrom,
    KeyColumn,
    KeyRoot,
    KeyUp,
    KeySys,
    KeyMy,

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
            Tok::KeyStruct => write!(f, "Struct"),
            Tok::KeyBind => write!(f, "Bind"),
            Tok::KeyFrom => write!(f, "From"),
            Tok::KeyColumn => write!(f, "Column"),
            Tok::KeyRoot => write!(f, "Root"),
            Tok::KeyUp => write!(f, "Up"),
            Tok::KeySys => write!(f, "Sys"),
            Tok::KeyMy => write!(f, "My"),
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

fn at_tok(word: String) -> Tok {
    if word == "bind" {
        Tok::KeyBind
    } else if word == "column" {
        Tok::KeyColumn
    } else if word == "from" {
        Tok::KeyFrom
    } else if word == "my" {
        Tok::KeyMy
    } else if word == "root" {
        Tok::KeyRoot
    } else if word == "struct" {
        Tok::KeyStruct
    } else if word == "sys" {
        Tok::KeySys
    } else if word == "up" {
        Tok::KeyUp
    } else {
        Tok::Error(format!("Unknown keyword @{}", word))
    }
}

fn lex_at(chars: &mut std::str::Chars) -> (Option<char>, Tok) {
    let (oc, word) = read_to(chars, |c| !c.is_alphanumeric());
    (oc, at_tok(word))
}

fn lex_q(chars: &mut std::str::Chars) -> Tok {
    let mut word = empty_literal();
    while let Some(c) = chars.next() {
        if c == '\'' {
            return Tok::Literal(word);
        } else if c == '\n' {
            return Tok::Error("Single quote to end of line".to_string());
        } else {
            word = word + &c.to_string();
        }
    }
    Tok::Error("Unreachable end of single quote".to_string())
}

fn lex_hash(chars: &mut std::str::Chars, pass_comment: bool) -> Option<Tok> {
    let mut n_hash = 1;
    while let Some(c) = chars.next() {
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

struct QqFlags {
    match_word: bool, // true for leading qQ.   false for leading tT
    digit_esc: bool, // d
    quote_esc: bool, // q
    tab_esc: bool, // t
    curl_esc: bool, // c
    // only w/ qQ
    hidden_chars: bool, // h
    newlines: bool, // n
}

fn init_qq_flags(mw: bool, df: bool) -> QqFlags {
    QqFlags {
        match_word: mw,
        digit_esc: df,
        quote_esc: df,
        tab_esc: df,
        curl_esc: df,
        hidden_chars: mw && df,
        newlines: mw && df,
    }
}

fn qq_flag(flags: String) -> Result<QqFlags, Tok> {
    let mut chars = flags.chars();
    let fc = match chars.next() {
        None => return Err(Tok::Error("qq empty word".to_string())),
        Some(c) => c
    };
    let t_cs = "cdqt";
    let q_cs = "cdhnqt";
    let (cs,rv,qf) = match fc {
        't' => (t_cs, true, init_qq_flags(false, false)),
        'T' => (t_cs, false, init_qq_flags(false, true)),
        'q' => (q_cs, true, init_qq_flags(true, false)),
        'Q' => (q_cs, false, init_qq_flags(true, true)),
        _ => { return Err(Tok::Error("qq flags must start with T or Q".to_string())) }
    };
    let mut f = qf;
    let mut last_ix: Option<usize> = None;
    while let Some(c) = chars.next() {
        let oix = cs.find(c);
        let ix = match oix {
            None => { return Err(Tok::Error("unrecognized qq flag".to_string())) }
            Some(ii) => ii
        };
        match last_ix {
            None => {},
            Some(lix) => if ix <= lix {
                return Err(Tok::Error("qq flags must be ordered".to_string()))
            }
        };
        last_ix = Some(ix);
        match c {
            'c' => { f.curl_esc = rv },
            'd' => { f.digit_esc = rv },
            'h' => { f.hidden_chars = rv },
            'n' => { f.newlines = rv },
            'q' => { f.quote_esc = rv },
            't' => { f.tab_esc = rv },
            _ => return Err(Tok::Error("invalid qq flag".to_string()))
        }
    }

    Ok(f)
}

fn lex_esc(chars: &mut std::str::Chars, digit_esc: bool, quote_esc: bool,
        tab_esc: bool) -> Option<char> {
    match chars.next() {
        None => None,
        Some(c) => if c == '\\' {
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

fn lex_tqq(chars: &mut std::str::Chars, curl_esc: bool, digit_esc: bool,
        quote_esc: bool, tab_esc: bool, newlines: bool, hidden_chars: bool) -> Tok {
    let slash_esc = tab_esc || digit_esc || quote_esc;
    let mut word = empty_literal();
    while let Some(c) = chars.next() {
        if c == '"' {
            return Tok::Literal(word)
        } else if slash_esc && c == '\\' {
            let ow = lex_esc(chars, digit_esc, quote_esc, tab_esc);
            match ow {
                Some(w) => word.push(w),
                None => return Tok::Error("esc to eof".to_string()),
            }
        } else if curl_esc && c == '{' {
            return Tok::Error("\\{ddd} NYI".to_string())
        } else if !newlines && c == '\n' {
            return Tok::Error("newline forbidden".to_string())
        } else if !hidden_chars && c.is_control() { // probably not the exact test
            return Tok::Error("hidden chars forbidden".to_string())
        } else {
            word.push(c);
        }
    }
    Tok::Error("tqq to eof".to_string())
}

fn lex_qq(flags: String, chars: &mut std::str::Chars) -> Tok {
    // t None        // tdqt == T      // t"text"
    // T c d q t       // Tcdqt == t   //  d=\123   q=\"\'   t=\t\n\r (dqt get \\)
    // q None        // qcdhnqt == Q   // q"alnum"text"alnum"
    // Q c d h n q t // Qcdhnqt == q // c={nl},,,  h=ctrl  n=newline
    let f = match qq_flag(flags) {
        Err(e) => return e,
        Ok(fv) => fv,
    };
    if f.match_word {
        return Tok::Error("qqq nyi".to_string())
    }
    lex_tqq(chars, f.curl_esc, f.digit_esc, f.quote_esc, f.tab_esc, f.newlines, f.hidden_chars)
}

fn lex_bare(first_char: char, chars: &mut std::str::Chars) -> (Option<char>, Tok) {
    let mut word = empty_literal();
    word.push(first_char);
    while let Some(c) = chars.next() {
        if c.is_alphanumeric() {
            word.push(c);
        } else if c == '"' {
            return (None, lex_qq(word, chars));
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

#[test]
fn test_empty_string() {
    let a = lex("".to_string());
    assert!(0 == a.len())
}

#[test]
fn test_white() {
    let w = "  \t  \n  ".to_string();
    let v = w.clone();
    let b = lex(w);
    assert!(1 == b.len());
    match b[0] {
        Tok::Whitespace(ref x) => assert!(x.clone() == v),
        _ => assert!(false),
    }
}

#[test]
fn test_bare() {
    let w = "abc123";
    let a = lex(w.to_string());
    assert!(1 == a.len());
    match a[0] {
        Tok::Literal(ref x) => assert!(x.clone() == w.to_string()),
        _ => assert!(false),
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
