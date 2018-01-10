pub mod ast;
mod util;
mod errors;

use super::lexer::{Lexer, ResultToken};
use super::lexer::tokens::Token;
use self::ast::*;
use self::util::{ArgType, TLCompType};
use self::errors::ParserError;

// Optional tuple with line number and result token
type OptToken = Option<(usize, ResultToken)>;

pub fn parse_start_symbol(mut stream: Lexer) -> Result<Ast, ParserError> {
    let (opt, ast) = parse_file_input(stream.next(), &mut stream)?;

    match opt {
        Some(_) => panic!("expected 'EOF' found '{:?}'", opt.unwrap()),
        None    => Ok(ast)
    }
}

fn parse_file_input(opt: OptToken, mut stream: &mut Lexer)
    -> Result<(OptToken, Ast), ParserError> {
    if opt.is_none() {
        return Ok((opt, Ast::Module { body: vec![] }));
    }

    match util::get_token(&opt)? {
        Token::Newline => parse_file_input(stream.next(), &mut stream),
        _ => {
            let (opt, mut stmt_vec) = parse_stmt(opt, &mut stream)?;
            let (opt, Ast::Module { body }) =
                parse_file_input(opt, &mut stream)?;

            stmt_vec.extend(body);
            Ok((opt, Ast::Module { body: stmt_vec }))
        }
    }
}

fn parse_decorator(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    let (opt, expr) = parse_dotted_name_attr(opt, stream)?;

    match util::get_token(&opt)? {
        Token::Newline => Ok((stream.next(), expr)),
        Token::Lparen => {
            let (opt, args, keywords) = parse_arglist(stream.next(), stream)?;
            let opt = match util::get_token(&opt)? {
                Token::Rparen => stream.next(),
                _ => return Err(ParserError::UnexpectedToken(
                    Token::Rparen, opt))
            };
            let opt = match util::get_token(&opt)? {
                Token::Newline => stream.next(),
                _ => return Err(ParserError::UnexpectedToken(
                    Token::Newline, opt))
            };

            Ok((opt, Expression::Call { func: Box::new(expr), args, keywords }))
        },
        _ => Err(ParserError::InvalidSyntax(opt))
    }
}

fn parse_decorators(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Vec<Expression>), ParserError> {
    match util::get_token(&opt)? {
        Token::At => {
            let (opt, decorator) = parse_decorator(stream.next(), stream)?;
            let (opt, mut decorator_list) = parse_decorators(opt, stream)?;

            decorator_list.insert(0, decorator);
            Ok((opt, decorator_list))
        },
        _ => Ok((opt, vec![]))
    }
}

fn parse_decorated(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Statement), ParserError> {
    let (opt, decorator_list) = parse_decorators(opt, stream)?;

    match util::get_token(&opt)? {
        Token::Def   => parse_func_def(stream.next(), decorator_list, stream),
        Token::Class => parse_class_def(stream.next(), decorator_list, stream),
        _ => Err(ParserError::InvalidSyntax(opt))
    }
}

fn parse_func_def(opt: OptToken, decorator_list: Vec<Expression>,
    stream: &mut Lexer) -> Result<(OptToken, Statement), ParserError> {
    let (opt, name) = match util::get_token(&opt)? {
        Token::Identifier(name) => (stream.next(), name),
        _ => return Err(ParserError::InvalidSyntax(opt))
    };
    let (opt, args) = parse_parameters(opt, stream)?;
    let (opt, returns) = match util::get_token(&opt)? {
        Token::Arrow => {
            let (opt, expr) = parse_test_expr(stream.next(), stream)?;
            (opt, Some(expr))
        },
        _ => (opt, None)
    };
    let (opt, body) = match util::get_token(&opt)? {
        Token::Colon => parse_suite(stream.next(), stream)?,
        _ => return Err(ParserError::UnexpectedToken(Token::Colon, opt))
    };

    Ok((opt, Statement::FunctionDef {
        name, args, body, decorator_list, returns }))
}

fn parse_parameters(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Arguments), ParserError> {
    match util::get_token(&opt)? {
        Token::Lparen => {
            let arguments = Arguments::Arguments {
                args: vec![], vararg: None, kwonlyargs: vec![],
                kw_defaults: vec![], kwarg: None, defaults: vec![]
            };
            let (opt, typedargslist) = parse_argslist(stream.next(),
                parse_tfpdef, false, arguments, stream)?;

            match util::get_token(&opt)? {
                Token::Rparen => Ok((stream.next(), typedargslist)),
                _ => Err(ParserError::UnexpectedToken(Token::Rparen, opt))
            }
        },
        _ => Err(ParserError::UnexpectedToken(Token::Lparen, opt))
    }
}

// Tail-recursively destructures and modifies an Arguments::Arguments the base
// case is an invalid argslist Token which returns the completed Arguments enum
fn parse_argslist(opt: OptToken,
    parse_f: fn(OptToken, &mut Lexer) -> Result<(OptToken, Arg), ParserError>,
    force_kw: bool, arguments: Arguments,
    stream: &mut Lexer) -> Result<(OptToken, Arguments), ParserError> {
    match util::get_token(&opt)? {
        Token::Times => {
            let (opt, arguments) = parse_argslist_vararg(stream.next(),
                parse_f, arguments, stream)?;

            let opt = match util::get_token(&opt)? {
                Token::Comma => stream.next(),
                _ => opt
            };
            parse_argslist(opt, parse_f, true, arguments, stream)
        },
        Token::Exponent => {
            let (opt, arguments) = parse_argslist_kwarg(stream.next(),
                parse_f, arguments, stream)?;

            let opt = match util::get_token(&opt)? {
                Token::Comma => stream.next(),
                _ => opt
            };
            parse_argslist(opt, parse_f, force_kw, arguments, stream)
        },
        Token::Identifier(_) => {
            let (opt, arguments) = parse_argslist_id(opt, parse_f, force_kw,
                arguments, stream)?;

            let opt = match util::get_token(&opt)? {
                Token::Comma => stream.next(),
                _ => opt
            };
            parse_argslist(opt, parse_f, force_kw, arguments, stream)
        },
        _ => Ok((opt, arguments))
    }
}

fn parse_argslist_vararg(opt: OptToken,
    parse_f: fn(OptToken, &mut Lexer) -> Result<(OptToken, Arg), ParserError>,
    arguments: Arguments, stream: &mut Lexer)
    -> Result<(OptToken, Arguments), ParserError> {
    // Destructure the Arguments enum to modify its contents
    let (args, vararg, kwonlyargs, kw_defaults, kwarg, defaults) =
        match arguments {
            Arguments::Arguments { args, vararg, kwonlyargs,
                kw_defaults, kwarg, defaults } =>
                (args, vararg, kwonlyargs, kw_defaults, kwarg, defaults)
    };

    if kwarg.is_some() || vararg.is_some() {
        return Err(ParserError::InvalidSyntax(opt))
    }

    let (opt, vararg) = match util::get_token(&opt)? {
        Token::Identifier(_) => {
            let (opt, arg) = parse_f(opt, stream)?;
            (opt, Some(arg))
        },
        _ => (opt, None)
    };


    Ok((opt, Arguments::Arguments { args, vararg, kwonlyargs,
        kw_defaults, kwarg, defaults }))
}

fn parse_argslist_kwarg(opt: OptToken,
    parse_f: fn(OptToken, &mut Lexer) -> Result<(OptToken, Arg), ParserError>,
    arguments: Arguments, stream: &mut Lexer)
    -> Result<(OptToken, Arguments), ParserError> {
    // Destructure the Arguments enum to modify its contents
    let (args, vararg, kwonlyargs, kw_defaults, kwarg, defaults) =
        match arguments {
            Arguments::Arguments { args, vararg, kwonlyargs,
                kw_defaults, kwarg, defaults } =>
                (args, vararg, kwonlyargs, kw_defaults, kwarg, defaults)
    };

    if kwarg.is_some() {
        return Err(ParserError::InvalidSyntax(opt))
    }

    let (opt, arg) = parse_f(opt, stream)?;
    Ok((opt, Arguments::Arguments { args, vararg, kwonlyargs,
        kw_defaults, kwarg: Some(arg), defaults }))
}

