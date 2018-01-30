#[macro_use(lazy_static)]
extern crate lazy_static;
extern crate regex;
extern crate unicode_names;
extern crate clap;

pub mod lexer;
pub mod parser;
pub mod compiler;

use std::io::prelude::*;
use std::fs::File;

use lexer::Lexer;
use clap::{Arg, App};

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
        .arg(Arg::with_name("parse")
            .long("parse")
            .help("Only parses the input file and prints the AST"))
        .get_matches();

    // Open file and read into `contents`
    // TODO move this to compiler so imports can be more easily managed
    let file = args.value_of("INPUT").unwrap();
    let mut fp = File::open(file).expect("file not found");
    let mut contents = String::new();
    fp.read_to_string(&mut contents)
        .expect("error reading the file");

    let stream = Lexer::new(&contents);
    let result = parser::parse_start_symbol(stream);
    let ast = if result.is_err() {
        println!("{}", result.unwrap_err());
        std::process::exit(1);
    } else {
        result.unwrap()
    };
    if args.is_present("parse") {
        println!("AST: {:?}", ast);
        std::process::exit(0);
    }

    println!("AST: {:?}", ast);
    // Handle errors here
    compiler::compile(file, ast, &args);
}
