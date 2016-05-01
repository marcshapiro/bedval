use std::io::prelude::*;
use std::fs::File;
//use std::concat!;
fn main() {
    let filen = "sample/t1.bv";
    let fr = File::open(filen);
    match fr {
        Ok(f) =>  val_file(f),
        Err(e) => println!("{}: {}",filen, e),
    };
}

fn val_file(mut f : File) -> () {
    println!("Reading file");
    let mut s : &mut String = &mut String::with_capacity(29);
    let nr = f.read_to_string(s);
    match nr {
        Ok(n) => println!("Read string {} {} as:{}",n,s,val_string(s.to_string())),
        Err(e) => println!("Error: {}",e),
    }

    println!("Done file");
}

fn val_string(s: String) -> &'static str {
    for cr in s.chars() {
        println!("Char {}", cr)
    }
    "xxx"
}
