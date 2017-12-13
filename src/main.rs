#[macro_use(lazy_static)]
extern crate lazy_static;
extern crate regex;
extern crate unicode_names;
extern crate clap;

pub mod lexer;
pub mod parser;
pub mod compiler;

use lexer::Lexer;
use clap::{Arg, App};
use std::io;
use std::io::prelude::*;
use std::fs::File;

fn main() {
    let args = App::new("cannoli")
        .version("0.1.0")
        .about("Cannoli Programming Language")
        .author("Jon Catanio <joncatanio@gmail.com>, \
                 Aaron Keen <aaronkeen@gmail.com>")
        .arg(Arg::with_name("INPUT")
            .help("Sets the source file to compile")
            .required(true))
        .arg(Arg::with_name("o")
            .short("o")
            .takes_value(true)
            .help("Sets the optimization level: [1-3]"))
        .get_matches();

    // Match optimization level
    match args.value_of("o").unwrap_or("") {
        "1" => unimplemented!(),
        "2" => unimplemented!(),
        "3" => unimplemented!(),
        _ => ()
    }

    // Open file and read into `contents`
    let filename = args.value_of("INPUT").unwrap();
    let mut fp = File::open(filename).expect("file not found");
    let mut contents = String::new();
    fp.read_to_string(&mut contents)
        .expect("error reading the file");

    let stream = Lexer::new(&contents);
    let ast = parser::parse_start_symbol(stream);
    println!("AST: {:?}", ast);
    compiler::compile(ast);
}
