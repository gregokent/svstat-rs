use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    let dirs = args.iter().skip(1);
    let currpath = Path::new(".");
    let origdir = match File::open(&currpath) {
        Err(why) => panic!("couldn't open current directory: {}",why.description()),
        Ok(origdir) => origdir,
    };

    for dir in dirs {
        println!("{:?}", dir );
    }
    println!("Hello, world!");
}
