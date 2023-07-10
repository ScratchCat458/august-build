use ariadne::{Label, Report, ReportKind};
use std::{
    fmt::{self, Debug},
    ops::Range,
};

use super::token::{EncapsulatorType, Span, Token};

#[derive(Debug)]
pub enum ParserError {
    OutOfTokens {
        scope: ParserScope,
    },
    TokenMismatch {
        scope: ParserScope,
        token: Token,
        expected_token: Token,
    },
    EncapsulatorMismatch {
        scope: ParserScope,
        encap: EncapsulatorType,
        expected_encap: EncapsulatorType,
        node_span: Span,
    },
    BadChunkLength {
        scope: ParserScope,
        len: usize,
        valid_len: Vec<usize>,
        chunk_span: Span,
    },
    InvalidBody {
        scope: ParserScope,
        token: Token,
        valid_body: Vec<String>,
    },
    IoError(std::io::Error),
}

impl ParserError {
    pub fn report_gen(&self, file_name: impl Into<String>) -> Report<(String, Range<usize>)> {
        #![allow(unused_variables)]
        let file_name = file_name.into();
        let report = match self {
            Self::OutOfTokens { scope } => self.out_of_tokens_impl(file_name),
            Self::TokenMismatch {
                scope,
                token,
                expected_token,
            } => self.token_mismatch_impl(file_name),
            Self::EncapsulatorMismatch {
                scope,
                encap,
                expected_encap,
                node_span,
            } => self.encapsulator_mismatch_impl(file_name),
            Self::BadChunkLength {
                scope,
                len,
                valid_len,
                chunk_span,
            } => self.bad_chunk_length_impl(file_name),
            Self::InvalidBody {
                scope,
                token,
                valid_body,
            } => self.invalid_body_impl(file_name),
            Self::IoError(e) => Report::build(ReportKind::Error, "_".to_string(), 0)
                .with_message(e)
                .finish(),
        };

        report
    }

    fn out_of_tokens_impl(&self, file_name: String) -> Report<(String, Range<usize>)> {
        let Self::OutOfTokens { scope } = self else { panic!("Developer did a no no!") };

        Report::build(ReportKind::Error, file_name, 0)
            .with_code(9999)
            .with_message(format!("Ran of out tokens while parsing {scope}."))
            .with_help("This should never occur. Check for incomplete blocks or statements.")
            .finish()
    }

    fn token_mismatch_impl(&self, file_name: String) -> Report<(String, Range<usize>)> {
        let Self::TokenMismatch {
                scope,
                token,
                expected_token,
            } = self else { panic!("Developer did a no no!") };

        Report::build(ReportKind::Error, file_name.clone(), token.span_start())
            .with_code(1)
            .with_message(format!(
                "Token Mismatch: expected {}, found {} while parsing {scope}",
                expected_token.token_name(),
                token.token_name()
            ))
            .with_label(
                Label::new((file_name, token.span()))
                    .with_message(format!(
                        "Consider replacing with {}",
                        expected_token.token_name()
                    ))
                    .with_color(ariadne::Color::Red),
            )
            .finish()
    }

    fn encapsulator_mismatch_impl(&self, file_name: String) -> Report<(String, Range<usize>)> {
        let Self::EncapsulatorMismatch {
                scope,
                encap,
                expected_encap,
                node_span,
            } = self else { panic!("Developer did a no no!") };

        Report::build(ReportKind::Error, file_name.clone(), node_span.0)
            .with_code(2)
            .with_message(format!(
                "Encapsulator Mismatch: expected {expected_encap}, found {encap} while parsing {scope}"
            ))
            .with_label(
                Label::new((file_name, node_span.0..node_span.1))
                    .with_message(format!("Consider replacing with {expected_encap}")).with_color(ariadne::Color::Cyan)
            )
            .finish()
    }

    fn bad_chunk_length_impl(&self, file_name: String) -> Report<(String, Range<usize>)> {
        let Self::BadChunkLength {
                scope,
                len,
                valid_len,
                chunk_span,
            } = self else { panic!("Developer did a no no!") };

        Report::build(ReportKind::Error, file_name.clone(), chunk_span.0)
            .with_code(4)
            .with_message(format!("Bad Chunk Length: found chunk with length {len} while parsing {scope}, expected either of the following {valid_len:?}"))
            .with_label(
                Label::new((file_name, chunk_span.0..chunk_span.1))
                    .with_message(format!("Adjust chunk length to either {valid_len:?}"))
                    .with_color(ariadne::Color::Cyan)
            )
            .finish()
    }

    fn invalid_body_impl(&self, file_name: String) -> Report<(String, Range<usize>)> {
        let Self::InvalidBody {
                scope,
                token,
                valid_body,
            } = self else { panic!("Developer did a no no!") };

        Report::build(ReportKind::Error, file_name.clone(), token.span_start())
            .with_code(8)
            .with_message(format!("Invalid Token Body: token contained invalid body content with parsing {scope}, consider replacing with {valid_body:?}"))
            .with_label(
                Label::new((file_name, token.span()))
                .with_message(format!("Consider replacing with one of the following {valid_body:?}"))
                .with_color(ariadne::Color::Yellow)
            )
            .finish()
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let m = match self {
            Self::OutOfTokens { scope } => format!("Ran of out tokens while parsing {scope}.\nThis should never occur. Check for incomplete blocks or statements."),
            Self::TokenMismatch { scope, token, expected_token } => format!(
                "Token Mismatch: expected {}, found {} while parsing {scope}",
                expected_token.token_name(),
                token.token_name()
            ),
            Self::EncapsulatorMismatch { scope, encap, expected_encap, node_span: _ } => format!(
                "Encapsulator Mismatch: expected {expected_encap}, found {encap} while parsing {scope}"
            ),
            Self::BadChunkLength { scope, len, valid_len, chunk_span: _ } => format!("Bad Chunk Length: found chunk with length {len} while parsing {scope}, expected either of the following {valid_len:?}"),
            Self::InvalidBody { scope, token: _, valid_body } => format!("Invalid Token Body: token contained invalid body content with parsing {scope}, consider replacing with {valid_body:?}"),
            Self::IoError(e) => format!("{e}")
        };
        writeln!(f, "{m}")
    }
}

impl From<std::io::Error> for ParserError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl std::error::Error for ParserError {}

#[derive(Debug, Clone)]
pub enum ParserScope {
    Global,
    Namespace,
    Pragma,
    Task,
    CommandDefinition,
    TaskBody,
    CommandDefinitionBody,
}

impl fmt::Display for ParserScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let m = match self {
            Self::Global => "the global scope",
            Self::Namespace => "a namespace declaration",
            Self::Pragma => "a pragma declaration",
            Self::Task => "a task",
            Self::CommandDefinition => "a command definition",
            Self::TaskBody => "the body of a task",
            Self::CommandDefinitionBody => "the body of a command definition",
        };

        writeln!(f, "{m}")
    }
}
