use crate::colours::OwoColorizeStderrSupported;
use ariadne::{Color, Label, Report, ReportKind, Source};
use august_build::LowerError;
use chumsky::{error::SimpleReason, prelude::Simple};
use std::{
    fmt::Display,
    hash::Hash,
    io::{self, Write},
    ops::Range,
};

/// Formatting construct for Chumsky's [`Simple`] error type.
/// Implemented for all [`Display`] to support [`char`] and
/// [`Token`](crate::lexer::Token) errors.
pub struct ParserErrorFormatter<D>
where
    D: Display + Eq + Hash,
{
    pub errors: Vec<Simple<D>>,
    pub file_name: String,
    pub code: String,
}

impl<D> ParserErrorFormatter<D>
where
    D: Display + Eq + Hash,
{
    /// Creates a new [`ParserErrorFormatter`] from a [`Vec`] of [`Simple`] errors,
    /// the name of the source file and it's contents.
    pub fn new(
        errors: Vec<Simple<D>>,
        file_name: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        Self {
            errors,
            file_name: file_name.into(),
            code: code.into(),
        }
    }

    /// Generates Ariadne [`Report`]'s for each error
    /// and writes them to a [`Write`] implementor.
    ///
    /// This process should only produce valid UTF-8.
    pub fn write_reports(&self, w: &mut dyn Write) -> io::Result<()> {
        let reports = self.errors.iter().map(|err| {
            let report = Report::<(String, Range<usize>)>::build(
                ReportKind::Error,
                self.file_name.clone(),
                err.span().start,
            );

            let report = if let SimpleReason::Unexpected = err.reason() {
                report
                    .with_message(format!(
                        "Unexpected {} found when parsing {}, expected one of {}",
                        match err.found() {
                            Some(f) => f.to_string(),
                            None => "EOF".to_string(),
                        }
                        .red(),
                        match err.label() {
                            Some(l) => l.to_string(),
                            None => "<unknown_pattern>".to_string(),
                        },
                        err.expected()
                            .fold(String::new(), |mut output, ex| {
                                use std::fmt::Write;
                                write!(
                                    output,
                                    "{}, ",
                                    match ex {
                                        Some(x) => x.to_string(),
                                        None => "<invalid_char>".to_string(),
                                    }
                                    .cyan()
                                )
                                .ok();
                                output
                            })
                            .trim_end_matches(", ")
                    ))
                    .with_label(
                        Label::new((self.file_name.clone(), err.span()))
                            .with_message(format!(
                                "Unexpected {}",
                                match err.found() {
                                    Some(f) => f.to_string(),
                                    None => "EOF".to_string(),
                                }
                                .red(),
                            ))
                            .with_color(Color::Red),
                    )
            } else {
                report
            };

            let report = if let SimpleReason::Unclosed { span, delimiter } = err.reason() {
                report
                    .with_message(format!(
                        "Unclosed delimiter {} found when parsing {}, expected one of {}",
                        match err.found() {
                            Some(f) => f.to_string(),
                            None => "EOF".to_string(),
                        }
                        .red(),
                        match err.label() {
                            Some(l) => l.to_string(),
                            None => "<unknown_pattern>".to_string(),
                        },
                        err.expected()
                            .fold(String::new(), |mut output, ex| {
                                use std::fmt::Write;
                                write!(
                                    output,
                                    "{}, ",
                                    match ex {
                                        Some(x) => x.to_string(),
                                        None => "<invalid_char>".to_string(),
                                    }
                                    .cyan()
                                )
                                .ok();

                                output
                            })
                            .trim_end_matches(", "),
                    ))
                    .with_label(
                        Label::new((self.file_name.clone(), err.span()))
                            .with_message(format!(
                                "Delimiter {} is never closed",
                                match err.found() {
                                    Some(f) => f.to_string(),
                                    None => "EOF".to_string(),
                                }
                                .red()
                            ))
                            .with_color(Color::Red),
                    )
                    .with_label(
                        Label::new((self.file_name.clone(), span.clone()))
                            .with_message(format!("Must be closed before {}", delimiter))
                            .with_color(Color::Yellow),
                    )
            } else {
                report
            };
            report.finish()
        });

        for r in reports {
            r.write(
                (self.file_name.clone(), Source::from(self.code.clone())),
                &mut *w,
            )?;
            writeln!(&mut *w)?;
        }

        Ok(())
    }
}

