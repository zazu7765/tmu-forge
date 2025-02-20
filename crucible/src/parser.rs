#![allow(dead_code)]
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while_m_n},
    character::complete::{alpha1, char, multispace0, multispace1},
    combinator::{opt, recognize},
    sequence::delimited,
    IResult, Parser,
};

#[derive(Debug, Clone, PartialEq)]
enum RequisiteGroup {
    Course(String),
    AllOf(Vec<RequisiteGroup>),
    AnyOf(Vec<RequisiteGroup>),
}

fn preprocess(input: &str, is_prereq: bool) -> String {
    input
        .replace("[", "(")
        .replace("]", ")")
        .replace(
            ",",
            match is_prereq {
                true => " and ",
                false => " or ",
            },
        )
        .replace("  ", " ")
        .trim()
        .to_ascii_uppercase()
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

fn term(input: &str) -> IResult<&str, RequisiteGroup> {
    alt((course_code.map(RequisiteGroup::Course), paren_delimited)).parse(input)
}

fn expr(input: &str) -> IResult<&str, RequisiteGroup> {
    let (input, (first, rest)) = (
        term,
        opt((multispace0, alt((tag("AND"), tag("OR"))), multispace0, expr)),
    )
        .parse(input)?;

    match rest {
        Some((_, op, _, next)) => match op {
            "AND" => Ok((input, RequisiteGroup::AllOf(vec![first, next]))),
            "OR" => Ok((input, RequisiteGroup::AnyOf(vec![first, next]))),
            _ => unreachable!(),
        },
        None => Ok((input, first)),
    }
}

fn paren_delimited(input: &str) -> IResult<&str, RequisiteGroup> {
    let (input, reqs) = delimited(
        char('('),
        delimited(multispace0, expr, multispace0),
        char(')'),
    )
    .parse(input)?;
    Ok((input, reqs))
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

    #[test]
    fn test_preprocess() {
        assert_eq!(preprocess("MTH101, PCS 125", true), "MTH101 AND PCS 125");
        assert_eq!(preprocess("A , B", false), "A OR B");
        assert_eq!(preprocess("A, B, C", true), "A AND B AND C");
        assert_eq!(preprocess("A, B, C", false), "A OR B OR C");
        assert_eq!(
            preprocess("[A and B and (C or D)]", false),
            "(A AND B AND (C OR D))"
        );
    }

    #[test]
    fn test_expr() {
        assert_eq!(
            expr("MTH 101 AND PCS 125").unwrap(),
            (
                "",
                RequisiteGroup::AllOf(vec![
                    RequisiteGroup::Course("MTH 101".to_string()),
                    RequisiteGroup::Course("PCS 125".to_string())
                ])
            )
        );

        assert_eq!(
            expr("(MTH 101) AND PCS 125").unwrap(),
            (
                "",
                RequisiteGroup::AllOf(vec![
                    RequisiteGroup::Course("MTH 101".to_string()),
                    RequisiteGroup::Course("PCS 125".to_string())
                ])
            )
        );
    }

    #[test]
    fn test_paren_delimited() {
        assert_eq!(
            paren_delimited("(MTH 101)"),
            Ok(("", RequisiteGroup::Course("MTH 101".to_string())))
        );

        assert_eq!(
            paren_delimited("(MTH 101 AND PCS 125)"),
            Ok((
                "",
                RequisiteGroup::AllOf(vec![
                    RequisiteGroup::Course("MTH 101".to_string()),
                    RequisiteGroup::Course("PCS 125".to_string())
                ])
            ))
        );

        assert_eq!(
            paren_delimited("(MTH 101 AND (PCS 125 OR CPS 109))"),
            Ok((
                "",
                RequisiteGroup::AllOf(vec![
                    RequisiteGroup::Course("MTH 101".to_string()),
                    RequisiteGroup::AnyOf(vec![
                        RequisiteGroup::Course("PCS 125".to_string()),
                        RequisiteGroup::Course("CPS 109".to_string())
                    ])
                ])
            ))
        );
    }

    #[test]
    fn test_complete_parser() {
        // Test parsing after preprocessing
        let input = preprocess("[CPS 305 AND MTH 108 AND (MTH 380 OR MTH 304)]", true);
        assert_eq!(
            expr(&input),
            Ok((
                "",
                // TODO: Fix this test and code to flatten like operators
                RequisiteGroup::AllOf(vec![
                    RequisiteGroup::Course("CPS 305".to_string()),
                    RequisiteGroup::AllOf(vec![
                        RequisiteGroup::Course("MTH 108".to_string()),
                        RequisiteGroup::AnyOf(vec![
                            RequisiteGroup::Course("MTH 380".to_string()),
                            RequisiteGroup::Course("MTH 304".to_string())
                        ])
                    ])
                ])
            ))
        );
    }
}