fn parse_argslist_id(opt: OptToken,
    parse_f: fn(OptToken, &mut Lexer) -> Result<(OptToken, Arg), ParserError>,
    force_kw: bool, arguments: Arguments, stream: &mut Lexer)
    -> Result<(OptToken, Arguments), ParserError> {
    // Destructure the Arguments enum to modify its contents
    let (mut args, vararg, mut kwonlyargs, mut kw_defaults, kwarg,
        mut defaults) = match arguments {
            Arguments::Arguments { args, vararg, kwonlyargs,
                kw_defaults, kwarg, defaults } =>
                (args, vararg, kwonlyargs, kw_defaults, kwarg, defaults)
    };

    if kwarg.is_some() {
        return Err(ParserError::InvalidSyntax(opt))
    }

    let (opt, arg) = parse_f(opt, stream)?;
    let (opt, default) = match util::get_token(&opt)? {
        Token::Assign => {
            let opt = stream.next();

            if util::valid_test_expr(&util::get_token(&opt)?) {
                let (opt, expr) = parse_test_expr(opt, stream)?;
                (opt, Some(expr))
            } else {
                return Err(ParserError::InvalidSyntax(opt))
            }
        },
        _ => (opt, None)
    };

    if force_kw || vararg.is_some() {
        // Keyword argument
        kwonlyargs.push(arg);

        match default {
            Some(expr) => kw_defaults.push(expr),
            None => kw_defaults.push(Expression::None)
        }
    } else if default.is_none() && !defaults.is_empty() {
        // Ensure that the positional argument is in a valid order
        return Err(ParserError::NonDefaultArgFollowsDefault(opt))
    } else {
        // Positional argument
        args.push(arg);

        if default.is_some() {
            defaults.push(default.unwrap());
        }
    }

    Ok((opt, Arguments::Arguments { args, vararg, kwonlyargs,
        kw_defaults, kwarg, defaults }))
}

fn parse_tfpdef(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Arg), ParserError> {
    let (opt, arg) = match util::get_token(&opt)? {
        Token::Identifier(arg) => (stream.next(), arg),
        _ => return Err(ParserError::InvalidSyntax(opt))
    };
    let (opt, annotation) = match util::get_token(&opt)? {
        Token::Colon => {
            let opt = stream.next();

            if util::valid_test_expr(&util::get_token(&opt)?) {
                let (opt, expr) = parse_test_expr(opt, stream)?;
                (opt, Some(expr))
            } else {
                return Err(ParserError::InvalidSyntax(opt))
            }
        },
        _ => (opt, None)
    };

    Ok((opt, Arg::Arg { arg, annotation }))
}

fn parse_vfpdef(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Arg), ParserError> {
    let (opt, arg) = match util::get_token(&opt)? {
        Token::Identifier(arg) => (stream.next(), arg),
        _ => return Err(ParserError::InvalidSyntax(opt))
    };

    Ok((opt, Arg::Arg { arg, annotation: None }))
}

fn parse_stmt(opt: OptToken, mut stream: &mut Lexer)
    -> Result<(OptToken, Vec<Statement>), ParserError> {
    let token = util::get_token(&opt)?;

    if util::valid_simple_stmt(&token) {
        parse_simple_stmt(opt, &mut stream)
    } else {
        let (opt, stmt) = parse_compound_stmt(opt, &mut stream)?;
        Ok((opt, vec![stmt]))
    }
}

// Both `parse_stmt` & `parse_stmts` return Vec structs, but `parse_stmts` calls
// `parse_stmt` recursively. If a compound_stmt is encountered then the Vec will
// be of size 1. Extending the Vec is a simple implementation of otherwise more
// complex logic.
fn parse_stmts(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Vec<Statement>), ParserError> {
    if util::valid_stmt(&util::get_token(&opt)?) {
        let (opt, mut stmt_vec) = parse_stmt(opt, stream)?;
        let (opt, stmts_vec) = parse_stmts(opt, stream)?;

        stmt_vec.extend(stmts_vec);
        Ok((opt, stmt_vec))
    } else {
        Ok((opt, vec![]))
    }
}

fn parse_compound_stmt(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Statement), ParserError> {
    match util::get_token(&opt)? {
        Token::If    => parse_if_stmt(stream.next(), stream),
        Token::While => parse_while_stmt(stream.next(), stream),
        Token::For   => parse_for_stmt(stream.next(), stream),
        Token::Try   => parse_try_stmt(stream.next(), stream),
        Token::With  => parse_with_stmt(stream.next(), stream),
        Token::Def   => parse_func_def(stream.next(), vec![], stream),
        Token::Class => parse_class_def(stream.next(), vec![], stream),
        Token::At    => parse_decorated(opt, stream),
        _ => unreachable!()
    }
}

fn parse_simple_stmt(opt: OptToken, mut stream: &mut Lexer)
    -> Result<(OptToken, Vec<Statement>), ParserError> {
    let (opt, small_stmt) = parse_small_stmt(opt, &mut stream)?;

    // TODO maybe use a peek here?
    match util::get_token(&opt)? {
        Token::Semi => {
            let opt = stream.next();

            match util::get_token(&opt)? {
                Token::Newline => Ok((stream.next(), vec![small_stmt])),
                _ => {
                    let (opt, mut stmts) = parse_simple_stmt(opt, stream)?;

                    stmts.insert(0, small_stmt);
                    Ok((opt, stmts))
                }
            }
        },
        Token::Newline => {
            Ok((stream.next(), vec![small_stmt]))
        },
        _ => Err(ParserError::InvalidSyntax(opt))
    }
}

fn parse_small_stmt(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Statement), ParserError> {
    match util::get_token(&opt)? {
        Token::Pass     => Ok((stream.next(), Statement::Pass)),
        Token::Global   => parse_global_stmt(stream.next(), stream),
        Token::Nonlocal => parse_nonlocal_stmt(stream.next(), stream),
        Token::Del      => parse_del_stmt(stream.next(), stream),
        Token::Assert   => parse_assert_stmt(stream.next(), stream),
        Token::Import   => parse_import_name(stream.next(), stream),
        Token::From     => parse_import_from(stream.next(), stream),
        ref token if util::valid_flow_stmt(&token) => {
            parse_flow_stmt(opt, stream)
        },
        _ => parse_expr_stmt(opt, stream)
    }
}

fn parse_expr_stmt(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Statement), ParserError> {
    let (opt, expr) = parse_test_list_star_expr(opt, stream)?;

    match util::get_token(&opt)? {
        Token::Colon => {
            let (opt, annotation, value) =
                parse_ann_assign(stream.next(), stream)?;

            Ok((opt, Statement::AnnAssign { target: expr, annotation, value }))
        },
        Token::Assign => {
            let (opt, mut targets, value) =
                parse_assign(stream.next(), stream)?;

            targets.insert(0, expr);
            Ok((opt, Statement::Assign { targets, value }))
        },
        ref token if util::valid_aug_assign(&token) => {
            let (opt, op) = parse_aug_assign(opt, stream)?;
            let (opt, value) = match util::get_token(&opt)? {
                Token::Yield => parse_yield_expr(stream.next(), stream)?,
                _ => parse_test_list(opt, stream)?
            };

            Ok((opt, Statement::AugAssign { target: expr, op, value }))
        },
        _ => Ok((opt, Statement::Expr { value: expr }))
    }
}

fn parse_assign(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Vec<Expression>, Expression), ParserError> {
    let (opt, expr) = match util::get_token(&opt)? {
        Token::Yield => parse_yield_expr(stream.next(), stream)?,
        _ => parse_test_list_star_expr(opt, stream)?
    };

    match util::get_token(&opt)? {
        Token::Assign => {
            let (opt, mut targets, value) =
                parse_assign(stream.next(), stream)?;

            targets.insert(0, expr);
            Ok((opt, targets, value))
        },
        _ => Ok((opt, vec![], expr))
    }
}

fn parse_ann_assign(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression, Option<Expression>), ParserError> {
    let token = util::get_token(&opt)?;
    if !util::valid_test_expr(&token) {
        return Err(ParserError::InvalidSyntax(opt))
    }

    let (opt, annotation) = parse_test_expr(opt, stream)?;
    match util::get_token(&opt)? {
        Token::Assign => {
            let opt = stream.next();
            let token = util::get_token(&opt)?;
            if !util::valid_test_expr(&token) {
                return Err(ParserError::InvalidSyntax(opt))
            }

            let (opt, value) = parse_test_expr(opt, stream)?;
            Ok((opt, annotation, Some(value)))
        },
        _ => Ok((opt, annotation, None))
    }
}

fn parse_test_list_star_expr(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    let (opt, expr) = match util::get_token(&opt)? {
        Token::Times => parse_star_expr(stream.next(), stream)?,
        _ => parse_test_expr(opt, stream)?
    };

    match util::get_token(&opt)? {
        Token::Comma => {
            let opt = stream.next();
            let token = util::get_token(&opt)?;

            if util::valid_test_star(&token) {
                let (opt, tup_expr) = parse_test_list_star_expr(opt, stream)?;
                let mut elts = match tup_expr {
                    Expression::Tuple { elts, .. } => elts,
                    expr => vec![expr]
                };

                elts.insert(0, expr);
                Ok((opt, Expression::Tuple { elts, ctx: ExprContext::Load }))
            } else {
                Ok((opt, Expression::Tuple { elts: vec![expr],
                    ctx: ExprContext::Load }))
            }
        },
        _ => Ok((opt, expr))
    }
}

