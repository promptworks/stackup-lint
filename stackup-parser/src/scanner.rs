use combine::error::ParseError;
use combine::parser::char::digit;
use combine::parser::item::satisfy;
use combine::parser::range::{self, recognize};
use combine::parser::repeat;
use combine::stream::{state::SourcePosition, RangeStream};
use combine::{self, choice, optional, position, token, Parser};

#[derive(Debug, Clone, PartialEq)]
enum TokenType<'a> {
    Punctuator(Punctuator),
    Name(&'a str),
    IntValue(i32),
    FloatValue(f32),
    StringValue(&'a str),
}

#[derive(Debug, Clone, PartialEq)]
enum Punctuator {
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
struct Token<'a> {
    kind: TokenType<'a>,
    pos: SourcePosition,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenType<'a>, pos: SourcePosition) -> Token<'a> {
        Self { kind, pos }
    }
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
    I: 'a,
{
    (position(), int_part())
        .map(|(pos, num)| Token::new(TokenType::IntValue(num.parse().unwrap()), pos))
}

// TODO: Add exponent parts
fn float_value<'a, I>() -> impl Parser<Input = I, Output = Token<'a>>
where
    I: RangeStream<Item = char, Range = &'a str, Position = SourcePosition>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
    I: 'a,
{
    (position(), int_part(), fractional_part()).map(|(pos, mut num, fraction)| {
        num += fraction;
        Token::new(TokenType::FloatValue(num.parse().unwrap()), pos)
    })
}

fn fractional_part<'a, I>() -> impl Parser<Input = I, Output = &'a str>
where
    I: RangeStream<Item = char, Range = &'a str, Position = SourcePosition>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
    I: 'a,
{
    recognize((token('.'), repeat::skip_many1(digit())))
}

fn int_part<'a, I>() -> impl Parser<Input = I, Output = String> + 'a
where
    I: RangeStream<Item = char, Range = &'a str, Position = SourcePosition>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
    I: 'a,
{
    let digits = repeat::many::<String, _>(digit());
    let non_zero_digit = satisfy(|c: char| c.is_digit(10) && c != '0').expected("non-zero digit");
    (
        optional(token('-').or(token('+'))).map(|sign| sign.unwrap_or('+')),
        range::range("0").or(recognize((non_zero_digit, digits))),
    )
        .map(|(sign, n): (char, &str)| {
            let mut num = n.to_owned();
            num.insert(0, sign);

            num
        })
}

#[cfg(test)]
mod test {
    use super::*;
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
}
