use crate::{ast::*, interval_set::*, rel::*};
use inari::{dec_interval, DecoratedInterval};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, char, digit0, digit1, space0},
    combinator::{all_consuming, map, not, opt, peek, recognize, value, verify},
    error::VerboseError,
    multi::fold_many0,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    Err as NomErr, IResult,
};

type ParseResult<'a, O> = IResult<&'a str, O, VerboseError<&'a str>>;

fn decimal_literal(i: &str) -> ParseResult<&str> {
    alt((
        recognize(pair(digit1, opt(pair(char('.'), digit0)))),
        recognize(pair(char('.'), digit1)),
    ))(i)
}

fn keyword<'a>(kw: &'a str) -> impl Fn(&'a str) -> ParseResult<'a, &'a str> {
    terminated(
        tag(kw),
        not(verify(peek(anychar), |c| c.is_alphanumeric() || *c == '_')),
    )
}

fn primary_expr(i: &str) -> ParseResult<Expr> {
    alt((
        map(decimal_literal, |s| {
            let s = ["[", s, ",", s, "]"].concat();
            let x = TupperIntervalSet::from(dec_interval!(&s).unwrap());
            Expr::new(ExprKind::Constant(Box::new(x)))
        }),
        map(alt((keyword("pi"), keyword("π"))), |_| {
            let x = TupperIntervalSet::from(DecoratedInterval::PI);
            Expr::new(ExprKind::Constant(Box::new(x)))
        }),
        map(keyword("e"), |_| {
            let x = TupperIntervalSet::from(DecoratedInterval::E);
            Expr::new(ExprKind::Constant(Box::new(x)))
        }),
        value(Expr::new(ExprKind::X), keyword("x")),
        value(Expr::new(ExprKind::Y), keyword("y")),
        delimited(
            terminated(char('('), space0),
            expr,
            preceded(space0, char(')')),
        ),
        map(
            delimited(
                terminated(char('|'), space0),
                expr,
                preceded(space0, char('|')),
            ),
            |x| Expr::new(ExprKind::Unary(UnaryOp::Abs, Box::new(x))),
        ),
        map(
            delimited(
                terminated(char('⌈'), space0),
                expr,
                preceded(space0, char('⌉')),
            ),
            |x| Expr::new(ExprKind::Unary(UnaryOp::Ceil, Box::new(x))),
        ),
        map(
            delimited(
                terminated(char('⌊'), space0),
                expr,
                preceded(space0, char('⌋')),
            ),
            |x| Expr::new(ExprKind::Unary(UnaryOp::Floor, Box::new(x))),
        ),
    ))(i)
}

fn fn1(i: &str) -> ParseResult<UnaryOp> {
    alt((
        value(UnaryOp::Acosh, keyword("acosh")),
        value(UnaryOp::Asinh, keyword("asinh")),
        value(UnaryOp::Atanh, keyword("atanh")),
        value(UnaryOp::Floor, keyword("floor")),
        value(UnaryOp::Log10, keyword("log10")),
        value(UnaryOp::Acos, keyword("acos")),
        value(UnaryOp::Asin, keyword("asin")),
        value(UnaryOp::Atan, keyword("atan")),
        value(UnaryOp::Ceil, keyword("ceil")),
        value(UnaryOp::Cosh, keyword("cosh")),
        value(UnaryOp::Log2, keyword("log2")),
        value(UnaryOp::Sign, keyword("sign")),
        value(UnaryOp::Sinh, keyword("sinh")),
        value(UnaryOp::Sqrt, keyword("sqrt")),
        value(UnaryOp::Tanh, keyword("tanh")),
        value(UnaryOp::Cos, keyword("cos")),
        value(UnaryOp::Exp, keyword("exp")),
        value(UnaryOp::Ln, keyword("log")),
        value(UnaryOp::Sin, keyword("sin")),
        value(UnaryOp::Tan, keyword("tan")),
    ))(i)
}

fn fn2(i: &str) -> ParseResult<BinaryOp> {
    alt((
        value(BinaryOp::Atan2, keyword("atan2")),
        value(BinaryOp::Max, keyword("max")),
        value(BinaryOp::Min, keyword("min")),
        value(BinaryOp::Mod, keyword("mod")),
    ))(i)
}

fn postfix_expr(i: &str) -> ParseResult<Expr> {
    alt((
        map(
            pair(
                fn1,
                delimited(
                    delimited(space0, char('('), space0),
                    expr,
                    preceded(space0, char(')')),
                ),
            ),
            |(f, x)| Expr::new(ExprKind::Unary(f, Box::new(x))),
        ),
        map(
            pair(
                fn2,
                delimited(
                    delimited(space0, char('('), space0),
                    separated_pair(expr, delimited(space0, char(','), space0), expr),
                    preceded(space0, char(')')),
                ),
            ),
            |(f, (x, y))| Expr::new(ExprKind::Binary(f, Box::new(x), Box::new(y))),
        ),
        primary_expr,
    ))(i)
}

