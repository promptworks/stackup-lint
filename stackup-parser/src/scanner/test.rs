use super::*;
use combine::stream::easy::{Error, Errors};
use combine::stream::state::State;

#[test]
fn test_int_value() {
    let mut parser = int_value();
    let result = parser.parse(State::new("12345")).map(|x| x.0);
    let result_0 = parser.parse(State::new("0")).map(|x| x.0);
    let result_negative = parser.parse(State::new("-12345")).map(|x| x.0);
    let result_positive = parser.parse(State::new("+12345")).map(|x| x.0);
    let result_err = parser.parse(State::new("name")).map(|x| x.0);

    assert_eq!(
        result,
        Ok(Token {
            kind: TokenType::IntValue(12345),
            pos: SourcePosition::default()
        })
    );

    assert_eq!(
        result_0,
        Ok(Token {
            kind: TokenType::IntValue(0),
            pos: SourcePosition::default()
        })
    );

    assert_eq!(
        result_negative,
        Ok(Token {
            kind: TokenType::IntValue(-12345),
            pos: SourcePosition::default()
        })
    );

    assert_eq!(
        result_positive,
        Ok(Token {
            kind: TokenType::IntValue(12345),
            pos: SourcePosition::default()
        })
    );

    assert!(result_err.is_err());
}

#[test]
fn test_float_value() {
    let mut parser = float_value();
    let result = parser.parse(State::new("12345.0")).map(|x| x.0);
    let result_1 = parser.parse(State::new("-42.5909")).map(|x| x.0);
    let result_2 = parser.parse(State::new("0.32")).map(|x| x.0);
    let result_err = parser.parse(State::new("12345.")).map(|x| x.0);

    assert_eq!(
        result,
        Ok(Token {
            kind: TokenType::FloatValue(12345.0),
            pos: SourcePosition::default()
        })
    );

    assert_eq!(
        result_1,
        Ok(Token {
            kind: TokenType::FloatValue(-42.5909),
            pos: SourcePosition::default()
        })
    );

    assert_eq!(
        result_2,
        Ok(Token {
            kind: TokenType::FloatValue(0.32),
            pos: SourcePosition::default()
        })
    );

    assert!(result_err.is_err());
}

#[test]
fn test_name() {
    let mut parser = name();
    let foo_bar_result = parser.parse(State::new("foo_bar")).map(|x| x.0);
    let foo_bar_camel_result = parser.parse(State::new("fooBar")).map(|x| x.0);
    let result_err = parser.parse(State::new("0err")).map(|x| x.0);

    assert_eq!(
        foo_bar_result,
        Ok(Token {
            kind: TokenType::Name("foo_bar"),
            pos: SourcePosition::default()
        })
    );

    assert_eq!(
        foo_bar_camel_result,
        Ok(Token {
            kind: TokenType::Name("fooBar"),
            pos: SourcePosition::default()
        })
    );

    assert!(result_err.is_err());
}

#[test]
fn test_punctuator() {
    let mut parser = punctuator();
    let result_ellipsis = parser.parse(State::new("...")).map(|x| x.0);
    let result_bang = parser.parse(State::new("!")).map(|x| x.0);
    let result_v_bar = parser.parse(State::new("|")).map(|x| x.0);
    let result_err = parser.parse(State::new("not a punctuator")).map(|x| x.0);

    assert_eq!(
        result_ellipsis,
        Ok(Token {
            kind: TokenType::Punctuator(Punctuator::Ellipsis),
            pos: SourcePosition::default()
        })
    );

    assert_eq!(
        result_bang,
        Ok(Token {
            kind: TokenType::Punctuator(Punctuator::Bang),
            pos: SourcePosition::default()
        })
    );

    assert_eq!(
        result_v_bar,
        Ok(Token {
            kind: TokenType::Punctuator(Punctuator::VerticalBar),
            pos: SourcePosition::default()
        })
    );

    assert!(result_err.is_err());
}

