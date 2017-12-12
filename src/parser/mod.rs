use super::lexer::Lexer;

pub fn parse_file_input(mut stream: Lexer) -> Option<bool> {
    println!("Parsing! var is {:?}", stream.next());
    Some(true)
}