fn parse_aug_assign(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Operator), ParserError> {
    let op = match util::get_token(&opt)? {
        Token::AssignPlus        => Operator::Add,
        Token::AssignMinus       => Operator::Sub,
        Token::AssignTimes       => Operator::Mult,
        Token::AssignAt          => Operator::MatMult,
        Token::AssignDivide      => Operator::Div,
        Token::AssignMod         => Operator::Mod,
        Token::AssignBitAnd      => Operator::BitAnd,
        Token::AssignBitOr       => Operator::BitOr,
        Token::AssignBitXor      => Operator::BitXor,
        Token::AssignLshift      => Operator::LShift,
        Token::AssignRshift      => Operator::RShift,
        Token::AssignExponent    => Operator::Pow,
        Token::AssignDivideFloor => Operator::FloorDiv,
        _ => return Err(ParserError::InvalidSyntax(opt))
    };

    Ok((stream.next(), op))
}

fn parse_del_stmt(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Statement), ParserError> {
    if !util::valid_expr_list(&util::get_token(&opt)?) {
        return Err(ParserError::InvalidSyntax(opt))
    }

    // TODO validate the targets to ensure deletion of proper things
    // TODO pass in context as Del
    let (opt, targets) = parse_expr_list(opt, stream)?;
    Ok((opt, Statement::Delete { targets }))
}

fn parse_flow_stmt(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Statement), ParserError> {
    match util::get_token(&opt)? {
        Token::Break    => Ok((stream.next(), Statement::Break)),
        Token::Continue => Ok((stream.next(), Statement::Continue)),
        Token::Return   => parse_return_stmt(stream.next(), stream),
        Token::Raise    => parse_raise_stmt(stream.next(), stream),
        Token::Yield    => parse_yield_stmt(stream.next(), stream),
        _ => Err(ParserError::InvalidSyntax(opt))
    }
}

fn parse_return_stmt(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Statement), ParserError> {
    let token = util::get_token(&opt)?;

    if util::valid_test_expr(&token) {
        let (opt, test_list) = parse_test_list(opt, stream)?;
        Ok((opt, Statement::Return { value: Some(test_list) }))
    } else {
        Ok((opt, Statement::Return { value: None }))
    }
}

fn parse_yield_stmt(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Statement), ParserError> {
    let (opt, value) = parse_yield_expr(opt, stream)?;
    Ok((opt, Statement::Expr { value }))
}

fn parse_raise_stmt(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Statement), ParserError> {
    if util::valid_test_expr(&util::get_token(&opt)?) {
        let (opt, exc) = parse_test_expr(opt, stream)?;

        match util::get_token(&opt)? {
            Token::From => {
                let opt = stream.next();

                if !util::valid_test_expr(&util::get_token(&opt)?) {
                    return Err(ParserError::InvalidSyntax(opt))
                }

                let (opt, cause) = parse_test_expr(opt, stream)?;
                Ok((opt, Statement::Raise {
                    exc: Some(exc), cause: Some(cause) }))
            },
            _ => Ok((opt, Statement::Raise { exc: Some(exc), cause: None }))
        }
    } else {
        Ok((opt, Statement::Raise { exc: None, cause: None }))
    }
}

fn parse_import_name(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Statement), ParserError> {
    let (opt, names) = parse_dotted_as_names(opt, stream)?;
    Ok((opt, Statement::Import { names }))
}

fn parse_import_from(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Statement), ParserError> {
    let (opt, level) = parse_import_level(opt, stream)?;

    let (opt, module) = if util::valid_import_as_name(&util::get_token(&opt)?) {
        let (opt, names) = parse_dotted_name(opt, stream)?;
        (opt, Some(names.join(".")))
    } else {
        (opt, None)
    };

    let (opt, names) = match util::get_token(&opt)? {
        Token::Import => {
            let opt = stream.next();

            match util::get_token(&opt)? {
                Token::Times => (stream.next(), vec![Alias::Alias {
                    name: String::from("*"), asname: None }]),
                Token::Lparen => {
                    let (opt, names) =
                        parse_import_as_names(stream.next(), stream)?;

                    match util::get_token(&opt)? {
                        Token::Rparen => (stream.next(), names),
                        _ => return Err(ParserError::UnexpectedToken(
                            Token::Rparen, opt))
                    }
                },
                _ => parse_import_as_names(opt, stream)?
            }
        },
        _ => return Err(ParserError::UnexpectedToken(Token::Import, opt))
    };

    Ok((opt, Statement::ImportFrom { module, names, level }))
}

pub fn parse_import_level(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, usize), ParserError> {
    match util::get_token(&opt)? {
        Token::Dot => {
            let (opt, level) = parse_import_level(stream.next(), stream)?;
            Ok((opt, level + 1))
        },
        Token::Ellipsis => {
            let (opt, level) = parse_import_level(stream.next(), stream)?;
            Ok((opt, level + 3))
        },
        _ => Ok((opt, 0))
    }
}

fn parse_import_as_name(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Alias), ParserError> {
    let (opt, name) = match util::get_token(&opt)? {
        Token::Identifier(s) => (stream.next(), s),
        _ => return Err(ParserError::InvalidSyntax(opt))
    };
    let (opt, asname) = match util::get_token(&opt)? {
        Token::As => {
            let opt = stream.next();
            match util::get_token(&opt)? {
                Token::Identifier(s) => (stream.next(), Some(s)),
                _ => return Err(ParserError::InvalidSyntax(opt))
            }
        },
        _ => (opt, None)
    };

    Ok((opt, Alias::Alias { name, asname }))
}

fn parse_dotted_as_name(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Alias), ParserError> {
    let (opt, names) = parse_dotted_name(opt, stream)?;
    let name = names.join(".");

    match util::get_token(&opt)? {
        Token::As => {
            let opt = stream.next();
            let (opt, asname) = match util::get_token(&opt)? {
                Token::Identifier(s) => (stream.next(), s),
                _ => return Err(ParserError::InvalidSyntax(opt))
            };

            Ok((opt, Alias::Alias { name, asname: Some(asname) }))
        },
        _ => Ok((opt, Alias::Alias { name, asname: None }))
    }
}

// CPython's parser reports an error when a trailing comma appears with no
// parentheses. ex: "from module import a,b,c," we might want to also error out
// but I don't see a reason for doing so at this moment in time.
fn parse_import_as_names(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Vec<Alias>), ParserError> {
    if util::valid_import_as_name(&util::get_token(&opt)?) {
        let (opt, alias) = parse_import_as_name(opt, stream)?;

        match util::get_token(&opt)? {
            Token::Comma => {
                let (opt, mut aliases) =
                    parse_import_as_names(stream.next(), stream)?;

                aliases.insert(0, alias);
                Ok((opt, aliases))
            },
            _ => Ok((opt, vec![alias]))
        }
    } else {
        Ok((opt, vec![]))
    }
}

fn parse_dotted_as_names(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Vec<Alias>), ParserError> {
    let (opt, alias) = parse_dotted_as_name(opt, stream)?;

    match util::get_token(&opt)? {
        Token::Comma => {
            let (opt, mut names) =
                parse_dotted_as_names(stream.next(), stream)?;

            names.insert(0, alias);
            Ok((opt, names))
        },
        _ => Ok((opt, vec![alias]))
    }
}

// Returns a vec of strings, which can be joined with a '.', this is to
// anticipate any changes to parsing.
fn parse_dotted_name(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Vec<String>), ParserError> {
    let (opt, name) = match util::get_token(&opt)? {
        Token::Identifier(s) => (stream.next(), s),
        _ => return Err(ParserError::InvalidSyntax(opt))
    };

    match util::get_token(&opt)? {
        Token::Dot => {
            let (opt, mut names) = parse_dotted_name(stream.next(), stream)?;

            names.insert(0, name);
            Ok((opt, names))
        },
        _ => Ok((opt, vec![name]))
    }
}

// Functions similarly to `parse_dotted_name` but returns an attribute expr
// instead of a Vec of strings, this is useful for decorators
fn parse_dotted_name_attr(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    let (opt, value) = match util::get_token(&opt)? {
        Token::Identifier(id) =>
            (stream.next(), Expression::Name { id, ctx: ExprContext::Load }),
        _ => return Err(ParserError::InvalidSyntax(opt))
    };

    rec_parse_dotted_name_attr(opt, value, stream)
}

fn rec_parse_dotted_name_attr(opt: OptToken, expr: Expression,
    stream: &mut Lexer) -> Result<(OptToken, Expression), ParserError> {
    match util::get_token(&opt)? {
        Token::Dot => {
            match util::get_token(&stream.next())? {
                Token::Identifier(attr) => rec_parse_dotted_name_attr(
                    stream.next(),
                    Expression::Attribute {
                        value: Box::new(expr), attr, ctx: ExprContext::Load
                    },
                    stream
                ),
                _ => return Err(ParserError::InvalidSyntax(opt))
            }
        },
        _ => Ok((opt, expr))
    }
}

