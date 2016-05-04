use std::io::Error;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

fn main() {
    let filen = "sample/t1.bv";
    let ok = process_file(filen);
    match ok {
        Ok(()) => println!("Ok"),
        Err(e) => println!("{}: {}", filen, e),
    }
}

fn process_file<P: AsRef<Path>>(filename: P) -> Result<(), Error> {
    let s = try!(read_file(filename));
    println!("String: {}", s);
    val_string(s);
    Ok(())
}

fn read_file<P: AsRef<Path>>(filename: P) -> Result<String, Error> {
    let mut fr = try!(File::open(filename));
    let mut s : &mut String = &mut String::with_capacity(29);
    try!(fr.read_to_string(s));
    Ok(s.to_string())
}

fn val_string(s: String) {
    for cr in s.chars() {
        println!("Char: {}", cr)
    }
}