impl<D> Display for ParserErrorFormatter<D>
where
    D: Display + Eq + Hash,
{
    /// ## Note
    /// This method has two instances of error ignorance.
    ///
    /// IO errors from `write_reports` are ignored with `ok()` as it is writing to a local `Vec<u8>` buffer.
    /// UTF-8 conversion has been `expect`ed as `write_reports` should only produce valid UTF-8.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = Vec::new();
        self.write_reports(&mut buf).ok();
        String::from_utf8_lossy(&buf).trim_end().fmt(f)
    }
}

/// Formatting construct for [`LowerError`]
pub struct LowerErrorFormatter {
    errors: Vec<LowerError>,
    file_name: String,
    code: String,
}

impl LowerErrorFormatter {
    /// Creates a new [`LowerErrorFormatter`] from a [`Vec`] of [`LowerError`]s,
    /// the name of the source file and it's contents.
    pub fn new(
        errors: Vec<LowerError>,
        file_name: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        Self {
            errors,
            file_name: file_name.into(),
            code: code.into(),
        }
    }

    /// Generates Ariadne [`Report`]'s for each error
    /// and writes them to a [`Write`] implementor.
    ///
    /// This process should only produce valid UTF-8.
    pub fn write_reports(&self, w: &mut dyn Write) -> io::Result<()> {
        let reports = self
            .errors
            .iter()
            .map(|err| -> Report<(String, Range<usize>)> {
                use LowerError::*;

                match err {
                    DuplicateExpose(pragma, unit) => {
                        Report::build(ReportKind::Error, self.file_name.clone(), unit.span().start)
                            .with_message(format!(
                                "Attempted to define another binding for pragma {:?}", pragma.cyan()
                            ))
                            .with_help("Consider assigning this to a different pragma or removing this expose statement.")
                            .with_label(Label::new((self.file_name.clone(), unit.span())).with_color(Color::Cyan))
                            .finish()
                    }
                    DuplicateUnit(fst, snd) => {
                        Report::build(ReportKind::Error, self.file_name.clone(), fst.span().start)
                            .with_message(format!("Attempted to define multiple units with the name {}", snd.cyan()))
                            .with_label(Label::new((self.file_name.clone(), fst.span())).with_color(Color::Green).with_message("Unit first defined here"))
                            .with_label(Label::new((self.file_name.clone(), snd.span())).with_color(Color::Red).with_message("Unit defined again here"))
                            .with_help("Remove or change the name of one of the unit definitions.")
                            .finish()
                    }
                    DuplicateDependency(fst, snd) => {
                        Report::build(ReportKind::Error, self.file_name.clone(), fst.span().start)
                            .with_message(format!("Dependency {} defined multiple times in the same unit", snd.cyan()))
                            .with_label(Label::new((self.file_name.clone(), fst.span())).with_color(Color::Green).with_message("First defined here"))
                            .with_label(Label::new((self.file_name.clone(), snd.span())).with_color(Color::Red).with_message("Defined again here"))
                            .with_help("Remove the duplicate dependency.")
                            .finish()
                    }
                    DuplicateMetaItem(fst, snd) => {
                        Report::build(ReportKind::Error, self.file_name.clone(), fst.span().start)
                            .with_message(format!("Meta item {} defined multiple times in the same unit", snd.cyan()))
                            .with_label(Label::new((self.file_name.clone(), fst.span())).with_color(Color::Green).with_message("First defined here"))
                            .with_label(Label::new((self.file_name.clone(), snd.span())).with_color(Color::Red).with_message("Defined again here"))
                            .with_help("Remove the duplicate meta item.")
                            .finish()
                    }
                    NameError(unit) => {
                        Report::build(ReportKind::Error, self.file_name.clone(), unit.span().start)
                            .with_message(format!("Identifier refers to a unit {} that doesn't exist", unit.red()))
                            .with_label(Label::new((self.file_name.clone(), unit.span())).with_color(Color::Red).with_message("Undefined unit"))
                            .with_help(format!("Define a unit with the name {} or change the unit being referred to.", unit.red()))
                            .finish()
                    },
                }
            });

        for r in reports {
            r.write(
                (self.file_name.clone(), Source::from(self.code.clone())),
                &mut *w,
            )?;
            writeln!(&mut *w)?;
        }

        Ok(())
    }
}

impl Display for LowerErrorFormatter {
    /// ## Note
    /// This method has two instances of error ignorance.
    ///
    /// IO errors from `write_reports` are ignored with `ok()` as it is writing to a local `Vec<u8>` buffer.
    /// UTF-8 conversion has been `expect`ed as `write_reports` should only produce valid UTF-8.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = Vec::new();
        self.write_reports(&mut buf).ok();
        String::from_utf8_lossy(&buf).trim_end().fmt(f)
    }
}