fn parse_global_stmt(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Statement), ParserError> {
    match util::get_token(&opt)? {
        Token::Identifier(name) => {
            let opt = stream.next();

            match util::get_token(&opt)? {
                Token::Comma => {
                    let (opt, stmt) =
                        parse_global_stmt(stream.next(), stream)?;
                    let mut names = match stmt {
                        Statement::Global { names } => names,
                        _ => unreachable!()
                    };

                    names.insert(0, name);
                    Ok((opt, Statement::Global { names }))
                },
                _ => Ok((opt, Statement::Global { names: vec![name] }))
            }
        },
        _ => return Err(ParserError::InvalidSyntax(opt))
    }
}

fn parse_nonlocal_stmt(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Statement), ParserError> {
    match util::get_token(&opt)? {
        Token::Identifier(name) => {
            let opt = stream.next();

            match util::get_token(&opt)? {
                Token::Comma => {
                    let (opt, stmt) =
                        parse_nonlocal_stmt(stream.next(), stream)?;
                    let mut names = match stmt {
                        Statement::Nonlocal { names } => names,
                        _ => unreachable!()
                    };

                    names.insert(0, name);
                    Ok((opt, Statement::Nonlocal { names }))
                },
                _ => Ok((opt, Statement::Nonlocal { names: vec![name] }))
            }
        }
        _ => return Err(ParserError::InvalidSyntax(opt))
    }
}

fn parse_assert_stmt(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Statement), ParserError> {
    if !util::valid_test_expr(&util::get_token(&opt)?) {
        return Err(ParserError::InvalidSyntax(opt))
    }

    let (opt, test) = parse_test_expr(opt, stream)?;
    match util::get_token(&opt)? {
        Token::Comma => {
            let opt = stream.next();

            if !util::valid_test_expr(&util::get_token(&opt)?) {
                return Err(ParserError::InvalidSyntax(opt))
            }

            let (opt, msg) = parse_test_expr(opt, stream)?;
            Ok((opt, Statement::Assert { test, msg: Some(msg) }))
        },
        _ => Ok((opt, Statement::Assert { test, msg: None }))
    }
}

fn parse_if_stmt(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Statement), ParserError> {
    let token = util::get_token(&opt)?;
    let (opt, test) = if util::valid_test_expr(&token) {
        parse_test_expr(opt, stream)?
    } else {
        return Err(ParserError::InvalidSyntax(opt))
    };
    let opt = match util::get_token(&opt)? {
        Token::Colon => stream.next(),
        _ => return Err(ParserError::UnexpectedToken(Token::Colon, opt))
    };
    let (opt, body) = parse_suite(opt, stream)?;

    // `compound_stmt` doesn't rely on an ending Newline token, we must
    // now check for EOF in some circumstance.
    if opt.is_none() {
        Ok((opt, Statement::If { test, body, orelse: vec![] }))
    } else {
        match util::get_token(&opt)? {
            Token::Elif => {
                let (opt, stmt) = parse_if_stmt(stream.next(), stream)?;
                Ok((opt, Statement::If { test, body, orelse: vec![stmt] }))
            },
            Token::Else => {
                let opt = match util::get_token(&stream.next())? {
                    Token::Colon => stream.next(),
                    _ => return Err(ParserError::UnexpectedToken(
                        Token::Colon, opt))
                };

                let (opt, orelse) = parse_suite(opt, stream)?;
                Ok((opt, Statement::If { test, body, orelse }))
            },
            _ => Ok((opt, Statement::If { test, body, orelse: vec![] }))
        }
    }
}

fn parse_while_stmt(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Statement), ParserError> {
    let token = util::get_token(&opt)?;
    let (opt, test) = if util::valid_test_expr(&token) {
        parse_test_expr(opt, stream)?
    } else {
        return Err(ParserError::InvalidSyntax(opt))
    };
    let opt = match util::get_token(&opt)? {
        Token::Colon => stream.next(),
        _ => return Err(ParserError::UnexpectedToken(Token::Colon, opt))
    };
    let (opt, body) = parse_suite(opt, stream)?;

    if opt.is_none() {
        Ok((opt, Statement::While { test, body, orelse: vec![] }))
    } else {
        match util::get_token(&opt)? {
            Token::Else => {
                let opt = match util::get_token(&stream.next())? {
                    Token::Colon => stream.next(),
                    _ => return Err(ParserError::UnexpectedToken(
                        Token::Colon, opt))
                };
                let (opt, orelse) = parse_suite(opt, stream)?;

                Ok((opt, Statement::While { test, body, orelse }))
            },
            _ => Ok((opt, Statement::While { test, body, orelse: vec![] }))
        }
    }
}

fn parse_for_stmt(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Statement), ParserError> {
    let (opt, mut expr_list) = parse_expr_list(opt, stream)?;
    let target = if expr_list.len() == 1 {
        expr_list.pop().unwrap()
    } else {
        Expression::Tuple { elts: expr_list, ctx: ExprContext::Store }
    };

    let opt = match util::get_token(&opt)? {
        Token::In => stream.next(),
        _ => return Err(ParserError::UnexpectedToken(Token::In, opt))
    };
    let (opt, iter) = parse_test_list(opt, stream)?;

    let opt = match util::get_token(&opt)? {
        Token::Colon => stream.next(),
        _ => return Err(ParserError::UnexpectedToken(Token::Colon, opt))
    };
    let (opt, body) = parse_suite(opt, stream)?;

    if opt.is_none() {
        Ok((opt, Statement::For {
            target, iter, body, orelse: vec![] }))
    } else {
        match util::get_token(&opt)? {
            Token::Else => {
                let opt = match util::get_token(&stream.next())? {
                    Token::Colon => stream.next(),
                    _ => return Err(ParserError::UnexpectedToken(
                        Token::Colon, opt))
                };
                let (opt, orelse) = parse_suite(opt, stream)?;

                Ok((opt, Statement::For { target, iter, body, orelse }))
            },
            _ => Ok((opt, Statement::For {
                target, iter, body, orelse: vec![] }))
        }
    }
}

fn parse_try_stmt(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Statement), ParserError> {
    let opt = match util::get_token(&opt)? {
        Token::Colon => stream.next(),
        _ => return Err(ParserError::UnexpectedToken(Token::Colon, opt))
    };
    let (opt, body) = parse_suite(opt, stream)?;

    match util::get_token(&opt)? {
        Token::Except => {
            let (opt, handlers) = parse_except_clauses(opt, stream)?;
            let (opt, orelse) = if opt.is_none() {
                (opt, vec![])
            } else {
                match util::get_token(&opt)? {
                    Token::Else => {
                        let opt = match util::get_token(&stream.next())? {
                            Token::Colon => stream.next(),
                            _ => return Err(ParserError::UnexpectedToken(
                                Token::Colon, opt))
                        };

                        parse_suite(opt, stream)?
                    },
                    _ => (opt, vec![])
                }
            };
            let (opt, finalbody) = if opt.is_none() {
                (opt, vec![])
            } else {
                match util::get_token(&opt)? {
                    Token::Finally => {
                        let opt = match util::get_token(&stream.next())? {
                            Token::Colon => stream.next(),
                            _ => return Err(ParserError::UnexpectedToken(
                                Token::Colon, opt))
                        };

                        parse_suite(opt, stream)?
                    },
                    _ => (opt, vec![])
                }
            };

            Ok((opt, Statement::Try { body, handlers, orelse, finalbody }))
        },
        Token::Finally => {
            let opt = match util::get_token(&stream.next())? {
                Token::Colon => stream.next(),
                _ => return Err(ParserError::UnexpectedToken(Token::Colon, opt))
            };
            let (opt, finalbody) = parse_suite(opt, stream)?;

            Ok((opt, Statement::Try { body, handlers: vec![],
                orelse: vec![], finalbody }))
        },
        _ => Err(ParserError::InvalidSyntax(opt))
    }
}

// The compiler can enforce the default exception being last
fn parse_except_clauses(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Vec<ExceptHandler>), ParserError> {
    match util::get_token(&opt)? {
        Token::Except => {
            let (opt, etype, name) =
                parse_except_clause(stream.next(), stream)?;
            let opt = match util::get_token(&opt)? {
                Token::Colon => stream.next(),
                _ => return Err(ParserError::UnexpectedToken(Token::Colon, opt))
            };

            let (opt, body) = parse_suite(opt, stream)?;
            let handler = ExceptHandler::ExceptHandler { etype, name, body };

            if opt.is_none() {
                Ok((opt, vec![handler]))
            } else {
                let (opt, mut handlers) = parse_except_clauses(opt, stream)?;

                handlers.insert(0, handler);
                Ok((opt, handlers))
            }
        },
        _ => Ok((opt, vec![]))
    }
}

fn parse_except_clause(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Option<Expression>, Option<String>), ParserError> {
    if util::valid_test_expr(&util::get_token(&opt)?) {
        let (opt, etype) = parse_test_expr(opt, stream)?;

        match util::get_token(&opt)? {
            Token::As => {
                let opt = stream.next();

                match util::get_token(&opt)? {
                    Token::Identifier(name) =>
                        Ok((stream.next(), Some(etype), Some(name))),
                    _ => return Err(ParserError::InvalidSyntax(opt))
                }
            },
            _ => Ok((opt, Some(etype), None))
        }
    } else {
        Ok((opt, None, None))
    }
}

