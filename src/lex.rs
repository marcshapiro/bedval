use std;
use std::fmt;

// token enum list
#[derive(Debug)]
pub enum BvTokE {
    // single char tokens
    CurlL,
    CurlR,

    // keywords (@)
    KeyStruct,
    KeyBind,
    KeyFrom,
    KeyColumn,

    // literals
    Literal(String), // 'text'

    // whitespace, comments, pragmas
    Whitespace(String), //  space, newline, tab
    Comment(String), // # string\n
    // Pragma

    // oops
    Error(String),
}

impl fmt::Display for BvTokE {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BvTokE::CurlL => write!(f, "Curl Start"),
            BvTokE::CurlR => write!(f, "Curl End"),
            BvTokE::KeyStruct => write!(f, "Struct"),
            BvTokE::KeyBind => write!(f, "Bind"),
            BvTokE::KeyFrom => write!(f, "From"),
            BvTokE::KeyColumn => write!(f, "Column"),
            BvTokE::Literal(ref q) => write!(f, "q {}", q),
            BvTokE::Whitespace(ref w) => write!(f, "White {}", w.len()),
            BvTokE::Comment(ref c) => write!(f, "Comment {}", c),
            BvTokE::Error(ref e) => write!(f, "Error {}", e),
        }
    }
}

// full token // todo: add range, source, ...
#[derive(Debug)]
pub struct BvToken {
    pub value: BvTokE,
}

impl fmt::Display for BvToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

fn empty_white() -> String {
    String::with_capacity(5)
}

fn empty_key() -> String {
    String::with_capacity(8)
}

fn empty_literal() -> String {
    String::with_capacity(20)
}

fn empty_comment() -> String {
    String::with_capacity(20)
}

fn lex_white(first_char: char, chars: &mut std::str::Chars)
    -> (Option<char>, BvTokE) {
    let mut word = empty_white();
    word.push(first_char);
    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            word.push(c);
        } else {
            return (Some(c), BvTokE::Whitespace(word));
        }
    }
    (None, BvTokE::Whitespace(word))
}

fn at_tok(word: String) -> BvTokE {
    if word == "struct" {
        return BvTokE::KeyStruct;
    }
    if word == "bind" {
        return BvTokE::KeyBind;
    }
    if word == "from" {
        return BvTokE::KeyFrom;
    }
    if word == "column" {
        return BvTokE::KeyColumn;
    }
    BvTokE::Error(format!("Unknown keyword @{}", word))
}

fn lex_at(chars: &mut std::str::Chars) -> (Option<char>, BvTokE) {
    let mut word = empty_key();
    while let Some(c) = chars.next() {
        if c.is_alphanumeric() {
            word.push(c);
        } else {
            return (Some(c), at_tok(word));
        }
    }
    (None, at_tok(word))
}

fn lex_q(chars: &mut std::str::Chars) -> BvTokE {
    let mut word = empty_literal();
    while let Some(c) = chars.next() {
        if c == '\'' {
            return BvTokE::Literal(word);
        } else if c == '\n' {
            return BvTokE::Error("Single quote to end of line".to_string());
        } else {
            word = word + &c.to_string();
        }
    }
    BvTokE::Error("Unreachable end of single quote".to_string())
}

fn lex_hash(chars: &mut std::str::Chars, pass_comment: bool) -> Option<BvTokE> {
    let mut n_hash = 1;
    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            if c == '\n' { // empty comment
                if 1 == n_hash {
                    return if pass_comment {
                        Some(BvTokE::Comment("".to_string()))
                    } else {
                        None
                    };
                } else {
                    // ....
                    return Some(BvTokE::Error("Empty multihash NYI".to_string()))
                }
            } else {
                if 1 == n_hash {
                    let toke = lex_comment_eol(chars);
                    return if pass_comment { Some(toke) } else { None };
                } else {
                    return Some(BvTokE::Error("multihash NYI".to_string()));
                }
            };
        } else if c == '#' {
            n_hash += 1;
        } else if c.is_alphanumeric() {
            return Some(BvTokE::Error("pragma NYI".to_string()));
        } else if "{([<`'\"|".contains(c) {
            return Some(BvTokE::Error("Inline comment NYI".to_string()));
        } else if "+-".contains(c) {
            return Some(BvTokE::Error("On/off pragma NYI".to_string()));
        } else {
            return Some(BvTokE::Error("Bad char after #".to_string()));
        }
    }
    None
}

