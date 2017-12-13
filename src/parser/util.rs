use ::lexer::ResultToken;
use ::lexer::tokens::Token;

/* Helper functions */
pub fn get_token(opt: &Option<(usize, ResultToken)>) -> Token {
    if opt.is_none() {
        panic!("token value of None detected");
    }
    let (_, result_token) = opt.clone().unwrap();
    result_token.clone().unwrap()
}

/* Token validation functions to determine if a starting token is found for
 * a given rule. */
pub fn valid_simple_stmt(token: &Token) -> bool {
    match *token {
        Token::Pass => true,
        _ => false
    }
}