// ^ is right-associative: x^y^z is the same as x^(y^z).
fn power_expr(i: &str) -> ParseResult<Expr> {
    alt((
        map(
            separated_pair(
                postfix_expr,
                delimited(space0, char('^'), space0),
                unary_expr,
            ),
            |(x, y)| Expr::new(ExprKind::Binary(BinaryOp::Pow, Box::new(x), Box::new(y))),
        ),
        postfix_expr,
    ))(i)
}

fn unary_expr(i: &str) -> ParseResult<Expr> {
    alt((
        map(
            separated_pair(value(UnaryOp::Neg, char('-')), space0, unary_expr),
            |(op, x)| Expr::new(ExprKind::Unary(op, Box::new(x))),
        ),
        power_expr,
    ))(i)
}

fn multiplicative_expr(i: &str) -> ParseResult<Expr> {
    let (i, x) = unary_expr(i)?;

    fold_many0(
        alt((
            pair(
                delimited(
                    space0,
                    alt((
                        value(BinaryOp::Mul, char('*')),
                        value(BinaryOp::Div, char('/')),
                    )),
                    space0,
                ),
                unary_expr,
            ),
            pair(value(BinaryOp::Mul, space0), power_expr),
        )),
        x,
        |xs, (op, y)| Expr::new(ExprKind::Binary(op, Box::new(xs), Box::new(y))),
    )(i)
}

fn additive_expr(i: &str) -> ParseResult<Expr> {
    let (i, x) = multiplicative_expr(i)?;

    fold_many0(
        pair(
            delimited(
                space0,
                alt((
                    value(BinaryOp::Add, char('+')),
                    value(BinaryOp::Sub, char('-')),
                )),
                space0,
            ),
            multiplicative_expr,
        ),
        x,
        |xs, (op, y)| Expr::new(ExprKind::Binary(op, Box::new(xs), Box::new(y))),
    )(i)
}

fn expr(i: &str) -> ParseResult<Expr> {
    additive_expr(i)
}

fn equality(i: &str) -> ParseResult<Rel> {
    map(
        tuple((
            expr,
            delimited(
                space0,
                alt((
                    value(RelOp::Eq, tag("==")),
                    value(RelOp::Ge, tag(">=")),
                    value(RelOp::Gt, char('>')),
                    value(RelOp::Le, tag("<=")),
                    value(RelOp::Lt, char('<')),
                )),
                space0,
            ),
            expr,
        )),
        |(x, op, y)| Rel::new(RelKind::Atomic(op, Box::new(x), Box::new(y))),
    )(i)
}

fn primary_rel(i: &str) -> ParseResult<Rel> {
    alt((
        delimited(
            terminated(char('('), space0),
            rel,
            preceded(space0, char(')')),
        ),
        equality,
    ))(i)
}

fn and_rel(i: &str) -> ParseResult<Rel> {
    let (i, x) = primary_rel(i)?;

    fold_many0(
        preceded(delimited(space0, tag("&&"), space0), primary_rel),
        x,
        |xs, y| Rel::new(RelKind::And(Box::new(xs), Box::new(y))),
    )(i)
}

fn or_rel(i: &str) -> ParseResult<Rel> {
    let (i, x) = and_rel(i)?;

    fold_many0(
        preceded(delimited(space0, tag("||"), space0), and_rel),
        x,
        |xs, y| Rel::new(RelKind::Or(Box::new(xs), Box::new(y))),
    )(i)
}

fn rel(i: &str) -> ParseResult<Rel> {
    or_rel(i)
}

fn relation(i: &str) -> ParseResult<Rel> {
    delimited(space0, rel, space0)(i)
}

pub fn parse(i: &str) -> Result<Rel, String> {
    match all_consuming(relation)(i) {
        Ok(("", x)) => Ok(x),
        Err(NomErr::Error(e)) | Err(NomErr::Failure(e)) => Err(convert_error(i, e)),
        _ => unreachable!(),
    }
}

// Copied from `nom::error::convert_error`.
#[allow(clippy::naive_bytecount)]
fn convert_error(input: &str, e: VerboseError<&str>) -> String {
    use nom::Offset;

    let substring = e.errors.first().unwrap().0;
    let offset = input.offset(substring);

    let prefix = &input.as_bytes()[..offset];

    // Count the number of newlines in the first `offset` bytes of input
    let line_number = prefix.iter().filter(|&&b| b == b'\n').count() + 1;

    // Find the line that includes the subslice:
    // Find the *last* newline before the substring starts
    let line_begin = prefix
        .iter()
        .rev()
        .position(|&b| b == b'\n')
        .map(|pos| offset - pos)
        .unwrap_or(0);

    // Find the full line after that newline
    let line = input[line_begin..]
        .lines()
        .next()
        .unwrap_or(&input[line_begin..])
        .trim_end();

    // The (1-indexed) column number is the offset of our substring into that line
    let column_number = line.offset(substring) + 1;

    format!(
        "at line {line_number}:\n\
               {line}\n\
               {caret:>column$}\n\n",
        line_number = line_number,
        line = line,
        caret = '^',
        column = column_number,
    )
}
