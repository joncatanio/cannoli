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
        Token::Pass     => true,
        Token::Global   => true,
        Token::Nonlocal => true,
        _ => valid_flow_stmt(token)
    }
}

pub fn valid_flow_stmt(token: &Token) -> bool {
    match *token {
        Token::Break    => true,
        Token::Continue => true,
        Token::Return   => true,
        _ => false
    }
}

pub fn valid_test_expr(token: &Token) -> bool {
    match *token {
        Token::Not => true,
        _ => valid_expr(token)
    }
}

pub fn valid_expr(token: &Token) -> bool {
    match *token {
        Token::Plus          => true,
        Token::Minus         => true,
        Token::BitNot        => true,
        //Token::Await         => true,
        Token::Lparen        => true,
        Token::Lbracket      => true,
        Token::Lbrace        => true,
        Token::Identifier(_) => true,
        Token::DecInteger(_) => true,
        Token::BinInteger(_) => true,
        Token::OctInteger(_) => true,
        Token::HexInteger(_) => true,
        Token::Float(_)      => true,
        Token::Imaginary(_)  => true,
        Token::String(_)     => true,
        Token::Ellipsis      => true,
        Token::None          => true,
        Token::True          => true,
        Token::False         => true,
        _ => false
    }
}
