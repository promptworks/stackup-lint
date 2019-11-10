use combine::error::ParseError;
use combine::parser::char::{alpha_num, digit};
use combine::parser::item::satisfy;
use combine::parser::range::{self, recognize};
use combine::parser::repeat;
use combine::stream::easy::ParseError as EasyParseError;
use combine::stream::state::{SourcePosition, State};
use combine::stream::RangeStream;
use combine::{self, choice, optional, position, token, Parser};

/// Lexical grammar can be found here
/// https://github.com/graphql/graphql-spec/blob/master/spec/Appendix%20B%20--%20Grammar%20Summary.md

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType<'a> {
    Punctuator(Punctuator),
    Name(&'a str),
    IntValue(i32),
    FloatValue(f32),
    StringValue(&'a str),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Punctuator {
    Bang,
    DollarSign,
    Ampersand,
    LeftParen,
    RightParen,
    Ellipsis,
    Colon,
    Equals,
    AtSign,
    LeftSquareBracket,
    RightSquareBracket,
    LeftCurlyBracket,
    RightCurlyBracket,
    VerticalBar,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenType<'a>,
    pub pos: SourcePosition,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenType<'a>, pos: SourcePosition) -> Token<'a> {
        Self { kind, pos }
    }
}

pub(crate) fn parse(s: &str) -> Result<Vec<Token>, EasyParseError<State<&str, SourcePosition>>> {
    let input = State::new(s);
    tokens().easy_parse(input).map(|(tokens, _)| tokens)
}

fn tokens<'a, I>() -> impl Parser<Input = I, Output = Vec<Token<'a>>>
where
    I: RangeStream<Item = char, Range = &'a str, Position = SourcePosition>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    repeat::many(choice((
        punctuator(),
        name(),
        int_value(),
        float_value(),
        string_value(),
    )))
}

fn punctuator<'a, I>() -> impl Parser<Input = I, Output = Token<'a>>
where
    I: RangeStream<Item = char, Range = &'a str, Position = SourcePosition>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    choice((
        position()
            .skip(combine::char::string("..."))
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::Ellipsis), pos)),
        position()
            .skip(token('!'))
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::Bang), pos)),
        position()
            .skip(token('$'))
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::DollarSign), pos)),
        position()
            .skip(token('&'))
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::Ampersand), pos)),
        position()
            .skip(token('('))
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::LeftParen), pos)),
        position()
            .skip(token(')'))
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::RightParen), pos)),
        position()
            .skip(token(':'))
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::Colon), pos)),
        position()
            .skip(token('='))
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::Equals), pos)),
        position()
            .skip(token('@'))
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::AtSign), pos)),
        position()
            .skip(token('['))
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::LeftSquareBracket), pos)),
        position()
            .skip(token(']'))
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::RightSquareBracket), pos)),
        position()
            .skip(token('{'))
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::LeftCurlyBracket), pos)),
        position()
            .skip(token('}'))
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::RightCurlyBracket), pos)),
        position()
            .skip(token('|'))
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::VerticalBar), pos)),
    ))
}

fn name<'a, I>() -> impl Parser<Input = I, Output = Token<'a>>
where
    I: RangeStream<Item = char, Range = &'a str, Position = SourcePosition>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let alphanumeric_or_underscore =
        satisfy(|c: char| c.is_alphanumeric() || c == '_').expected("letter, number or underscore");
    let alphabetic_or_underscore =
        satisfy(|c: char| c.is_alphabetic() || c == '_').expected("letter or underscore");
    (
        position(),
        recognize((
            alphabetic_or_underscore,
            repeat::skip_many(alphanumeric_or_underscore),
        )),
    )
        .map(|(pos, name)| Token::new(TokenType::Name(name), pos))
}

fn int_value<'a, I>() -> impl Parser<Input = I, Output = Token<'a>>
where
    I: RangeStream<Item = char, Range = &'a str, Position = SourcePosition>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (position(), int_part())
        .map(|(pos, num)| Token::new(TokenType::IntValue(num.parse().unwrap()), pos))
}

// TODO: Block strings
fn string_value<'a, I>() -> impl Parser<Input = I, Output = Token<'a>>
where
    I: RangeStream<Item = char, Range = &'a str, Position = SourcePosition>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (position(), double_quote_string())
        .map(|(pos, str_literal)| Token::new(TokenType::StringValue(str_literal), pos))
}

// TODO: Add exponent parts
fn float_value<'a, I>() -> impl Parser<Input = I, Output = Token<'a>>
where
    I: RangeStream<Item = char, Range = &'a str, Position = SourcePosition>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (position(), recognize(int_part().and(fractional_part()))).map(|(pos, num): (_, &'a str)| {
        Token::new(TokenType::FloatValue(num.parse().unwrap()), pos)
    })
}

fn fractional_part<'a, I>() -> impl Parser<Input = I, Output = &'a str>
where
    I: RangeStream<Item = char, Range = &'a str, Position = SourcePosition>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    recognize((token('.'), repeat::skip_many1(digit())))
}

fn int_part<'a, I>() -> impl Parser<Input = I, Output = &'a str>
where
    I: RangeStream<Item = char, Range = &'a str, Position = SourcePosition>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let digits = repeat::many::<String, _>(digit());
    let non_zero_digit = satisfy(|c: char| c.is_digit(10) && c != '0').expected("non-zero digit");

    recognize((
        optional(token('-').or(token('+'))),
        range::range("0").or(recognize((non_zero_digit, digits))),
    ))
}

fn double_quote_string<'a, I>() -> impl Parser<Input = I, Output = &'a str>
where
    I: RangeStream<Item = char, Range = &'a str, Position = SourcePosition>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        token('\"'),
        recognize(repeat::skip_many(string_character())),
        token('\"').message("unterminated string"),
    )
        .map(|(_, value, _)| value)
}

fn string_character<'a, I>() -> impl Parser<Input = I, Output = &'a str>
where
    I: RangeStream<Item = char, Range = &'a str, Position = SourcePosition>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let escaped_character = satisfy(|c: char| {
        c == '\"'
            || c == '\\'
            || c == '/'
            || c == 'b'
            || c == 'f'
            || c == 'n'
            || c == 'r'
            || c == 't'
    })
    .map(|_| ());

    let unicode_sequence = (
        token('u'),
        alpha_num(),
        alpha_num(),
        alpha_num(),
        alpha_num(),
    )
        .map(|_| ());

    let source_character_string =
        satisfy(|c: char| is_source_character(c) && c != '\"' && c != '\n');

    choice((
        recognize(token('\\').and(unicode_sequence.or(escaped_character))),
        recognize(source_character_string),
    ))
}

fn is_source_character(c: char) -> bool {
    c == '\u{0009}' || c == '\u{000A}' || c == '\u{000D}' || (c >= '\u{0020}' && c <= '\u{FFFF}')
}

#[cfg(test)]
mod test {
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
}
