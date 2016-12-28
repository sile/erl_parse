use num::BigUint;

use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Symbol(Symbol),
    Keyword(Keyword),
    Char(char),
    Var(String),
    Atom(String),
    Integer(BigUint),
    Float(f64),
    String(String),
    Comment(String),
    LineNum(usize),
}
macro_rules! impl_from_for_token {
    ($from:ident, $conv:ident) => {
        impl From<$from> for Token {
            fn from(f: $from) -> Self {
                Token::$conv(f)
            }
        }
    }
}
impl_from_for_token!(Symbol, Symbol);
impl_from_for_token!(Keyword, Keyword);
impl_from_for_token!(char, Char);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    OpenSquare, // [
    CloseSquare, // ]
    OpenParen, // (
    CloseParen, // )
    OpenBrace, // {
    CloseBrace, // }
    Sharp, // #
    Slash, // /
    Dot, // .
    Comma, // ,
    Colon, // :
    Semicolon, // ;
    Match, // =
    MapMatch, // :=
    VerticalBar, // |
    DoubleVerticalBar, // ||
    Question, // ?
    Not, // !
    Hyphen, // -
    MinusMinus, // --
    Plus, // +
    PlusPlus, // ++
    Multiply, // *
    RightAllow, // ->
    LeftAllow, // <-
    DoubleRightAllow, // =>
    DoubleLeftAllow, // <=
    DoubleRightAngle, // >>
    DoubleLeftAngle, // <<
    Eq, // ==
    ExactEq, // =:=
    NotEq, // /=
    ExactNotEq, // =/=
    Greater, // >
    GreaterEq, // >=
    Less, // <
    LessEq, // =<
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotKeyword;

// http://erlang.org/doc/reference_manual/introduction.html
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    After,
    And,
    Andalso,
    Band,
    Begin,
    Bnot,
    Bor,
    Bsl,
    Bsr,
    Bxor,
    Case,
    Catch,
    Cond,
    Div,
    End,
    Fun,
    If,
    Let,
    Not,
    Of,
    Or,
    Orelse,
    Receive,
    Rem,
    Try,
    When,
    Xor,
}
impl FromStr for Keyword {
    type Err = NotKeyword;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "after" => Ok(Keyword::After),
            "and" => Ok(Keyword::And),
            "andalso" => Ok(Keyword::Andalso),
            "band" => Ok(Keyword::Band),
            "begin" => Ok(Keyword::Begin),
            "bnot" => Ok(Keyword::Bnot),
            "bor" => Ok(Keyword::Bor),
            "bsl" => Ok(Keyword::Bsl),
            "bsr" => Ok(Keyword::Bsr),
            "bxor" => Ok(Keyword::Bxor),
            "case" => Ok(Keyword::Case),
            "catch" => Ok(Keyword::Catch),
            "cond" => Ok(Keyword::Cond),
            "div" => Ok(Keyword::Div),
            "end" => Ok(Keyword::End),
            "fun" => Ok(Keyword::Fun),
            "if" => Ok(Keyword::If),
            "let" => Ok(Keyword::Let),
            "not" => Ok(Keyword::Not),
            "of" => Ok(Keyword::Of),
            "or" => Ok(Keyword::Or),
            "orelse" => Ok(Keyword::Orelse),
            "receive" => Ok(Keyword::Receive),
            "rem" => Ok(Keyword::Rem),
            "try" => Ok(Keyword::Try),
            "when" => Ok(Keyword::When),
            "xor" => Ok(Keyword::Xor),
            _ => Err(NotKeyword),
        }
    }
}
