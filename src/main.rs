#[macro_use(lazy_static)]
extern crate lazy_static;
extern crate regex;
extern crate unicode_names;
extern crate clap;

pub mod lexer;
pub mod parser;
pub mod compiler;

use std::io::prelude::*;
use std::fs::{File, OpenOptions};
use regex::Regex;

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
        .get_matches();

    // Open file and read into `contents`
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

    println!("AST: {:?}", ast);
    let program = compiler::compile(ast, &args);
    let filename = get_file_prefix(file);
    let result = OpenOptions::new()
        .write(true)
        .create(true)
        .open(format!("{}.ll", filename));
    let mut outfile = if result.is_err() {
        println!("{}", result.unwrap_err());
        std::process::exit(1);
    } else {
        result.unwrap()
    };

    if let Err(e) = program.output_llvm(&mut outfile) {
        println!("{}", e);
        std::process::exit(1);
    }
}

fn get_file_prefix(file: &str) -> String {
    if let Some(caps) = FILENAME_RE.captures(&file) {
        caps[1].to_string()
    } else {
        println!("unsupported filetype for file: {}", file);
        std::process::exit(1)
    }
}

lazy_static! {
   static ref FILENAME_RE: Regex = Regex::new(r"(.+)\.(py|cannoli)$").unwrap();
}