fn lex_comment_eol(chars: &mut std::str::Chars) -> BvTokE {
    let mut word = empty_comment();
    while let Some(c) = chars.next() {
        if c == '\n' {
            return BvTokE::Comment(word.clone());
        } else {
            word = word + &c.to_string();
        }
    }
    BvTokE::Error("Comment unreachable".to_string())
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

fn qq_flag(flags: String) -> Result<QqFlags, BvTokE> {
    let mut chars = flags.chars();
    let fc = match chars.next() {
        None => return Err(BvTokE::Error("qq empty word".to_string())),
        Some(c) => c
    };
    let t_cs = "cdqt";
    let q_cs = "cdhnqt";
    let (cs,rv,qf) = match fc {
        't' => (t_cs, true, init_qq_flags(false, false)),
        'T' => (t_cs, false, init_qq_flags(false, true)),
        'q' => (q_cs, true, init_qq_flags(true, false)),
        'Q' => (q_cs, false, init_qq_flags(true, true)),
        _ => { return Err(BvTokE::Error("qq flags must start with T or Q".to_string())) }
    };
    let mut f = qf;
    let mut last_ix: Option<usize> = None;
    while let Some(c) = chars.next() {
        let oix = cs.find(c);
        let ix = match oix {
            None => { return Err(BvTokE::Error("unrecognized qq flag".to_string())) }
            Some(ii) => ii
        };
        match last_ix {
            None => {},
            Some(lix) => if ix <= lix {
                return Err(BvTokE::Error("qq flags must be ordered".to_string()))
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
            _ => return Err(BvTokE::Error("invalid qq flag".to_string()))
        }
    }

    Ok(f)
}

fn lex_esc(chars: &mut std::str::Chars, digit_esc: bool, quote_esc: bool,
        tab_esc: bool) -> Option<char> {
    let oc = chars.next();
    match oc {
        None => return None,
        Some(c) => if c == '\\' {
            return Some('\\')
        } else if digit_esc && c == '{' {
            // ....
        } else if tab_esc && c == 't' {
            return Some('\t')
        } else if tab_esc && c == 'n' {
            return Some('\n')
        } else if tab_esc && c == '0' {
            return Some('\0')
        } else if quote_esc && c == '"' {
            return Some('\"')
        } else {
            return None
        }
    }
    None
}

fn lex_tqq(chars: &mut std::str::Chars, curl_esc: bool, digit_esc: bool,
        quote_esc: bool, tab_esc: bool, newlines: bool, hidden_chars: bool) -> BvTokE {
    let slash_esc = tab_esc || digit_esc || quote_esc;
    let mut word = empty_literal();
    while let Some(c) = chars.next() {
        if c == '"' {
            return BvTokE::Literal(word)
        } else if slash_esc && c == '\\' {
            let ow = lex_esc(chars, digit_esc, quote_esc, tab_esc);
            match ow {
                Some(w) => word.push(w),
                None => return BvTokE::Error("esc to eof".to_string()),
            }
        } else if curl_esc && c == '{' {
            return BvTokE::Error("\\{ddd} NYI".to_string())
        } else if !newlines && c == '\n' {
            return BvTokE::Error("newline forbidden".to_string())
        } else if !hidden_chars && c.is_control() { // probably not the exact test
            return BvTokE::Error("hidden chars forbidden".to_string())
        } else {
            word.push(c);
        }
    }


    BvTokE::Error("tqq to eof".to_string())
}

fn lex_qq(flags: String, chars: &mut std::str::Chars) -> BvTokE {
    // t None        // tdqt == T      // t"text"
    // T c d q t       // Tcdqt == t   //  d=\123   q=\"\'   t=\t\n\r (dqt get \\)
    // q None        // qcdhnqt == Q   // q"alnum"text"alnum"
    // Q c d h n q t // Qcdhnqt == q // c={nl},,,  h=ctrl  n=newline
    let f = match qq_flag(flags) {
        Err(e) => return e,
        Ok(fv) => fv,
    };
    if f.match_word {
        return BvTokE::Error("qqq nyi".to_string())
    }
    lex_tqq(chars, f.curl_esc, f.digit_esc, f.quote_esc, f.tab_esc, f.newlines, f.hidden_chars)
}

fn lex_bare(first_char: char, chars: &mut std::str::Chars) -> (Option<char>, BvTokE) {
    let mut word = empty_literal();
    word.push(first_char);
    while let Some(c) = chars.next() {
        if c.is_alphanumeric() {
            word.push(c);
        } else if c == '"' {
            return (None, lex_qq(word, chars));
        } else {
            return (Some(c), BvTokE::Literal(word));
        }
    }
    (None, BvTokE::Literal(word))
}

pub fn lex(text: String, pass_white: bool, pass_comment: bool) -> Vec<BvToken> {
    let mut tokens : Vec<BvToken> = vec![];

    let mut chars = text.chars();
    while let Some(sc) = chars.next() {
        let mut c = sc;
        let mut rpt = true;
        while rpt {
            rpt = false;
            if c.is_whitespace() {
                let (oc, toke) = lex_white(c, &mut chars);
                if pass_white {
                    let tok = BvToken { value: toke };
                    tokens.push(tok)
                }
                match oc {
                    None => {},
                    Some(ac) => {
                        c = ac;
                        rpt = true;
                    },
                }
            } else if c.is_alphanumeric() {
                let (oc, toke) = lex_bare(c, &mut chars);
                let tok = BvToken { value: toke };
                tokens.push(tok);
                match oc {
                    None => {},
                    Some(ac) => {
                        c = ac;
                        rpt = true;
                    },
                }
            }
            else {
                if c == '{' {
                    let tok = BvToken { value: BvTokE::CurlL };
                    tokens.push(tok);
                } else if c == '}' {
                    let tok = BvToken { value: BvTokE::CurlR };
                    tokens.push(tok);
                } else if c == '\'' {
                    let toke = lex_q(&mut chars);
                    let tok = BvToken { value: toke };
                    tokens.push(tok);
                } else if c == '@' {
                    let (oc, toke) = lex_at(&mut chars);
                    let tok = BvToken { value: toke };
                    tokens.push(tok);
                    match oc {
                        None => {},
                        Some(ac) => {
                            c = ac;
                            rpt = true;
                        },
                    }
                } else if c == '#' {
                    let otoke = lex_hash(&mut chars, pass_comment);
                    match otoke {
                        Some(toke) => {
                            let tok = BvToken { value: toke };
                            tokens.push(tok);
                        },
                        None => {}
                    }
                } else {
                    // TODO: read to nl
                    let toke = BvTokE::Error("Unknown character - NYI".to_string());
                    let tok = BvToken { value : toke };
                    tokens.push(tok);
                }
            }
        }
    }

    tokens
}
