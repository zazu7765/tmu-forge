#![allow(dead_code)]
use nom::{
    branch::alt,
    bytes::complete::take_while_m_n,
    character::complete::{alpha1, char, multispace1},
    combinator::{opt, recognize},
    IResult, Parser,
};

#[derive(Debug, Clone)]
enum RequisiteGroup {
    Course(String),
    AllOf(Vec<RequisiteGroup>),
    AnyOf(Vec<RequisiteGroup>),
}

fn course_code(input: &str) -> IResult<&str, String> {
    // Take 3 upper case letters
    let (input, dep) = take_while_m_n(3, 3, |c: char| c.is_ascii_uppercase())(input)?;
    // Take 1+ spaces
    let (input, _) = opt(multispace1).parse(input)?;
    // Take 3 digits
    let (input, code) = take_while_m_n(3, 3, |c: char| c.is_ascii_digit())(input)?;
    // Throw away weird course code semantics
    let (input, _) = opt(alt((
        recognize((alpha1, char('/'), alpha1)),
        recognize(alpha1),
    )))
    .parse(input)?;
    // Return the course code
    Ok((input, format!("{} {}", dep, code)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_course_code() {
        assert_eq!(course_code("MTH 101"), Ok(("", "MTH 101".to_string())));
        assert_eq!(course_code("PCS 102A"), Ok(("", "PCS 102".to_string())));
        assert_eq!(course_code("MTH 130A/B"), Ok(("", "MTH 130".to_string())));
    }
}