fn parse_with_stmt(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Statement), ParserError> {
    let (opt, items) = parse_with_items(opt, stream)?;

    if items.is_empty() {
        return Err(ParserError::InvalidSyntax(opt))
    }

    let opt = match util::get_token(&opt)? {
        Token::Colon => stream.next(),
        _ => return Err(ParserError::UnexpectedToken(Token::Colon, opt))
    };
    let (opt, body) = parse_suite(opt, stream)?;

    Ok((opt, Statement::With { items, body }))
}

fn parse_with_items(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Vec<WithItem>), ParserError> {
    if util::valid_test_expr(&util::get_token(&opt)?) {
        let (opt, item) = parse_with_item(opt, stream)?;

        match util::get_token(&opt)? {
            Token::Comma => {
                let (opt, mut items) = parse_with_items(stream.next(), stream)?;

                items.insert(0, item);
                Ok((opt, items))
            },
            _ => Ok((opt, vec![item]))
        }
    } else {
        Ok((opt, vec![]))
    }
}

fn parse_with_item(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, WithItem), ParserError> {
    let (opt, context_expr) = parse_test_expr(opt, stream)?;

    match util::get_token(&opt)? {
        Token::As => {
            let (opt, expr) = parse_expr(stream.next(), stream)?;
            Ok((opt, WithItem::WithItem {
                context_expr, optional_vars: Some(expr) }))
        },
        _ => Ok((opt, WithItem::WithItem { context_expr, optional_vars: None }))
    }
}

fn parse_suite(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Vec<Statement>), ParserError> {
    match util::get_token(&opt)? {
        Token::Newline => {
            let opt = match util::get_token(&stream.next())? {
                Token::Indent => stream.next(),
                _ => return Err(ParserError::UnexpectedToken(
                    Token::Indent, opt))
            };

            if util::valid_stmt(&util::get_token(&opt)?) {
                let (opt, stmts) = parse_stmts(opt, stream)?;

                match util::get_token(&opt)? {
                    Token::Dedent => Ok((stream.next(), stmts)),
                    _ => return Err(ParserError::UnexpectedToken(
                        Token::Dedent, opt))
                }
            } else {
                Err(ParserError::InvalidSyntax(opt))
            }
        },
        token => {
            if util::valid_simple_stmt(&token) {
                let (opt, stmts) = parse_simple_stmt(opt, stream)?;
                Ok((opt, stmts))
            } else {
                Err(ParserError::InvalidSyntax(opt))
            }
        }
    }
}

fn parse_test_expr(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    match util::get_token(&opt)? {
        Token::Lambda => parse_lambda(stream.next(), parse_test_expr, stream),
        _ => {
            let (opt, expr) = parse_or_test(opt, stream)?;

            match util::get_token(&opt)? {
                Token::If => {
                    let (opt, guard) = parse_or_test(stream.next(), stream)?;
                    let opt = match util::get_token(&opt)? {
                        Token::Else => stream.next(),
                        _ => return Err(ParserError::UnexpectedToken(
                            Token::Else, opt))
                    };

                    let (opt, else_expr) = parse_test_expr(opt, stream)?;
                    Ok((opt, Expression::If { test: Box::new(guard), body:
                        Box::new(expr), orelse: Box::new(else_expr) }))
                },
                _ => Ok((opt, expr))
            }
        }
    }
}

fn parse_test_nocond(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    match util::get_token(&opt)? {
        Token::Lambda => parse_lambda(stream.next(), parse_test_nocond, stream),
        _ => parse_or_test(opt, stream)
    }
}

// `parse_lambda` covers both `parse_lambdef` and `parse_lambdef_nocond` rules
// which only vary by the body expression rule which is passed in as `parse_f`
fn parse_lambda(opt: OptToken,
    parse_f: fn(OptToken, &mut Lexer)
    -> Result<(OptToken, Expression), ParserError>, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    let arguments = Arguments::Arguments {
        args: vec![], vararg: None, kwonlyargs: vec![],
        kw_defaults: vec![], kwarg: None, defaults: vec![]
    };
    let (opt, varargslist) = parse_argslist(opt, parse_vfpdef, false,
        arguments, stream)?;
    let opt = match util::get_token(&opt)? {
        Token::Colon => stream.next(),
        _ => return Err(ParserError::UnexpectedToken(Token::Colon, opt))
    };
    let (opt, body) = if util::valid_test_expr(&util::get_token(&opt)?) {
        let (opt, expr) = parse_f(opt, stream)?;
        (opt, Box::new(expr))
    } else {
        return Err(ParserError::InvalidSyntax(opt))
    };

    Ok((opt, Expression::Lambda { args: Box::new(varargslist), body }))
}

fn parse_or_test(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    let (opt, expr) = parse_and_test(opt, stream)?;

    match util::get_token(&opt)? {
        Token::Or => {
            let (opt, mut values) = rec_parse_or_test(stream.next(), stream)?;

            values.insert(0, expr);
            Ok((opt, Expression::BoolOp { op: BoolOperator::Or, values }))
        },
        _ => Ok((opt, expr))
    }
}

fn rec_parse_or_test(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Vec<Expression>), ParserError> {
    let (opt, expr) = parse_and_test(opt, stream)?;

    match util::get_token(&opt)? {
        Token::Or => {
            let (opt, mut values) = rec_parse_or_test(stream.next(), stream)?;

            values.insert(0, expr);
            Ok((opt, values))
        },
        _ => Ok((opt, vec![expr]))
    }
}

fn parse_and_test(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    let (opt, expr) = parse_not_test(opt, stream)?;

    match util::get_token(&opt)? {
        Token::And => {
            let (opt, mut values) = rec_parse_and_test(stream.next(), stream)?;

            values.insert(0, expr);
            Ok((opt, Expression::BoolOp { op: BoolOperator::And, values }))
        },
        _ => Ok((opt, expr))
    }
}

fn rec_parse_and_test(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Vec<Expression>), ParserError> {
    let (opt, expr) = parse_not_test(opt, stream)?;

    match util::get_token(&opt)? {
        Token::And => {
            let (opt, mut values) = rec_parse_and_test(stream.next(), stream)?;

            values.insert(0, expr);
            Ok((opt, values))
        },
        _ => Ok((opt, vec![expr]))
    }
}

fn parse_not_test(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    match util::get_token(&opt)? {
        Token::Not => {
            let (opt, expr) = parse_not_test(stream.next(), stream)?;

            Ok((opt, Expression::UnaryOp {
                op: UnaryOperator::Not, operand: Box::new(expr) }))
        },
        _ => parse_comparison_expr(opt, stream)
    }
}

fn parse_comparison_expr(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    let (opt, expr) = parse_expr(opt, stream)?;
    let token = util::get_token(&opt)?;

    if util::valid_cmp_op(&token) {
        let (opt, ops, comparators) = rec_parse_comparison_expr(opt, stream)?;

        Ok((opt, Expression::Compare {
            left: Box::new(expr), ops, comparators }))
    } else {
        Ok((opt, expr))
    }
}

fn rec_parse_comparison_expr(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Vec<CmpOperator>, Vec<Expression>), ParserError> {
    let (opt, op) = util::get_cmp_op(&opt, stream)?;
    let (opt, expr) = parse_expr(opt, stream)?;
    let token = util::get_token(&opt)?;

    if util::valid_cmp_op(&token) {
        let (opt, mut ops, mut comparators) =
            rec_parse_comparison_expr(opt, stream)?;

        ops.insert(0, op);
        comparators.insert(0, expr);
        Ok((opt, ops, comparators))
    } else {
        Ok((opt, vec![op], vec![expr]))
    }
}

// Basically a wrapper for `parse_expr` that returns Expression::Starred,
// the check for an asterisk should be done prior to calling this function
fn parse_star_expr(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    if !util::valid_expr(&util::get_token(&opt)?) {
        return Err(ParserError::InvalidSyntax(opt))
    }

    let (opt, expr) = parse_expr(opt, stream)?;
    Ok((opt, Expression::Starred { value: Box::new(expr),
        ctx: ExprContext::Load }))
}

fn parse_expr(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    let (opt, expr) = parse_xor_expr(opt, stream)?;

    match util::get_token(&opt)? {
        Token::BitOr => {
            let (opt, right_expr) = parse_expr(stream.next(), stream)?;

            Ok((opt, Expression::BinOp { left: Box::new(expr),
                op: Operator::BitOr, right: Box::new(right_expr) }))
        },
        _ => Ok((opt, expr))
    }
}

