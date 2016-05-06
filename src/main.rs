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
    let toks = s.lex(false, false);
    //println!("tokens");
    for tok in toks {
        match tok.value {
            BvTokE::QLiteral(q) => println!("QLiteral {}", q),
            BvTokE::Whitespace(w) => println!("Whitespace {}", w.len()),
            BvTokE::Comment(c) => println!("Comment {}", c),
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
}


// full token // todo: add range, source, ...
struct BvToken {
    value: BvTokE,
}

enum LexState {
    White, // [start] whitespace
    Q, // single quote
    Hash, // ###
    Comment, // # text\n
    Error,
}
impl BvFile {
    fn new(s : String) -> BvFile {
        BvFile {
            text: s,
        }
    }
    fn lex(&self, pass_white: bool, pass_comment: bool) -> Vec<BvToken> {
        let mut tokens : Vec<BvToken> = vec![];
        let mut word = String::with_capacity(20);
        let mut n_hash = 0;
        // let mut hash_word = String::with_capacity(20);

        let mut state = LexState::White;
        for c in self.text.chars() {
            match state {
                LexState::White => {
                    // println!("Start {}", c);
                    if c.is_whitespace() {
                        word = word + &c.to_string();
                    }
                    else {
                        if 0 < word.len() {
                            if pass_white {
                                let toke = BvTokE::Whitespace(word.clone());
                                let tok = BvToken { value: toke };
                                tokens.push(tok);
                            }
                            word.truncate(0);
                        }
                        if c == '\'' {
                            state = LexState::Q;
                        } else if c == '#' {
                            n_hash = 1;
                            state = LexState::Hash;
                        } else {
                            state = LexState::Error;
                        }
                    }
                },
                LexState::Q => {
                    // println!("Q {}",c);
                    if c == '\'' {
                        let toke = BvTokE::QLiteral(word.clone());
                        let tok = BvToken { value: toke };
                        tokens.push(tok);
                        word.truncate(0);
                        state = LexState::White;
                    } else if c == '\n' {
                        state = LexState::Error;
                    } else {
                        word = word + &c.to_string();
                    }
                },
                LexState::Hash => {
                    //println!("Hash");
                    if c.is_whitespace() {
                        if c == '\n' { // empty comment
                            if 1 == n_hash {
                                if pass_comment {
                                    let toke = BvTokE::Comment("".to_string());
                                    let tok = BvToken{ value: toke };
                                    tokens.push(tok);
                                }
                                state = LexState::White;
                            } else {
                                // ...
                                state = LexState::Error;
                            }
                        } else {
                            if 1 == n_hash {
                                state = LexState::Comment;
                            } else {
                                // ...
                                state = LexState::Error;
                            }
                        }
                    } else if c == '#' {
                        n_hash += 1;
                    } else if c.is_alphanumeric() {
                        // ...
                        state = LexState::Error;
                    } else if "{([<`'\"|".contains(c) {
                        // ...
                        state = LexState::Error;
                    } else if "+-".contains(c) {
                        // ...
                        state = LexState::Error;
                    } else {
                        // ...
                        state = LexState::Error;
                    }
                },
                LexState::Comment => {
                    //println!("Comment");
                    if c == '\n' {
                        if pass_comment {
                            let toke = BvTokE::Comment(word.clone());
                            let tok = BvToken{ value: toke };
                            tokens.push(tok);
                        }
                        word.truncate(0);
                        state = LexState::White;
                    } else {
                        word = word + &c.to_string();
                    }
                },
                LexState::Error => {
                    println!("Error {}", c)
                    // FIXME: reset at nl
                }
            }
        }

        tokens
    }
}
