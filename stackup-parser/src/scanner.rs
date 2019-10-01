enum TokenType<'a> {
    Punctuator(Punctuator),
    Name(&'a str),
    IntValue(u32),
    FloatValue(i32),
    StringValue(&'a str),
}

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
