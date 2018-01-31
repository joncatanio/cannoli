#[macro_use(lazy_static)]
extern crate lazy_static;
extern crate regex;
extern crate unicode_names;
extern crate clap;

pub mod lexer;
pub mod parser;
pub mod compiler;

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

    let file = args.value_of("INPUT").unwrap();
    let result = compiler::compile_file(file, Some(&args));
    if result.is_err() {
        println!("{}", result.unwrap_err());
        std::process::exit(1);
    }
}
