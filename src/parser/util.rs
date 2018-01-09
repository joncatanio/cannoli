use ::lexer::{Lexer, ResultToken};
use ::lexer::tokens::Token;
use super::ast::*;

/* Helper functions */
pub fn get_token(opt: &Option<(usize, ResultToken)>)
    -> Result<Token, ParserError> {
    if opt.is_none() {
        Err(ParserError::UnexpectedEOF)
    } else {
        let (_, result_token) = opt.clone().unwrap();
        Ok(result_token.clone().unwrap())
    }
}

// Checks for `not in` and `is not` which needs to peek at the next token and
// will modify the `stream`.
pub fn get_cmp_op(opt: &Option<(usize, ResultToken)>, stream: &mut Lexer)
    -> Option<(Option<(usize, ResultToken)>, CmpOperator)> {
    let token = get_token(&opt);
    let opt = stream.next();
    let next_token = get_token(&opt);

    match token {
        Token::EQ => Some((opt, CmpOperator::EQ)),
        Token::NE => Some((opt, CmpOperator::NE)),
        Token::LT => Some((opt, CmpOperator::LT)),
        Token::LE => Some((opt, CmpOperator::LE)),
        Token::GT => Some((opt, CmpOperator::GT)),
        Token::GE => Some((opt, CmpOperator::GE)),
        Token::Is => {
            match next_token {
                Token::Not => Some((stream.next(), CmpOperator::IsNot)),
                _ => Some((opt, CmpOperator::Is))
            }
        },
        Token::In => Some((opt, CmpOperator::In)),
        Token::Not => {
            match next_token {
                Token::In => Some((stream.next(), CmpOperator::NotIn)),
                _ => panic!("expected 'not in', found '{:?}'", next_token)
            }
        }
        _ => None
    }
}

pub fn get_shift_op(opt: &Option<(usize, ResultToken)>) -> Option<Operator> {
    match get_token(&opt) {
        Token::Lshift => Some(Operator::LShift),
        Token::Rshift => Some(Operator::RShift),
        _ => None
    }
}

pub fn get_arith_op(opt: &Option<(usize, ResultToken)>) -> Option<Operator> {
    match get_token(&opt) {
        Token::Plus  => Some(Operator::Add),
        Token::Minus => Some(Operator::Sub),
        _ => None
    }
}

pub fn get_term_op(opt: &Option<(usize, ResultToken)>) -> Option<Operator> {
    match get_token(&opt) {
        Token::Times       => Some(Operator::Mult),
        Token::At          => Some(Operator::MatMult),
        Token::Divide      => Some(Operator::Div),
        Token::Mod         => Some(Operator::Mod),
        Token::DivideFloor => Some(Operator::FloorDiv),
        _ => None
    }
}

pub fn get_factor_op(opt: &Option<(usize, ResultToken)>)
    -> Option<UnaryOperator> {
    match get_token(&opt) {
        Token::Plus   => Some(UnaryOperator::UAdd),
        Token::Minus  => Some(UnaryOperator::USub),
        Token::BitNot => Some(UnaryOperator::Invert),
        _ => None
    }
}

/* Token validation functions to determine if a starting token is found for
 * a given rule. */
pub fn valid_stmt(token: &Token) -> bool {
    match *token {
        _ if valid_simple_stmt(token)   => true,
        _ if valid_compound_stmt(token) => true,
        _ => false
    }
}

pub fn valid_simple_stmt(token: &Token) -> bool {
    match *token {
        Token::Del      => true,
        Token::Pass     => true,
        Token::Global   => true,
        Token::Nonlocal => true,
        Token::Assert   => true,
        Token::Import   => true,
        Token::From     => true,
        _ if valid_flow_stmt(token) => true,
        _ => valid_test_expr(token)
    }
}

pub fn valid_compound_stmt(token: &Token) -> bool {
    match *token {
        Token::If    => true,
        Token::While => true,
        Token::For   => true,
        Token::Try   => true,
        Token::With  => true,
        Token::Def   => true,
        Token::Class => true,
        Token::At    => true,
        _ => false
    }
}

pub fn valid_aug_assign(token: &Token) -> bool {
    match *token {
        Token::AssignPlus        => true,
        Token::AssignMinus       => true,
        Token::AssignTimes       => true,
        Token::AssignAt          => true,
        Token::AssignDivide      => true,
        Token::AssignMod         => true,
        Token::AssignBitAnd      => true,
        Token::AssignBitOr       => true,
        Token::AssignBitXor      => true,
        Token::AssignLshift      => true,
        Token::AssignRshift      => true,
        Token::AssignExponent    => true,
        Token::AssignDivideFloor => true,
        _ => false
    }
}

pub fn valid_flow_stmt(token: &Token) -> bool {
    match *token {
        Token::Break    => true,
        Token::Continue => true,
        Token::Return   => true,
        Token::Raise    => true,
        Token::Yield    => true,
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

pub fn valid_expr_list(token: &Token) -> bool {
    match *token {
        Token::Times => true,
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
        Token::LT  => true,
        Token::GT  => true,
        Token::EQ  => true,
        Token::GE  => true,
        Token::LE  => true,
        Token::NE  => true,
        Token::In  => true,
        Token::Not => true, // `not in`
        Token::Is  => true, // `is` and `is not`
        _ => false
    }
}

pub fn valid_argument(token: &Token) -> bool {
    match *token {
        Token::Times    => true,
        Token::Exponent => true,
        _ => valid_test_expr(token)
    }
}

pub fn valid_subscript(token: &Token) -> bool {
    match *token {
        Token::Semi => true,
        _ => valid_test_expr(token)
    }
}

pub fn valid_yield_arg(token: &Token) -> bool {
    match *token {
        Token::From => true,
        _ => valid_test_expr(token)
    }
}

pub fn valid_atom_paren(token: &Token) -> bool {
    match *token {
        Token::Yield => true,
        _ => valid_test_star(token)
    }
}

pub fn valid_test_star(token: &Token) -> bool {
    match *token {
        Token::Times => true,
        _ => valid_test_expr(token)
    }
}

pub fn valid_dict_set_maker(token: &Token) -> bool {
    match *token {
        Token::Exponent => true,
        _ => valid_test_star(token)
    }
}

pub fn valid_dict_maker(token: &Token) -> bool {
    match *token {
        Token::Exponent => true,
        _ => valid_test_expr(token)
    }
}

pub fn valid_import_as_name(token: &Token) -> bool {
    match *token {
        Token::Identifier(_) => true,
        _ => false
    }
}

/* Utility Types */
#[derive(Debug, PartialEq)]
pub enum ArgType {
    Positional,
    Keyword
}

#[derive(Debug, PartialEq)]
pub enum TLCompType {
    Tuple,
    List
}
