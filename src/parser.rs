use chumsky::{combinator::DelimitedBy, prelude::*, primitive::Just};
use std::{fmt::Display, hash::Hash, ops::Range};

use crate::{
    lexer::{Delim, Token},
    Command, EnvCommand, FsCommand, IoCommand, Pragma,
};

#[derive(Debug, Clone)]
pub struct Spanned<T>(T, Range<usize>);

impl<T> Spanned<T> {
    pub fn new(val: T) -> Self {
        Self(val, 0..0)
    }

    pub fn span(&self) -> Range<usize> {
        self.1.clone()
    }

    pub fn inner(&self) -> &T {
        &self.0
    }
}

impl<T: Clone> Spanned<T> {
    pub fn inner_owned(&self) -> T {
        self.0.clone()
    }
}

impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: PartialEq<T>> PartialEq<T> for Spanned<T> {
    fn eq(&self, other: &T) -> bool {
        self.0 == *other
    }
}

impl<T> Eq for Spanned<T> where T: PartialEq {}

impl<T: Display> Display for Spanned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: Hash> Hash for Spanned<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AST {
    Expose(Pragma, Spanned<String>),
    Unit(Spanned<String>, Vec<Command>),
    Err,
}

pub fn parser() -> impl Parser<Token, Vec<AST>, Error = Simple<Token>> {
    choice((expose(), unit()))
        .recover_with(skip_parser(
            choice((expose().ignored(), unit().ignored()))
                .not()
                .repeated()
                .ignore_then(any().rewind())
                .to(AST::Err),
        ))
        .repeated()
}

pub fn expose() -> impl Parser<Token, AST, Error = Simple<Token>> + Clone {
    just(Token::Expose)
        .ignore_then(ident())
        .then_ignore(just(Token::As))
        .then(
            with_ident("test")
                .to(Pragma::Test)
                .or(with_ident("build").to(Pragma::Build)),
        )
        .map(|(src, dst)| AST::Expose(dst, src))
        .labelled("expose declaration")
}

pub fn unit() -> impl Parser<Token, AST, Error = Simple<Token>> {
    just(Token::Unit)
        .ignore_then(ident())
        .then(command().repeated().curly_delimited())
        .map(|(name, cmds)| AST::Unit(name, cmds))
        .labelled("unit definition")
}

pub fn command() -> impl Parser<Token, Command, Error = Simple<Token>> {
    choice((
        module_prefix("FS")
            .ignore_then(fs_command())
            .map(Command::Fs),
        module_prefix("IO")
            .ignore_then(io_command())
            .map(Command::Io),
        module_prefix("ENV")
            .ignore_then(env_command())
            .map(Command::Env),
        with_ident("depends_on")
            .ignore_then(ident().separated_by(just(Token::Comma)).round_delimited())
            .map(Command::DependsOn),
        with_ident("do")
            .ignore_then(ident().separated_by(just(Token::Comma)).round_delimited())
            .map(Command::Do),
        with_ident("meta")
            .to(Token::Attr)
            .or(just(Token::Attr))
            .ignore_then(
                just(Token::Attr)
                    .ignore_then(ident())
                    .then(str().map(|s| s.0))
                    .repeated()
                    .round_delimited()
                    .collect()
                    .map(Command::Meta),
            ),
        with_ident("exec")
            .to(Token::Tilde)
            .or(just(Token::Tilde))
            .ignore_then(
                ident()
                    .or(str())
                    .or(select! {|span| Token::RawIdent(i) => Spanned(i, span)})
                    .repeated()
                    .round_delimited()
                    .map(Command::Exec),
            ),
    ))
    .labelled("command call")
}

fn fs_command() -> impl Parser<Token, FsCommand, Error = Simple<Token>> {
    choice((
        with_ident("create")
            .ignore_then(str().round_delimited())
            .map(FsCommand::Create),
        with_ident("create_dir")
            .ignore_then(str().round_delimited())
            .map(FsCommand::CreateDir),
        with_ident("remove")
            .ignore_then(str().round_delimited())
            .map(FsCommand::Remove),
        with_ident("copy")
            .ignore_then(
                str()
                    .then_ignore(just(Token::Comma))
                    .then(str())
                    .round_delimited(),
            )
            .map(|(src, dst)| FsCommand::Copy(src, dst)),
        with_ident("copy")
            .ignore_then(
                str()
                    .then_ignore(just(Token::Comma))
                    .then(str())
                    .round_delimited(),
            )
            .map(|(src, dst)| FsCommand::Move(src, dst)),
        with_ident("print_file")
            .ignore_then(str().round_delimited())
            .map(FsCommand::PrintFile),
        with_ident("eprint_file")
            .ignore_then(str().round_delimited())
            .map(FsCommand::EPrintFile),
    ))
}

