use std::{fmt::Display, ops::Range};

use chumsky::{
    prelude::*,
    text::{ident, Character},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Token {
    String(String),
    Ident(String),
    RawIdent(String),

    Unit,
    Expose,
    As,

    Attr,
    Tilde,
    DoubleColon,
    DoubleArrow,
    Comma,

    OpenDelim(Delim),
    CloseDelim(Delim),

    Err(char),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Delim {
    Round,
    Square,
    Arrow,
    Curly,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Delim::*;
        use Token::*;

        match self {
            String(s) => f.write_fmt(format_args!("\"{s}\"")),
            Ident(i) => i.fmt(f),
            RawIdent(i) => i.fmt(f),
            Unit => "unit".fmt(f),
            Expose => "expose".fmt(f),
            As => "as".fmt(f),
            Attr => "@".fmt(f),
            Tilde => "~".fmt(f),
            Comma => ",".fmt(f),
            DoubleColon => "::".fmt(f),
            DoubleArrow => "=>".fmt(f),
            OpenDelim(Round) => "(".fmt(f),
            OpenDelim(Square) => "[".fmt(f),
            OpenDelim(Arrow) => "<".fmt(f),
            OpenDelim(Curly) => "{".fmt(f),
            CloseDelim(Round) => ")".fmt(f),
            CloseDelim(Square) => "]".fmt(f),
            CloseDelim(Arrow) => ">".fmt(f),
            CloseDelim(Curly) => "}".fmt(f),
            Err(c) => c.fmt(f),
        }
    }
}

pub fn lexer() -> impl Parser<char, Vec<(Token, Range<usize>)>, Error = Simple<char>> {
    let escape = just('\\').ignore_then(
        just('\\')
            .or(just('/'))
            .or(just('"'))
            .or(just('b').to('\x08'))
            .or(just('f').to('\x0C'))
            .or(just('n').to('\n'))
            .or(just('r').to('\r'))
            .or(just('t').to('\t')),
    );

    let str_lit = filter(|c| *c != '\\' && *c != '"')
        .or(escape)
        .repeated()
        .delimited_by(just('"'), just('"'))
        .collect();

    let keywords = choice((
        just("expose").to(Token::Expose),
        just("as").to(Token::As),
        just("unit").to(Token::Unit),
        just('@').to(Token::Attr),
        just('~').to(Token::Tilde),
        just("::").to(Token::DoubleColon),
        just("=>").to(Token::DoubleArrow),
        just(',').to(Token::Comma),
    ));

    let delim = choice((
        just('(').to(Token::OpenDelim(Delim::Round)),
        just('[').to(Token::OpenDelim(Delim::Square)),
        just('<').to(Token::OpenDelim(Delim::Arrow)),
        just('{').to(Token::OpenDelim(Delim::Curly)),
        just(')').to(Token::CloseDelim(Delim::Round)),
        just(']').to(Token::CloseDelim(Delim::Square)),
        just('>').to(Token::CloseDelim(Delim::Arrow)),
        just('}').to(Token::CloseDelim(Delim::Curly)),
    ));

    let raw_ident = filter(|c: &char| {
        matches!(c.to_char(),
        '!'..='&' | '*'..='+' | '-'..='.' | '0'..=';' | '=' | '?'..='Z' | '^'..='z' | '|')
    })
    .repeated()
    .at_least(1)
    .collect::<String>();

    let token = choice((
        keywords,
        delim,
        str_lit.map(Token::String),
        ident().map(Token::Ident),
        raw_ident.map(Token::RawIdent),
    ))
    .padded()
    .map_with_span(|t, span| (t, span));

    token
        .clone()
        .recover_with(skip_parser(
            token
                .not()
                .repeated()
                .ignore_then(any().rewind())
                .map_with_span(|c, span| (Token::Err(c), span)),
        ))
        .repeated()
        .padded()
        .then_ignore(end())
        .labelled("tokens")
}
