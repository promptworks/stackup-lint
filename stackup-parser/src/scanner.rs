use combine::error::ParseError;
use combine::parser::char::{alpha_num, digit, spaces};
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
    combine::sep_by(
        choice((
            punctuator(),
            name(),
            int_value(),
            float_value(),
            string_value(),
        )),
        spaces(),
    )
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
mod test;
