pub mod cfg;

use super::parser::ast::Ast;
use clap::ArgMatches;

use self::cfg::block::Block;

pub fn compile(_ast: Ast, _args: &ArgMatches) {
    /*
    match args.value_of("o").unwrap_or("") {
        "1" => unimplemented!(),
        "2" => unimplemented!(),
        "3" => unimplemented!(),
        _ => ()
    }
    */

    unimplemented!()
}