#[test]
fn test_double_quote_string() {
    let mut parser = double_quote_string();
    let result = parser.parse(State::new("\"...\"")).map(|x| x.0);
    let result_1 = parser.parse(State::new("\"12345\"")).map(|x| x.0);
    let result_2 = parser.parse(State::new("\"Valid string\"")).map(|x| x.0);
    let result_escape_sequence = parser
        .parse(State::new("\"Test escapes \\t \\f\""))
        .map(|x| x.0);
    let result_err = double_quote_string()
        .easy_parse(State::new("\"missing ending quote"))
        .map(|x| x.0);

    let result_err_2 = double_quote_string()
        .easy_parse(State::new(
            "\"newline
            error",
        ))
        .map(|x| x.0);

    assert_eq!(result, Ok("..."));
    assert_eq!(result_1, Ok("12345"));
    assert_eq!(result_2, Ok("Valid string"));
    assert_eq!(result_escape_sequence, Ok("Test escapes \\t \\f"));

    assert_eq!(
        result_err,
        Err(Errors {
            position: SourcePosition {
                line: 1,
                column: 22
            },
            errors: vec![
                Error::Unexpected("end of input".into()),
                Error::Expected('\"'.into()),
                Error::Message("unterminated string".into())
            ]
        })
    );

    assert_eq!(
        result_err_2,
        Err(Errors {
            position: SourcePosition { line: 1, column: 9 },
            errors: vec![
                Error::Unexpected('\n'.into()),
                Error::Expected('\"'.into()),
                Error::Message("unterminated string".into())
            ]
        })
    );
}