fn parse_xor_expr(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    let (opt, expr) = parse_and_expr(opt, stream)?;

    match util::get_token(&opt)? {
        Token::BitXor => {
            let (opt, right_expr) = parse_xor_expr(stream.next(), stream)?;

            Ok((opt, Expression::BinOp { left: Box::new(expr),
                op: Operator::BitXor, right: Box::new(right_expr) }))
        },
        _ => Ok((opt, expr))
    }
}

fn parse_and_expr(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    let (opt, expr) = parse_shift_expr(opt, stream)?;

    match util::get_token(&opt)? {
        Token::BitAnd => {
            let (opt, right_expr) = parse_and_expr(stream.next(), stream)?;

            Ok((opt, Expression::BinOp { left: Box::new(expr),
                op: Operator::BitAnd, right: Box::new(right_expr) }))
        },
        _ => Ok((opt, expr))
    }
}

fn parse_shift_expr(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    let (opt, expr) = parse_arith_expr(opt, stream)?;

    match util::get_shift_op(&opt) {
        Some(op) => {
            let (opt, right_expr) = parse_shift_expr(stream.next(), stream)?;

            Ok((opt, Expression::BinOp { left: Box::new(expr),
                op, right: Box::new(right_expr) }))
        }
        None => Ok((opt, expr))
    }
}

fn parse_arith_expr(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    let (opt, expr) = parse_term(opt, stream)?;

    match util::get_arith_op(&opt) {
        Some(op) => {
            let (opt, right_expr) = parse_arith_expr(stream.next(), stream)?;

            Ok((opt, Expression::BinOp { left: Box::new(expr),
                op, right: Box::new(right_expr) }))
        },
        None => Ok((opt, expr))
    }
}

fn parse_term(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    let (opt, expr) = parse_factor(opt, stream)?;

    match util::get_term_op(&opt) {
        Some(op) => {
            let (opt, right_expr) = parse_term(stream.next(), stream)?;

            Ok((opt, Expression::BinOp { left: Box::new(expr),
                op, right: Box::new(right_expr) }))
        },
        None => Ok((opt, expr))
    }
}

fn parse_factor(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    match util::get_factor_op(&opt) {
        Some(op) => {
            let (opt, operand) = parse_factor(stream.next(), stream)?;

            Ok((opt, Expression::UnaryOp { op, operand: Box::new(operand) }))
        },
        None => parse_power(opt, stream)
    }
}

fn parse_power(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    let (opt, expr) = parse_atom_expr(opt, stream)?;

    match util::get_token(&opt)? {
        Token::Exponent => {
            let (opt, right_expr) = parse_factor(stream.next(), stream)?;

            Ok((opt, Expression::BinOp { left: Box::new(expr),
                op: Operator::Pow, right: Box::new(right_expr) }))
        },
        _ => Ok((opt, expr))
    }
}

fn parse_atom_expr(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    let (opt, expr) = parse_atom(opt, stream)?;
    parse_atom_trailer(opt, expr, stream)
}

fn parse_atom(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    match util::get_token(&opt)? {
        Token::Lparen => {
            let opt = stream.next();
            let token = util::get_token(&opt)?;
            let (opt, expr) = if util::valid_atom_paren(&token) {
                match token {
                    Token::Yield => parse_yield_expr(stream.next(), stream)?,
                    _ => parse_test_list_comp(opt, TLCompType::Tuple, stream)?
                }
            } else {
                (opt, Expression::Tuple {elts: vec![], ctx: ExprContext::Load})
            };

            match util::get_token(&opt)? {
                Token::Rparen => Ok((stream.next(), expr)),
                _ => Err(ParserError::UnexpectedToken(Token::Rparen, opt))
            }
        },
        Token::Lbracket => {
            let opt = stream.next();
            let token = util::get_token(&opt)?;
            let (opt, expr) = if util::valid_test_star(&token) {
                parse_test_list_comp(opt, TLCompType::List, stream)?
            } else {
                (opt, Expression::List { elts: vec![], ctx: ExprContext::Load })
            };

            match util::get_token(&opt)? {
                Token::Rbracket => Ok((stream.next(), expr)),
                _ => Err(ParserError::UnexpectedToken(Token::Rbracket, opt))
            }
        },
        Token::Lbrace => {
            let opt = stream.next();
            let token = util::get_token(&opt)?;
            let (opt, expr) = if util::valid_dict_set_maker(&token) {
                parse_dict_set_maker(opt, stream)?
            } else {
                // There is no empty set literal, default to dict
                (opt, Expression::Dict { keys: vec![], values: vec![] })
            };

            match util::get_token(&opt)? {
                Token::Rbrace => Ok((stream.next(), expr)),
                _ => Err(ParserError::UnexpectedToken(Token::Rbrace, opt))
            }
        },
        Token::Identifier(id) => Ok((stream.next(),
            Expression::Name { id, ctx: ExprContext::Load })),
        Token::DecInteger(n) =>
            Ok((stream.next(), Expression::Num { n: Number::DecInteger(n) })),
        Token::BinInteger(n) =>
            Ok((stream.next(), Expression::Num { n: Number::BinInteger(n) })),
        Token::OctInteger(n) =>
            Ok((stream.next(), Expression::Num { n: Number::OctInteger(n) })),
        Token::HexInteger(n) =>
            Ok((stream.next(), Expression::Num { n: Number::HexInteger(n) })),
        Token::Float(n) =>
            Ok((stream.next(), Expression::Num { n: Number::Float(n) })),
        Token::Imaginary(n) =>
            Ok((stream.next(), Expression::Num { n: Number::Imaginary(n) })),
        Token::String(s) => Ok((stream.next(), Expression::Str { s })),
        Token::Ellipsis => Ok((stream.next(), Expression::Ellipsis)),
        Token::None => Ok((stream.next(),
                Expression::NameConstant { value: Singleton::None })),
        Token::True => Ok((stream.next(),
                Expression::NameConstant { value: Singleton::True })),
        Token::False => Ok((stream.next(),
                Expression::NameConstant { value: Singleton::False })),
        _ => Err(ParserError::InvalidSyntax(opt))
    }
}

fn parse_test_list_comp(opt: OptToken, ctype: TLCompType, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    let (opt, expr) = match util::get_token(&opt)? {
        Token::Times => parse_star_expr(stream.next(), stream)?,
        _ => parse_test_expr(opt, stream)?
    };

    match util::get_token(&opt)? {
        Token::Comma => {
            let (opt, mut elts) =
                rec_parse_test_list_comp(stream.next(), stream)?;

            elts.insert(0, expr);
            match ctype {
                TLCompType::Tuple =>
                    Ok((opt, Expression::Tuple { elts,
                        ctx: ExprContext::Load })),
                TLCompType::List  =>
                    Ok((opt, Expression::List { elts,
                        ctx: ExprContext::Load }))
            }
        },
        Token::For => {
            match ctype {
                TLCompType::Tuple => parse_comp_for(stream.next(),
                    Expression::Generator { elt: Box::new(expr),
                    generators: vec![] }, stream),
                TLCompType::List  => parse_comp_for(stream.next(),
                    Expression::ListComp { elt: Box::new(expr),
                    generators: vec![] }, stream)
            }
        },
        _ => Ok((opt, expr))
    }
}

// Gets the list of test/star expressions for a non-comprehension descent
fn rec_parse_test_list_comp(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Vec<Expression>), ParserError> {
    let token = util::get_token(&opt)?;

    if util::valid_test_star(&token) {
        let (opt, expr) = match token {
            Token::Times => parse_star_expr(stream.next(), stream)?,
            _ => parse_test_expr(opt, stream)?
        };

        match util::get_token(&opt)? {
            Token::Comma => {
                let (opt, mut exprs) =
                    rec_parse_test_list_comp(stream.next(), stream)?;

                exprs.insert(0, expr);
                Ok((opt, exprs))
            },
            _ => Ok((opt, vec![expr]))
        }
    } else {
        Ok((opt, vec![]))
    }
}

fn parse_atom_trailer(opt: OptToken, expr: Expression, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    match util::get_token(&opt)? {
        Token::Lparen => {
            let (opt, args, keywords) = parse_arglist(stream.next(), stream)?;

            match util::get_token(&opt)? {
                Token::Rparen => parse_atom_trailer(stream.next(),
                    Expression::Call { func: Box::new(expr), args, keywords },
                    stream),
                _ => Err(ParserError::UnexpectedToken(Token::Rparen, opt))
            }
        },
        Token::Lbracket => {
            let (opt, slice) = parse_subscript_list(stream.next(), stream)?;

            match util::get_token(&opt)? {
                Token::Rbracket => parse_atom_trailer(stream.next(),
                    Expression::Subscript {
                        value: Box::new(expr), slice: Box::new(slice),
                        ctx: ExprContext::Load
                    },
                    stream),
                _ => Err(ParserError::UnexpectedToken(Token::Rbracket, opt))
            }
        },
        Token::Dot => {
            match util::get_token(&stream.next())? {
                Token::Identifier(attr) => parse_atom_trailer(stream.next(),
                    Expression::Attribute {
                        value: Box::new(expr), attr, ctx: ExprContext::Load
                    },
                    stream
                ),
                _ => Err(ParserError::InvalidSyntax(opt))
            }
        },
        _ => Ok((opt, expr))
    }
}

