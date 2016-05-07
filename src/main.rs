use std::io;
use std::io::prelude::*;
use std::fmt;
use std::fs::File;
use std::path::Path;

fn main() {
    let filen = "sample/t2.bv";
    let ok = process_file(filen);
    match ok {
        Ok(()) => println!("Ok"),
        Err(e) => println!("{}: {}", filen, e),
    }
}

fn process_file<P: AsRef<Path>>(filename: P) -> io::Result<()> {
    let src = try!(read_file(filename));
    let toks = lex(src.text, false, false);
    for tok in toks {
        println!("{}", tok)
    }
    Ok(())
}

fn read_file<P: AsRef<Path>>(filename: P) -> io::Result<BvSource> {
    let mut fr = try!(File::open(filename));
    let mut s : &mut String = &mut String::with_capacity(29);
    try!(fr.read_to_string(s));
    Ok(BvSource { text: s.to_string() })
}

struct BvSource {
    text: String,
    // name: ...
}

// token enum list
#[derive(Debug)]
enum BvTokE {
    QLiteral(String),  // 'string'
    Whitespace(String), //  space, newline, tab
    Comment(String), // # string\n
    Error(String),
}
impl fmt::Display for BvTokE {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BvTokE::QLiteral(ref q) => write!(f, "QLiteral {}", q),
            BvTokE::Whitespace(ref w) => write!(f, "Whitespace {}", w.len()),
            BvTokE::Comment(ref c) => write!(f, "Comment {}", c),
            BvTokE::Error(ref e) => write!(f, "Error {}", e),
        }
    }
}

// full token // todo: add range, source, ...
#[derive(Debug)]
struct BvToken {
    value: BvTokE,
}
impl fmt::Display for BvToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

fn lex_white(first_char: char, chars: &mut std::str::Chars)
    -> (Option<char>, BvTokE) {
    let mut word = String::with_capacity(20);
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

fn lex_q(chars: &mut std::str::Chars) -> BvTokE {
    let mut word = String::with_capacity(20);
    while let Some(c) = chars.next() {
        if c == '\'' {
            return BvTokE::QLiteral(word);
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
    let mut word = String::with_capacity(20);
    while let Some(c) = chars.next() {
        if c == '\n' {
            return BvTokE::Comment(word.clone());
        } else {
            word = word + &c.to_string();
        }
    }
    BvTokE::Error("Comment unreachable".to_string())
}

fn lex(text: String, pass_white: bool, pass_comment: bool) -> Vec<BvToken> {
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
            }
            else {
                if c == '\'' {
                    let toke = lex_q(&mut chars);
                    let tok = BvToken { value: toke };
                    tokens.push(tok);
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
