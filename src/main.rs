use std::io;
use std::io::prelude::*;
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
    let s = try!(read_file(filename));
    //println!("BvFile");
    let toks = s.lex(false, true);
    //println!("tokens");
    for tok in toks {
        match tok.value {
            BvTokE::QLiteral(q) => println!("QLiteral {}", q),
            BvTokE::Whitespace(w) => println!("Whitespace {}", w.len()),
            BvTokE::Comment(c) => println!("Comment {}", c),
            BvTokE::Error(e) => println!("Error {}", e),
        }
    }
    Ok(())
}

fn read_file<P: AsRef<Path>>(filename: P) -> io::Result<BvFile> {
    let mut fr = try!(File::open(filename));
    let mut s : &mut String = &mut String::with_capacity(29);
    try!(fr.read_to_string(s));
    Ok(BvFile::new(s.to_string()))
}

struct BvFile {
    text: String,
}

// token enum list
enum BvTokE {
    QLiteral(String),  // 'string'
    Whitespace(String), //  space, newline, tab
    Comment(String), // # string\n
    Error(String),
}


// full token // todo: add range, source, ...
struct BvToken {
    value: BvTokE,
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


impl BvFile {
    fn new(s : String) -> BvFile {
        BvFile {
            text: s,
        }
    }

    fn lex(&self, pass_white: bool, pass_comment: bool) -> Vec<BvToken> {
        let mut tokens : Vec<BvToken> = vec![];

        let mut chars = self.text.chars();
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
}