fn parse_subscript_list(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Slice), ParserError> {
    let (opt, slice) = parse_subscript(opt, stream)?;
    // We need to keep track of a trailing comma and only one subscript.
    let trailing_comma = match util::get_token(&opt)? {
        Token::Comma => true,
        _ => false
    };
    let opt = if trailing_comma { stream.next() } else { opt };
    let (opt, mut slices) = rec_parse_subscript_list(opt, stream)?;

    if slices.is_empty() {
        if trailing_comma {
            match slice {
                Slice::Slice { .. } => {
                    Ok((opt, Slice::ExtSlice { dims: vec![slice] }))
                },
                Slice::Index { value } => {
                    Ok((opt, Slice::Index { value: Expression::Tuple {
                        elts: vec![value], ctx: ExprContext::Load } }))
                },
                _ => unreachable!()
            }
        } else {
            Ok((opt, slice))
        }
    } else {
        slices.insert(0, slice);

        // If a Slice::Slice is found then we need to create an ExtSlice
        // instead of a Index with a Tuple
        let mut contains_slice = false;
        for s in slices.iter() {
            match *s {
                Slice::Slice { .. } => contains_slice = true,
                _ => ()
            }
        };

        if contains_slice {
            Ok((opt, Slice::ExtSlice { dims: slices }))
        } else {
            // Consumes `slices` variable
            let expr_list = slices.into_iter().map(|s| {
                match s {
                    Slice::Index { value } => value,
                    _ => panic!("parser error: expected Index values only")
                }
            }).collect();

            Ok((opt, Slice::Index { value: Expression::Tuple {
                elts: expr_list, ctx: ExprContext::Load } }))
        }
    }
}

fn rec_parse_subscript_list(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Vec<Slice>), ParserError> {
    if util::valid_subscript(&util::get_token(&opt)?) {
        let (opt, slice) = parse_subscript(opt, stream)?;

        match util::get_token(&opt)? {
            Token::Comma => {
                let (opt, mut slices) =
                    rec_parse_subscript_list(stream.next(), stream)?;

                slices.insert(0, slice);
                Ok((opt, slices))
            },
            _ => Ok((opt, vec![slice]))
        }
    } else {
        Ok((opt, vec![]))
    }
}

fn parse_subscript(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Slice), ParserError> {
    let token = util::get_token(&opt)?;
    let (opt, lower) = if util::valid_test_expr(&token) {
        let (opt, expr) = parse_test_expr(opt, stream)?;
        (opt, Some(expr))
    } else {
        (opt, None)
    };

    match util::get_token(&opt)? {
        Token::Colon => {
            let opt = stream.next();
            let token = util::get_token(&opt)?;
            let (opt, upper) = if util::valid_test_expr(&token) {
                let (opt, expr) = parse_test_expr(opt, stream)?;
                (opt, Some(expr))
            } else {
                (opt, None)
            };
            let (opt, step) = parse_sliceop(opt, stream)?;

            Ok((opt, Slice::Slice { lower, upper, step }))
        },
        _ => Ok((opt, Slice::Index { value: lower.unwrap() }))
    }
}

fn parse_sliceop(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Option<Expression>), ParserError> {
    match util::get_token(&opt)? {
        Token::Colon => {
            let opt = stream.next();
            let token = util::get_token(&opt)?;

            if util::valid_test_expr(&token) {
                let (opt, expr) = parse_test_expr(opt, stream)?;
                Ok((opt, Some(expr)))
            } else {
                Ok((opt, None))
            }
        },
        _ => Ok((opt, None))
    }
}

// Returns a Vec since there are multiple Expression values that wrap the
// expression list. If a Vec of size one is returned, the contained
// Expression might be pulled out.
fn parse_expr_list(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Vec<Expression>), ParserError> {
    let (opt, expr) = match util::get_token(&opt)? {
        Token::Times => parse_star_expr(stream.next(), stream)?,
        _ => parse_expr(opt, stream)?
    };

    match util::get_token(&opt)? {
        Token::Comma => {
            let opt = stream.next();
            let token = util::get_token(&opt)?;

            if util::valid_expr_list(&token) {
                let (opt, mut exprs) = parse_expr_list(opt, stream)?;

                exprs.insert(0, expr);
                Ok((opt, exprs))
            } else {
                // Trailing comma case
                Ok((opt, vec![expr]))
            }
        },
        _ => Ok((opt, vec![expr]))
    }
}

fn parse_test_list(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    let (opt, test_expr) = parse_test_expr(opt, stream)?;

    match util::get_token(&opt)? {
        Token::Comma => {
            let opt = stream.next();
            let token = util::get_token(&opt)?;

            if util::valid_test_expr(&token) {
                let (opt, expr) = parse_test_list(opt, stream)?;
                let mut elts = match expr {
                    Expression::Tuple { elts, .. } => elts,
                    _ => vec![expr]
                };

                elts.insert(0, test_expr);
                Ok((opt, Expression::Tuple { elts: elts,
                    ctx: ExprContext::Load }))
            } else {
                Ok((opt, Expression::Tuple { elts: vec![test_expr],
                    ctx: ExprContext::Load }))
            }
        },
        _ => Ok((opt, test_expr))
    }
}

fn parse_dict_set_maker(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    // Get the first expression value and determine if a dict|set is being made
    let (opt, expr, value, is_dict) = match util::get_token(&opt)? {
        Token::Exponent => {
            let (opt, expr) = parse_expr(stream.next(), stream)?;
            (opt, Expression::None, Some(expr), true)
        },
        Token::Times => {
            let (opt, expr) = parse_star_expr(stream.next(), stream)?;
            (opt, expr, None, false)
        }
        _ => {
            let (opt, expr) = parse_test_expr(opt, stream)?;

            match util::get_token(&opt)? {
                Token::Colon => {
                    let opt = stream.next();
                    let token = util::get_token(&opt)?;

                    if !util::valid_test_expr(&token) {
                        return Err(ParserError::InvalidSyntax(opt))
                    }

                    let (opt, value) = parse_test_expr(opt, stream)?;
                    (opt, expr, Some(value), true)
                },
                _ => (opt, expr, None, false)
            }
        }
    };

    if is_dict {
        parse_dict_maker(opt, expr, value.unwrap(), stream)
    } else {
        parse_set_maker(opt, expr, stream)
    }
}

fn parse_dict_maker(opt: OptToken, key: Expression, value: Expression,
    stream: &mut Lexer) -> Result<(OptToken, Expression), ParserError> {
    match util::get_token(&opt)? {
        Token::Comma => {
            let (opt, mut keys, mut values) =
                rec_parse_dict_maker(stream.next(), stream)?;

            keys.insert(0, key);
            values.insert(0, value);
            Ok((opt, Expression::Dict { keys, values }))
        },
        Token::For => {
            // TODO error if dict unpacking was matched above
            parse_comp_for(stream.next(),
                Expression::DictComp { key: Box::new(key),
                value: Box::new(value), generators: vec![] }, stream)
        },
        _ => Ok((opt, Expression::Dict {
            keys: vec![key], values: vec![value] }))
    }
}

fn rec_parse_dict_maker(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Vec<Expression>, Vec<Expression>), ParserError> {
    let token = util::get_token(&opt)?;

    if util::valid_dict_maker(&token) {
        let (opt, key, value) = match util::get_token(&opt)? {
            Token::Exponent => {
                let (opt, expr) = parse_expr(stream.next(), stream)?;
                (opt, Expression::None, expr)
            },
            _ => {
                let (opt, key) = parse_test_expr(opt, stream)?;
                let opt = match util::get_token(&opt)? {
                    Token::Colon => stream.next(),
                    _ => return Err(ParserError::UnexpectedToken(
                        Token::Colon, opt))
                };
                let token = util::get_token(&opt)?;

                if !util::valid_test_expr(&token) {
                    return Err(ParserError::InvalidSyntax(opt))
                }

                let (opt, value) = parse_test_expr(opt, stream)?;
                (opt, key, value)
            }
        };

        match util::get_token(&opt)? {
            Token::Comma => {
                let (opt, mut keys, mut values) =
                    rec_parse_dict_maker(stream.next(), stream)?;

                keys.insert(0, key);
                values.insert(0, value);
                Ok((opt, keys, values))
            },
            _ => Ok((opt, vec![key], vec![value]))
        }
    } else {
        Ok((opt, vec![], vec![]))
    }
}

fn parse_set_maker(opt: OptToken, expr: Expression, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    match util::get_token(&opt)? {
        Token::Comma => {
            // Reuse the testlist comp builder since it's the same pattern
            let (opt, mut elts) =
                rec_parse_test_list_comp(stream.next(), stream)?;

            elts.insert(0, expr);
            Ok((opt, Expression::Set { elts }))
        },
        Token::For => {
            parse_comp_for(stream.next(),
                Expression::SetComp { elt: Box::new(expr),
                generators: vec![] }, stream)
        },
        _ => Ok((opt, Expression::Set { elts: vec![expr] }))
    }
}