fn io_command() -> impl Parser<Token, IoCommand, Error = Simple<Token>> {
    choice((
        with_ident("println")
            .ignore_then(str().round_delimited())
            .map(IoCommand::PrintLn),
        with_ident("print")
            .ignore_then(str().round_delimited())
            .map(IoCommand::Print),
        with_ident("eprintln")
            .ignore_then(str().round_delimited())
            .map(IoCommand::EPrintLn),
        with_ident("eprint")
            .ignore_then(str().round_delimited())
            .map(IoCommand::EPrint),
    ))
}

fn env_command() -> impl Parser<Token, EnvCommand, Error = Simple<Token>> {
    choice((
        with_ident("set_var")
            .ignore_then(
                str()
                    .then_ignore(just(Token::Comma))
                    .then(str())
                    .round_delimited(),
            )
            .map(|(var, val)| EnvCommand::SetVar(var, val)),
        with_ident("remove_var")
            .ignore_then(str().round_delimited())
            .map(EnvCommand::RemoveVar),
        with_ident("path_push")
            .ignore_then(str().round_delimited())
            .map(EnvCommand::PathPush),
        with_ident("path_remove")
            .ignore_then(str().round_delimited())
            .map(EnvCommand::PathRemove),
    ))
}

fn module_prefix(s: impl AsRef<str>) -> impl Parser<Token, (), Error = Simple<Token>> {
    select! { |span| Token::Ident(i) if i.eq_ignore_ascii_case(s.as_ref())  => Spanned(i, span) }
        .ignored()
        .then_ignore(just(Token::DoubleColon))
        .or_not()
        .to(())
}

fn with_ident(
    s: impl Into<String> + Clone,
) -> impl Parser<Token, Spanned<String>, Error = Simple<Token>> + Clone {
    select! { |span| Token::Ident(i) if i == s.clone().into() => Spanned(i, span) }
}

fn ident() -> impl Parser<Token, Spanned<String>, Error = Simple<Token>> + Clone {
    select! { |span| Token::Ident(i) => Spanned(i, span) }
}

fn str() -> impl Parser<Token, Spanned<String>, Error = Simple<Token>> + Clone {
    select! { |span| Token::String(i) => Spanned(i, span) }
}

/// Alias for the return type of [`ParserExt`]'s methods
pub type TokenDelim<T> = DelimitedBy<
    T,
    Just<Token, Token, Simple<Token>>,
    Just<Token, Token, Simple<Token>>,
    Token,
    Token,
>;

/// Utility trait for chumsky's Parser
///
/// Provides methods for wrapping [`Token`] groups in bracket delimiters
pub trait ParserExt<O>: Parser<Token, O> + Sized {
    fn round_delimited(self) -> TokenDelim<Self>;
    fn square_delimited(self) -> TokenDelim<Self>;
    fn curly_delimited(self) -> TokenDelim<Self>;
    fn arrow_delimited(self) -> TokenDelim<Self>;
}

impl<O, P> ParserExt<O> for P
where
    P: Parser<Token, O, Error = Simple<Token>>,
{
    fn round_delimited(self) -> TokenDelim<Self> {
        use Delim::*;
        use Token::*;

        self.delimited_by(just(OpenDelim(Round)), just(CloseDelim(Round)))
    }

    fn square_delimited(self) -> TokenDelim<Self> {
        use Delim::*;
        use Token::*;

        self.delimited_by(just(OpenDelim(Square)), just(CloseDelim(Square)))
    }

    fn curly_delimited(self) -> TokenDelim<Self> {
        use Delim::*;
        use Token::*;

        self.delimited_by(just(OpenDelim(Curly)), just(CloseDelim(Curly)))
    }

    fn arrow_delimited(self) -> TokenDelim<Self> {
        use Delim::*;
        use Token::*;

        self.delimited_by(just(OpenDelim(Arrow)), just(CloseDelim(Arrow)))
    }
}
