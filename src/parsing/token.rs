use core::fmt;
use std::{
    iter::{Enumerate, Peekable},
    ops::Range,
    str::Chars,
};

#[derive(Debug, Clone)]
pub enum Token {
    Ident(String, Span),
    Punct(char, Span),
    StringLiteral(String, Span),
    IntegerLiteral(u64, Span),
    BooleanLiteral(bool, Span),
    Node(Node, Span),
}

impl Token {
    pub fn span(&self) -> Range<usize> {
        match self {
            Self::Ident(_, s)
            | Self::Punct(_, s)
            | Self::StringLiteral(_, s)
            | Self::IntegerLiteral(_, s)
            | Self::BooleanLiteral(_, s)
            | Self::Node(_, s) => s.0..s.1,
        }
    }

    pub fn span_start(&self) -> usize {
        match self {
            Self::Ident(_, s)
            | Self::Punct(_, s)
            | Self::StringLiteral(_, s)
            | Self::IntegerLiteral(_, s)
            | Self::BooleanLiteral(_, s)
            | Self::Node(_, s) => s.0,
        }
    }

    pub fn span_end(&self) -> usize {
        match self {
            Self::Ident(_, s)
            | Self::Punct(_, s)
            | Self::StringLiteral(_, s)
            | Self::IntegerLiteral(_, s)
            | Self::BooleanLiteral(_, s)
            | Self::Node(_, s) => s.1,
        }
    }

    pub fn token_name(&self) -> String {
        match self {
            Self::Ident(_, _) => "an ident".to_string(),
            Self::Punct(_, _) => "punct".to_string(),
            Self::StringLiteral(_, _) => "a string literal".to_string(),
            Self::IntegerLiteral(_, _) => "an integer".to_string(),
            Self::BooleanLiteral(_, _) => "a boolean".to_string(),
            Self::Node(_, _) => "a node".to_string(),
        }
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Ident(b, _) => {
                if let Self::Ident(c, _) = other {
                    b == c
                } else {
                    false
                }
            }
            Self::Punct(b, _) => {
                if let Self::Punct(c, _) = other {
                    b == c
                } else {
                    false
                }
            }
            Self::StringLiteral(b, _) => {
                if let Self::StringLiteral(c, _) = other {
                    b == c
                } else {
                    false
                }
            }
            Self::IntegerLiteral(b, _) => {
                if let Self::IntegerLiteral(c, _) = other {
                    b == c
                } else {
                    false
                }
            }
            Self::BooleanLiteral(b, _) => {
                if let Self::BooleanLiteral(c, _) = other {
                    b == c
                } else {
                    false
                }
            }
            Self::Node(b, _) => {
                if let Self::Node(c, _) = other {
                    b == c
                } else {
                    false
                }
            }
        }
    }
}

pub type Span = (usize, usize);

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub encapsulator: EncapsulatorType,
    pub children: Vec<Token>,
}

impl Node {
    pub fn new(encap: EncapsulatorType) -> Self {
        Node {
            encapsulator: encap,
            children: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncapsulatorType {
    Round,
    Curly,
    Square,
    Arrow,
}

impl EncapsulatorType {
    pub fn new(encap: char) -> Self {
        match encap {
            '(' | ')' => Self::Round,
            '{' | '}' => Self::Curly,
            '[' | ']' => Self::Square,
            '<' | '>' => Self::Arrow,
            _ => panic!("This should not be able to be triggered by the end-user"),
        }
    }

    pub fn opening_char(&self) -> char {
        match self {
            Self::Round => '(',
            Self::Curly => '{',
            Self::Square => '[',
            Self::Arrow => '<',
        }
    }

    pub fn closing_char(&self) -> char {
        match self {
            Self::Round => ')',
            Self::Curly => '}',
            Self::Square => ']',
            Self::Arrow => '>',
        }
    }
}

impl fmt::Display for EncapsulatorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let m = match self {
            Self::Round => "round brackets",
            Self::Curly => "curly brackets",
            Self::Square => "square brackets",
            Self::Arrow => "arrow brackets",
        };

        write!(f, "{m}")
    }
}

pub fn tokenise(input: String) -> Vec<Token> {
    let mut cursor = input.chars().enumerate().peekable();
    let mut tokens = Vec::new();

    while cursor.peek().is_some() {
        if let Some(t) = classify(&mut cursor) {
            tokens.push(t);
        }
    }

    tokens
}

fn create_node(cursor: &mut Peekable<Enumerate<Chars>>, encap: EncapsulatorType) -> Node {
    let mut node = Node::new(encap.clone());

    while let Some((_, c)) = cursor.peek() {
        if *c == encap.closing_char() {
            break;
        }

        if let Some(t) = classify(cursor) {
            node.children.push(t);
        }
    }

    node
}

fn classify(cursor: &mut Peekable<Enumerate<Chars>>) -> Option<Token> {
    #![allow(unused_assignments)]
    match *cursor.peek().unwrap() {
        (s, 'A'..='Z' | 'a'..='z' | '~') => {
            let mut ident = String::new();
            let mut ending_pos = 0_usize;

            loop {
                let (sp, ch) = cursor.peek().unwrap();
                match ch {
                    'A'..='Z' | 'a'..='z' | '_' | '-' | '~' => {
                        ident.push(ch.to_owned());
                        cursor.next();
                    }
                    _ => {
                        ending_pos = sp.to_owned();
                        break;
                    }
                }
            }

            match ident.as_str() {
                "True" | "true" => Some(Token::BooleanLiteral(true, (s, ending_pos))),
                "False" | "false" => Some(Token::BooleanLiteral(false, (s, ending_pos))),
                _ => Some(Token::Ident(ident, (s, ending_pos))),
            }
        }
        (s, '"') => {
            let mut str_lit = String::new();
            let mut ending_pos = 0_usize;
            cursor.next();

            loop {
                let (sp, ch) = cursor.peek().unwrap();
                if let '"' = ch {
                    ending_pos = sp.to_owned();
                    cursor.next();
                    break;
                }

                str_lit.push(*ch);
                cursor.next();
            }
            Some(Token::StringLiteral(
                str_lit,
                (s.to_owned(), ending_pos + 1),
            ))
        }
        (s, '0'..='9') => {
            let mut num_str = String::new();
            let mut ending_pos = 0_usize;

            loop {
                let (sp, ch) = cursor.peek().unwrap();

                if let '0'..='9' = ch {
                    num_str.push(ch.to_owned());
                    cursor.next();
                } else {
                    ending_pos = sp.to_owned();
                    break;
                }
            }

            Some(Token::IntegerLiteral(
                num_str.parse::<u64>().expect("Passed a non digit char"),
                (s, ending_pos),
            ))
        }
        (s, '(' | '{' | '[' | '<') => {
            let (_, t) = cursor.next().unwrap();
            let node = create_node(cursor, EncapsulatorType::new(t));
            let (ending_pos, _) = cursor.next().unwrap();

            Some(Token::Node(node, (s, ending_pos + 1)))
        }
        (s, ',' | ';' | ':' | '.' | '=' | '+' | '?' | '#' | '@') => {
            let (_, c) = cursor.next().unwrap();
            Some(Token::Punct(c, (s, s + 1)))
        }
        _ => {
            cursor.next();
            None
        }
    }
}
