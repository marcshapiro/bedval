mod lex;
mod ast;

use std::io;
use std::io::Read;
use std::fs;
use std::path;

fn main() {
    let filen = "sample/t4.bv";
    let ok = process_file(filen);
    match ok {
        Ok(()) => println!("Ok"),
        Err(e) => println!("{}: {}", filen, e),
    }
}

fn process_file<P: AsRef<path::Path>>(filename: P) -> io::Result<()> {
    let src = try!(read_file(filename));
    let toks = lex::lex(src.text);
    let ast = ast::parse(toks);
    println!("{}", ast);
    Ok(())
}

fn empty_text() -> String {
    String::with_capacity(50)
}

fn read_file<P: AsRef<path::Path>>(filename: P) -> io::Result<BvSource> {
    let mut fr = try!(fs::File::open(filename));
    let mut s = empty_text();
    try!(fr.read_to_string(&mut s));
    Ok(BvSource { text: s.to_string() })
}

struct BvSource {
    text: String,
    // name: ...
}
