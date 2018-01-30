mod util;

use std::fs::OpenOptions;

use clap::ArgMatches;
use super::parser::ast::*;

pub fn compile(file: &str, ast: Ast, _args: &ArgMatches) {
    let filename = util::get_file_prefix(file);
    let result = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(format!("{}.rs", filename));
    let mut outfile = if result.is_err() {
        println!("{}", result.unwrap_err());
        // TODO return CompileError
        unimplemented!()
    } else {
        result.unwrap()
    };
}