#[test]
fn test_parse() {
    let schema = include_str!("../../../tests/stackup-example.graphql");

    assert_eq!(
        parse(schema),
        Ok(vec![
            Token {
                kind: TokenType::Name("type"),
                pos: SourcePosition { line: 1, column: 1 }
            },
            Token {
                kind: TokenType::Name("User"),
                pos: SourcePosition { line: 1, column: 6 }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::AtSign),
                pos: SourcePosition {
                    line: 1,
                    column: 11
                }
            },
            Token {
                kind: TokenType::Name("authenticate"),
                pos: SourcePosition {
                    line: 1,
                    column: 12
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::LeftCurlyBracket),
                pos: SourcePosition {
                    line: 1,
                    column: 25
                }
            },
            Token {
                kind: TokenType::Name("id"),
                pos: SourcePosition { line: 2, column: 3 }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Colon),
                pos: SourcePosition { line: 2, column: 5 }
            },
            Token {
                kind: TokenType::Name("ID"),
                pos: SourcePosition { line: 2, column: 7 }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Bang),
                pos: SourcePosition { line: 2, column: 9 }
            },
            Token {
                kind: TokenType::Name("email"),
                pos: SourcePosition { line: 3, column: 3 }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Colon),
                pos: SourcePosition { line: 3, column: 8 }
            },
            Token {
                kind: TokenType::Name("String"),
                pos: SourcePosition {
                    line: 3,
                    column: 10
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Bang),
                pos: SourcePosition {
                    line: 3,
                    column: 16
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::AtSign),
                pos: SourcePosition {
                    line: 3,
                    column: 18
                }
            },
            Token {
                kind: TokenType::Name("unique"),
                pos: SourcePosition {
                    line: 3,
                    column: 19
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::RightCurlyBracket),
                pos: SourcePosition { line: 4, column: 1 }
            },
            Token {
                kind: TokenType::Name("enum"),
                pos: SourcePosition { line: 6, column: 1 }
            },
            Token {
                kind: TokenType::Name("Genre"),
                pos: SourcePosition { line: 6, column: 6 }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::LeftCurlyBracket),
                pos: SourcePosition {
                    line: 6,
                    column: 12
                }
            },
            Token {
                kind: TokenType::Name("FICTION"),
                pos: SourcePosition { line: 7, column: 3 }
            },
            Token {
                kind: TokenType::Name("NONFICTION"),
                pos: SourcePosition { line: 8, column: 3 }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::RightCurlyBracket),
                pos: SourcePosition { line: 9, column: 1 }
            },
            Token {
                kind: TokenType::Name("type"),
                pos: SourcePosition {
                    line: 11,
                    column: 1
                }
            },
            Token {
                kind: TokenType::Name("Book"),
                pos: SourcePosition {
                    line: 11,
                    column: 6
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::LeftCurlyBracket),
                pos: SourcePosition {
                    line: 11,
                    column: 11
                }
            },
            Token {
                kind: TokenType::Name("id"),
                pos: SourcePosition {
                    line: 12,
                    column: 3
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Colon),
                pos: SourcePosition {
                    line: 12,
                    column: 5
                }
            },
            Token {
                kind: TokenType::Name("ID"),
                pos: SourcePosition {
                    line: 12,
                    column: 7
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Bang),
                pos: SourcePosition {
                    line: 12,
                    column: 9
                }
            },
            Token {
                kind: TokenType::Name("title"),
                pos: SourcePosition {
                    line: 13,
                    column: 3
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Colon),
                pos: SourcePosition {
                    line: 13,
                    column: 8
                }
            },
            Token {
                kind: TokenType::Name("String"),
                pos: SourcePosition {
                    line: 13,
                    column: 10
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Bang),
                pos: SourcePosition {
                    line: 13,
                    column: 16
                }
            },
            Token {
                kind: TokenType::Name("genre"),
                pos: SourcePosition {
                    line: 14,
                    column: 3
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Colon),
                pos: SourcePosition {
                    line: 14,
                    column: 8
                }
            },
            Token {
                kind: TokenType::Name("Genre"),
                pos: SourcePosition {
                    line: 14,
                    column: 10
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Bang),
                pos: SourcePosition {
                    line: 14,
                    column: 15
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::AtSign),
                pos: SourcePosition {
                    line: 14,
                    column: 17
                }
            },
            Token {
                kind: TokenType::Name("column"),
                pos: SourcePosition {
                    line: 14,
                    column: 18
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::LeftParen),
                pos: SourcePosition {
                    line: 14,
                    column: 24
                }
            },
            Token {
                kind: TokenType::Name("default"),
                pos: SourcePosition {
                    line: 14,
                    column: 25
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Colon),
                pos: SourcePosition {
                    line: 14,
                    column: 32
                }
            },
            Token {
                kind: TokenType::StringValue("NONFICTION"),
                pos: SourcePosition {
                    line: 14,
                    column: 34
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::RightParen),
                pos: SourcePosition {
                    line: 14,
                    column: 46
                }
            },
            Token {
                kind: TokenType::Name("author"),
                pos: SourcePosition {
                    line: 15,
                    column: 3
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Colon),
                pos: SourcePosition {
                    line: 15,
                    column: 9
                }
            },
            Token {
                kind: TokenType::Name("Author"),
                pos: SourcePosition {
                    line: 15,
                    column: 11
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Bang),
                pos: SourcePosition {
                    line: 15,
                    column: 17
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::AtSign),
                pos: SourcePosition {
                    line: 15,
                    column: 19
                }
            },
            Token {
                kind: TokenType::Name("belongsTo"),
                pos: SourcePosition {
                    line: 15,
                    column: 20
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::RightCurlyBracket),
                pos: SourcePosition {
                    line: 16,
                    column: 1
                }
            },
            Token {
                kind: TokenType::Name("type"),
                pos: SourcePosition {
                    line: 18,
                    column: 1
                }
            },
            Token {
                kind: TokenType::Name("Author"),
                pos: SourcePosition {
                    line: 18,
                    column: 6
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::LeftCurlyBracket),
                pos: SourcePosition {
                    line: 18,
                    column: 13
                }
            },
            Token {
                kind: TokenType::Name("id"),
                pos: SourcePosition {
                    line: 19,
                    column: 3
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Colon),
                pos: SourcePosition {
                    line: 19,
                    column: 5
                }
            },
            Token {
                kind: TokenType::Name("ID"),
                pos: SourcePosition {
                    line: 19,
                    column: 7
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Bang),
                pos: SourcePosition {
                    line: 19,
                    column: 9
                }
            },
            Token {
                kind: TokenType::Name("name"),
                pos: SourcePosition {
                    line: 20,
                    column: 3
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Colon),
                pos: SourcePosition {
                    line: 20,
                    column: 7
                }
            },
            Token {
                kind: TokenType::Name("String"),
                pos: SourcePosition {
                    line: 20,
                    column: 9
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Bang),
                pos: SourcePosition {
                    line: 20,
                    column: 15
                }
            },
            Token {
                kind: TokenType::Name("books"),
                pos: SourcePosition {
                    line: 21,
                    column: 3
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Colon),
                pos: SourcePosition {
                    line: 21,
                    column: 8
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::LeftSquareBracket),
                pos: SourcePosition {
                    line: 21,
                    column: 10
                }
            },
            Token {
                kind: TokenType::Name("Book"),
                pos: SourcePosition {
                    line: 21,
                    column: 11
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Bang),
                pos: SourcePosition {
                    line: 21,
                    column: 15
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::RightSquareBracket),
                pos: SourcePosition {
                    line: 21,
                    column: 16
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Bang),
                pos: SourcePosition {
                    line: 21,
                    column: 17
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::RightCurlyBracket),
                pos: SourcePosition {
                    line: 22,
                    column: 1
                }
            },
            Token {
                kind: TokenType::Name("type"),
                pos: SourcePosition {
                    line: 24,
                    column: 1
                }
            },
            Token {
                kind: TokenType::Name("Widget"),
                pos: SourcePosition {
                    line: 24,
                    column: 6
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::LeftCurlyBracket),
                pos: SourcePosition {
                    line: 24,
                    column: 13
                }
            },
            Token {
                kind: TokenType::Name("id"),
                pos: SourcePosition {
                    line: 25,
                    column: 3
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Colon),
                pos: SourcePosition {
                    line: 25,
                    column: 5
                }
            },
            Token {
                kind: TokenType::Name("ID"),
                pos: SourcePosition {
                    line: 25,
                    column: 7
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Bang),
                pos: SourcePosition {
                    line: 25,
                    column: 9
                }
            },
            Token {
                kind: TokenType::Name("string"),
                pos: SourcePosition {
                    line: 26,
                    column: 3
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Colon),
                pos: SourcePosition {
                    line: 26,
                    column: 9
                }
            },
            Token {
                kind: TokenType::Name("String"),
                pos: SourcePosition {
                    line: 26,
                    column: 11
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Bang),
                pos: SourcePosition {
                    line: 26,
                    column: 17
                }
            },
            Token {
                kind: TokenType::Name("date"),
                pos: SourcePosition {
                    line: 27,
                    column: 3
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Colon),
                pos: SourcePosition {
                    line: 27,
                    column: 7
                }
            },
            Token {
                kind: TokenType::Name("Date"),
                pos: SourcePosition {
                    line: 27,
                    column: 9
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Bang),
                pos: SourcePosition {
                    line: 27,
                    column: 13
                }
            },
            Token {
                kind: TokenType::Name("datetime"),
                pos: SourcePosition {
                    line: 28,
                    column: 3
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Colon),
                pos: SourcePosition {
                    line: 28,
                    column: 11
                }
            },
            Token {
                kind: TokenType::Name("DateTime"),
                pos: SourcePosition {
                    line: 28,
                    column: 13
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Bang),
                pos: SourcePosition {
                    line: 28,
                    column: 21
                }
            },
            Token {
                kind: TokenType::Name("integer"),
                pos: SourcePosition {
                    line: 29,
                    column: 3
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Colon),
                pos: SourcePosition {
                    line: 29,
                    column: 10
                }
            },
            Token {
                kind: TokenType::Name("Int"),
                pos: SourcePosition {
                    line: 29,
                    column: 12
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Bang),
                pos: SourcePosition {
                    line: 29,
                    column: 15
                }
            },
            Token {
                kind: TokenType::Name("float"),
                pos: SourcePosition {
                    line: 30,
                    column: 3
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Colon),
                pos: SourcePosition {
                    line: 30,
                    column: 8
                }
            },
            Token {
                kind: TokenType::Name("Float"),
                pos: SourcePosition {
                    line: 30,
                    column: 10
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Bang),
                pos: SourcePosition {
                    line: 30,
                    column: 15
                }
            },
            Token {
                kind: TokenType::Name("decimal"),
                pos: SourcePosition {
                    line: 31,
                    column: 3
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Colon),
                pos: SourcePosition {
                    line: 31,
                    column: 10
                }
            },
            Token {
                kind: TokenType::Name("Decimal"),
                pos: SourcePosition {
                    line: 31,
                    column: 12
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Bang),
                pos: SourcePosition {
                    line: 31,
                    column: 19
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::AtSign),
                pos: SourcePosition {
                    line: 31,
                    column: 21
                }
            },
            Token {
                kind: TokenType::Name("column"),
                pos: SourcePosition {
                    line: 31,
                    column: 22
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::LeftParen),
                pos: SourcePosition {
                    line: 31,
                    column: 28
                }
            },
            Token {
                kind: TokenType::Name("precision"),
                pos: SourcePosition {
                    line: 31,
                    column: 29
                }
            },
            Token {
                kind: TokenType::Punctuator(Punctuator::Colon),
                pos: SourcePosition {
                    line: 31,
                    column: 38
                }
            },
            Token {
                kind: TokenType::IntValue(10),
                pos: SourcePosition {
                    line: 31,
                    column: 40
                }
            }
        ])
    )
}