fn parse_class_def(opt: OptToken, decorator_list: Vec<Expression>,
    stream: &mut Lexer) -> Result<(OptToken, Statement), ParserError> {
    let (opt, name) = match util::get_token(&opt)? {
        Token::Identifier(name) => (stream.next(), name),
        _ => return Err(ParserError::InvalidSyntax(opt))
    };
    let (opt, bases, keywords) = match util::get_token(&opt)? {
        Token::Lparen => {
            let (opt, bases, keywords) = parse_arglist(stream.next(), stream)?;

            match util::get_token(&opt)? {
                Token::Rparen => (stream.next(), bases, keywords),
                _ => return Err(ParserError::UnexpectedToken(
                    Token::Rparen, opt))
            }
        }
        _ => (opt, vec![], vec![])
    };
    let (opt, body) = match util::get_token(&opt)? {
        Token::Colon => parse_suite(stream.next(), stream)?,
        _ => return Err(ParserError::UnexpectedToken(Token::Colon, opt))
    };

    Ok((opt, Statement::ClassDef {
        name, bases, keywords, body, decorator_list }))
}

// Wrapper to abstract tail-recursion from caller
fn parse_arglist(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Vec<Expression>, Vec<Keyword>), ParserError> {
    rec_parse_arglist(opt, vec![], vec![], stream)
}

fn rec_parse_arglist(opt: OptToken, mut args: Vec<Expression>,
    mut keywords: Vec<Keyword>, stream: &mut Lexer)
    -> Result<(OptToken, Vec<Expression>, Vec<Keyword>), ParserError> {
    if util::valid_argument(&util::get_token(&opt)?) {
        let (opt, expr, arg, arg_type) = parse_argument(opt, stream)?;

        match arg_type {
            ArgType::Positional => {
                if !keywords.is_empty() {
                    return Err(ParserError::PositionalArgAfterKeyword(opt))
                }
                args.push(expr)
            },
            ArgType::Keyword => {
                keywords.push(Keyword::Keyword { arg, value: expr })
            }
        }

        let opt = match util::get_token(&opt)? {
            Token::Comma => stream.next(),
            _ => opt
        };

        rec_parse_arglist(opt, args, keywords, stream)
    } else {
        Ok((opt, args, keywords))
    }
}

fn parse_argument(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression, Option<String>, ArgType), ParserError> {
    match util::get_token(&opt)? {
        Token::Exponent => {
            let (opt, expr) = parse_test_expr(stream.next(), stream)?;

            Ok((opt, expr, None, ArgType::Keyword))
        },
        Token::Times => {
            let (opt, expr) = parse_test_expr(stream.next(), stream)?;

            Ok((opt, Expression::Starred { value: Box::new(expr),
                ctx: ExprContext::Load }, None, ArgType::Positional))
        },
        _ => {
            let (opt, expr) = parse_test_expr(opt, stream)?;

            match util::get_token(&opt)? {
                Token::Assign => {
                    let (opt, value) = parse_test_expr(stream.next(), stream)?;
                    let arg = match expr {
                        Expression::Name { id, .. } => id,
                        _ => return Err(ParserError::KeywordExpression(opt))
                    };

                    Ok((opt, value, Some(arg), ArgType::Keyword))
                },
                Token::For => {
                    // The Async token could come before For if we support it,
                    // in that case we may want to add a pattern to match
                    let (opt, expr) = parse_comp_for(stream.next(),
                        Expression::Generator { elt: Box::new(expr),
                        generators: vec![] }, stream)?;
                    Ok((opt, expr, None, ArgType::Positional))
                },
                _ => Ok((opt, expr, None, ArgType::Positional))
            }
        }
    }
}

fn parse_comp_iter(opt: OptToken, gc_expr: Expression, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    match util::get_token(&opt)? {
        Token::For => parse_comp_for(stream.next(), gc_expr, stream),
        Token::If  => parse_comp_if(stream.next(), gc_expr, stream),
        _ => Ok((opt, gc_expr))
    }
}

// Returns updated Generator/Comp, it's up to the caller to supply this method
// with a Expression::(Generator|*Comp) that will be "filled"
fn parse_comp_for(opt: OptToken, gc_expr: Expression, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    let (opt, mut expr_list) = parse_expr_list(opt, stream)?;
    let token = util::get_token(&opt)?;

    if token != Token::In {
        return Err(ParserError::UnexpectedToken(Token::In, opt))
    }
    let (opt, iter) = parse_or_test(stream.next(), stream)?;

    // Comprehensions deal with Tuples so if the size of the list is greater
    // than one element we make it a tuple. The `del` keyword expects an
    // exprlist in the grammar but doesn't expect a Tuple, therefore we let the
    // caller manage the Expression type.
    let target = if expr_list.len() == 1 {
        expr_list.pop().unwrap()
    } else {
        Expression::Tuple { elts: expr_list, ctx: ExprContext::Load }
    };
    let comp = Comprehension::Comprehension { target, iter, ifs: vec![] };

    match gc_expr {
        Expression::Generator { elt, mut generators } => {
            generators.push(comp);
            parse_comp_iter(opt,
                Expression::Generator { elt, generators }, stream)
        },
        Expression::ListComp { elt, mut generators }  => {
            generators.push(comp);
            parse_comp_iter(opt,
                Expression::ListComp { elt, generators }, stream)
        },
        Expression::SetComp { elt, mut generators }  => {
            generators.push(comp);
            parse_comp_iter(opt,
                Expression::SetComp { elt, generators }, stream)
        },
        Expression::DictComp { key, value, mut generators }  => {
            generators.push(comp);
            parse_comp_iter(opt,
                Expression::DictComp { key, value, generators }, stream)
        },
        _ => Err(ParserError::InvalidSyntax(opt))
    }
}

// Modifies the most recent Comprehension within the generators
fn parse_comp_if(opt: OptToken, gc_expr: Expression, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    let (opt, expr) = parse_test_nocond(opt, stream)?;

    // TODO refactor this in a Rust-esque manner to limit repeated code
    match gc_expr {
        Expression::Generator { elt, mut generators } => {
            let (target, iter, mut ifs) = match generators.pop().unwrap() {
                Comprehension::Comprehension { target, iter, ifs } =>
                    (target, iter, ifs)
            };

            ifs.push(expr);
            generators.push(Comprehension::Comprehension { target, iter, ifs });
            parse_comp_iter(opt,
                Expression::Generator { elt, generators }, stream)
        },
        Expression::ListComp { elt, mut generators } => {
            let (target, iter, mut ifs) = match generators.pop().unwrap() {
                Comprehension::Comprehension { target, iter, ifs } =>
                    (target, iter, ifs)
            };

            ifs.push(expr);
            generators.push(Comprehension::Comprehension { target, iter, ifs });
            parse_comp_iter(opt,
                Expression::ListComp { elt, generators }, stream)
        },
        Expression::SetComp { elt, mut generators } => {
            let (target, iter, mut ifs) = match generators.pop().unwrap() {
                Comprehension::Comprehension { target, iter, ifs } =>
                    (target, iter, ifs)
            };

            ifs.push(expr);
            generators.push(Comprehension::Comprehension { target, iter, ifs });
            parse_comp_iter(opt,
                Expression::SetComp { elt, generators }, stream)
        },
        Expression::DictComp { key, value, mut generators } => {
            let (target, iter, mut ifs) = match generators.pop().unwrap() {
                Comprehension::Comprehension { target, iter, ifs } =>
                    (target, iter, ifs)
            };

            ifs.push(expr);
            generators.push(Comprehension::Comprehension { target, iter, ifs });
            parse_comp_iter(opt,
                Expression::DictComp { key, value, generators }, stream)
        },
        _ => Err(ParserError::InvalidSyntax(opt))
    }
}

fn parse_yield_expr(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    if util::valid_yield_arg(&util::get_token(&opt)?) {
        parse_yield_arg(opt, stream)
    } else {
        Ok((opt, Expression::Yield { value: None }))
    }
}

fn parse_yield_arg(opt: OptToken, stream: &mut Lexer)
    -> Result<(OptToken, Expression), ParserError> {
    match util::get_token(&opt)? {
        Token::From => {
            let opt = stream.next();

            if !util::valid_test_expr(&util::get_token(&opt)?) {
                return Err(ParserError::InvalidSyntax(opt))
            }

            let (opt, expr) = parse_test_expr(opt, stream)?;
            Ok((opt, Expression::YieldFrom { value: Box::new(expr) }))
        },
        _ => {
            let (opt, expr) = parse_test_list(opt, stream)?;
            Ok((opt, Expression::Yield { value: Some(Box::new(expr)) }))
        }
    }
}
