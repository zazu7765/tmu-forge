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
    let scan_dep = take_while_m_n(3, 3, |c: char| c.is_ascii_uppercase());
    // Take 3 digits
    let scan_code = take_while_m_n(3, 3, |c: char| c.is_ascii_digit());
    // Throw away weird course code semantics
    let ignore_extra = alt((recognize((alpha1, char('/'), alpha1)), recognize(alpha1)));

    // Combine and Parse
    let (input, (dep, _, code, _)) =
        (scan_dep, opt(multispace1), scan_code, opt(ignore_extra)).parse(input)?;
    Ok((input, format!("{} {}", dep, code)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_course_code() {
        assert!(course_code("ABC").is_err());
        assert!(course_code("ABC12").is_err());
        assert!(course_code("AB 123").is_err());
        assert!(course_code("123 ABC").is_err());
        assert!(course_code("abc 123").is_err());
        assert_eq!(course_code("MTH 101"), Ok(("", "MTH 101".to_string())));
        assert_eq!(course_code("PCS 102A"), Ok(("", "PCS 102".to_string())));
        assert_eq!(course_code("MTH 130A/B"), Ok(("", "MTH 130".to_string())));
    }
}
