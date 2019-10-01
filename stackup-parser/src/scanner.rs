use combine::error::ParseError;
use combine::stream::{state::SourcePosition, RangeStream};
use combine::{self, choice, position, token, Parser};

#[derive(Debug, Clone, PartialEq)]
enum TokenType<'a> {
    Punctuator(Punctuator),
    Name(&'a str),
    IntValue(u32),
    FloatValue(i32),
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
        combine::char::string("...")
            .with(position())
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::Ellipsis), pos)),
        token('!')
            .with(position())
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::Bang), pos)),
        token('$')
            .with(position())
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::DollarSign), pos)),
        token('&')
            .with(position())
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::Ampersand), pos)),
        token('(')
            .with(position())
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::LeftParen), pos)),
        token(')')
            .with(position())
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::RightParen), pos)),
        token(':')
            .with(position())
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::Colon), pos)),
        token('=')
            .with(position())
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::Equals), pos)),
        token('@')
            .with(position())
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::AtSign), pos)),
        token('[')
            .with(position())
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::LeftSquareBracket), pos)),
        token(']')
            .with(position())
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::RightSquareBracket), pos)),
        token('{')
            .with(position())
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::LeftCurlyBracket), pos)),
        token('}')
            .with(position())
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::RightCurlyBracket), pos)),
        token('|')
            .with(position())
            .map(|pos| Token::new(TokenType::Punctuator(Punctuator::VerticalBar), pos)),
    ))
}
