use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{alpha1, digit1, multispace0},
    combinator::{map, map_res, opt, recognize},
    multi::{many0, separated_list1},
    sequence::{delimited, pair, preceded},
    IResult,
};
// use rust_decimal::Decimal;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Integer(i64),
    String(String),
    // Decimal(Decimal),
    Variable(String),
    Equal(Box<Expr>, Box<Expr>),
    NotEqual(Box<Expr>, Box<Expr>),
    LessThan(Box<Expr>, Box<Expr>),
    LessThanOrEqual(Box<Expr>, Box<Expr>),
    GreaterThan(Box<Expr>, Box<Expr>),
    GreaterThanOrEqual(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Predicate(String, Box<Expr>),
}

pub fn parse_number(input: &str) -> IResult<&str, Expr> {
    map_res(digit1, |s: &str| s.parse().map(Expr::Integer))(input)
}

pub fn parse_identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"), tag("$"))),
        opt(take_while(|c: char| {
            c.is_alphanumeric() || c == '_' || c == '-'
        })),
    ))(input)
}

pub fn parse_variable_path(input: &str) -> IResult<&str, String> {
    map(separated_list1(tag("."), parse_identifier), |parts| {
        parts.join(".")
    })(input)
}

pub fn parse_predicate(input: &str) -> IResult<&str, Expr> {
    delimited(
        preceded(multispace0, tag("[")),
        parse_expr,
        preceded(multispace0, tag("]")),
    )(input)
}

pub fn parse_variable_with_predicate(input: &str) -> IResult<&str, Expr> {
    let (input, var_path) = parse_variable_path(input)?;
    let (input, predicate) = opt(parse_predicate)(input)?;

    match predicate {
        Some(pred) => Ok((input, Expr::Predicate(var_path, Box::new(pred)))),
        None => Ok((input, Expr::Variable(var_path))),
    }
}

pub fn parse_atom(input: &str) -> IResult<&str, Expr> {
    alt((
        parse_number,
        parse_variable_with_predicate,
        delimited(
            preceded(multispace0, tag("(")),
            parse_expr,
            preceded(multispace0, tag(")")),
        ),
    ))(input)
}

pub fn parse_comparison(input: &str) -> IResult<&str, Expr> {
    let (input, left) = parse_atom(input)?;
    let (input, rest) = opt(pair(
        preceded(
            multispace0,
            alt((
                tag("=="),
                tag("!="),
                tag("<="),
                tag(">="),
                tag("<"),
                tag(">"),
            )),
        ),
        preceded(multispace0, parse_atom),
    ))(input)?;

    match rest {
        Some((op, right)) => {
            let expr = match op {
                "==" => Expr::Equal(Box::new(left), Box::new(right)),
                "!=" => Expr::NotEqual(Box::new(left), Box::new(right)),
                "<=" => Expr::LessThanOrEqual(Box::new(left), Box::new(right)),
                ">=" => Expr::GreaterThanOrEqual(Box::new(left), Box::new(right)),
                "<" => Expr::LessThan(Box::new(left), Box::new(right)),
                ">" => Expr::GreaterThan(Box::new(left), Box::new(right)),
                _ => unreachable!(),
            };
            Ok((input, expr))
        }
        None => Ok((input, left)),
    }
}

pub fn parse_and(input: &str) -> IResult<&str, Expr> {
    let (input, first) = parse_comparison(input)?;
    let (input, rest) = many0(preceded(
        multispace0,
        pair(tag("&&"), preceded(multispace0, parse_comparison)),
    ))(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, (_, expr)| {
            Expr::And(Box::new(acc), Box::new(expr))
        }),
    ))
}

pub fn parse_or(input: &str) -> IResult<&str, Expr> {
    let (input, first) = parse_and(input)?;
    let (input, rest) = many0(preceded(
        multispace0,
        pair(tag("||"), preceded(multispace0, parse_and)),
    ))(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, (_, expr)| {
            Expr::Or(Box::new(acc), Box::new(expr))
        }),
    ))
}

pub fn parse_expr(input: &str) -> IResult<&str, Expr> {
    parse_or(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_with_predicate() {
        let input_with_predicate = "abc.def[a == 5 && (b < 10 || c >= 20)]";
        match parse_variable_with_predicate(input_with_predicate) {
            Ok((remaining, expr)) => println!(
                "Parsed expression with predicate: {:?}\nRemaining: {:?}",
                expr, remaining
            ),
            Err(e) => println!("Error: {:?}", e),
        }
    }

    #[test]
    fn test_predicate() {
        let input_predicate = "[$1 == 5 && (b < 10 || c >= 20)]";
        match parse_predicate(input_predicate) {
            Ok((remaining, expr)) => {
                println!("Parsed predicate: {:?}\nRemaining: {:?}", expr, remaining)
            }
            Err(e) => println!("Error: {:?}", e),
        }
    }
}
