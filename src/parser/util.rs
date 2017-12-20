use ::lexer::{Lexer, ResultToken};
use ::lexer::tokens::Token;
use super::ast::*;

/* Helper functions */
pub fn get_token(opt: &Option<(usize, ResultToken)>) -> Token {
    if opt.is_none() {
        panic!("token value of None detected");
    }
    let (_, result_token) = opt.clone().unwrap();
    result_token.clone().unwrap()
}

// Checks for `not in` and `is not` which needs to peek at the next token and
// will modify the `stream`.
pub fn get_cmp_op(opt: &Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> (Option<(usize, ResultToken)>, CmpOperator) {
    let token = get_token(&opt);
    let opt = stream.next();
    let next_token = get_token(&opt);

    match token {
        Token::EQ => (opt, CmpOperator::EQ),
        Token::NE => (opt, CmpOperator::NE),
        Token::LT => (opt, CmpOperator::LT),
        Token::LE => (opt, CmpOperator::LE),
        Token::GT => (opt, CmpOperator::GT),
        Token::GE => (opt, CmpOperator::GE),
        Token::Is => {
            match next_token {
                Token::Not => (stream.next(), CmpOperator::IsNot),
                _ => (opt, CmpOperator::Is)
            }
        },
        Token::In => (opt, CmpOperator::In),
        Token::Not => {
            match next_token {
                Token::In => (stream.next(), CmpOperator::NotIn),
                _ => panic!("expected 'not in', found '{:?}'", next_token)
            }
        }
        _ => panic!("expected valid comparison operator")
    }
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
        Token::Lambda => true,
        Token::Not    => true,
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

pub fn valid_cmp_op(token: &Token) -> bool {
    match *token {
        Token::LT => true,
        Token::GT => true,
        Token::EQ => true,
        Token::GE => true,
        Token::LE => true,
        Token::NE => true,
        Token::In => true,
        Token::Not => true, // `not in`
        Token::Is  => true, // `is` and `is not`
        _ => false
    }
}
