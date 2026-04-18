use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Number(String),
    StringLiteral(String),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Dot,
    Colon,
    Semicolon,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Arrow,
    As,
    ByVal,
    ByRef,
    Dim,
    New,
    Set,
    Mut,
    If,
    Else,
    ElseIf,
    Then,
    End,
    Function,
    Loop,
    While,
    Do,
    Until,
    Exit,
    For,
    ForEach,
    To,
    Step,
    In,
    Each,
    And,
    Or,
    Not,
    Is,
    Mod,
    Xor,
    Eqv,
    Imp,
    Like,
    Type,
    With,
    Property,
    Get,
    Let,
    SetAccessor,
    Return,
    True,
    False,
    Nothing,
    Null,
    Try,
    Catch,
    Finally,
    Throw,
    AsResult,
    Match,
    Case,
    CaseElse,
    Select,
    Continue,
    InStr,
    Len,
    Left,
    Right,
    Mid,
    UCase,
    LCase,
    Trim,
    Replace,
    Str,
    Val,
    Abs,
    Sqr,
    Int,
    Round,
    Sin,
    Cos,
    Tan,
    Log,
    Exp,
    Print,
    InputBox,
    Const,
    Public,
    Private,
    Static,
    ReDim,
    Preserve,
    Option,
    Base,
    Variant,
    Currency,
    LongLong,
    Long,
    Integer,
    Single,
    Double,
    Boolean,
    Byte,
    Date,
    StringType,
    HashMap,
    Vec,
    Error,
    Warning,
    Info,
    EOF,
}

#[derive(Debug, Clone)]
pub struct Lexer {
    input: String,
    position: usize,
    read_position: usize,
    ch: char,
    line: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let mut l = Lexer {
            input,
            position: 0,
            read_position: 0,
            ch: '\0',
            line: 1,
        };
        l.read_char();
        l
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\0';
        } else {
            let c = self.input[self.read_position..].chars().next().unwrap();
            self.ch = c;
        }
        self.position = self.read_position;
        self.read_position += self.ch.len_utf8();
    }

    fn peek_char(&self) -> char {
        if self.read_position >= self.input.len() {
            '\0'
        } else {
            self.input[self.read_position..].chars().next().unwrap()
        }
    }

    fn skip_whitespace(&mut self) {
        while self.ch.is_whitespace() && self.ch != '\n' && self.ch != '\0' {
            self.read_char();
        }
    }

    fn read_identifier(&mut self) -> String {
        let start = self.position;
        while self.ch.is_alphanumeric() || self.ch == '_' || self.ch == '.' {
            self.read_char();
        }
        self.input[start..self.position].to_string()
    }

    fn read_number(&mut self) -> String {
        let start = self.position;
        while self.ch.is_ascii_digit() {
            self.read_char();
        }
        self.input[start..self.position].to_string()
    }

    fn read_string(&mut self) -> String {
        self.read_char();
        let start = self.position;
        while self.ch != '"' && self.ch != '\0' {
            self.read_char();
        }
        let s = self.input[start..self.position].to_string();
        self.read_char();
        s
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let token = match self.ch {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            ',' => Token::Comma,
            '.' => Token::Dot,
            ':' => Token::Colon,
            ';' => Token::Semicolon,
            '=' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::NotEqual
                } else {
                    Token::Equal
                }
            }
            '<' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::LessEqual
                } else {
                    Token::Less
                }
            }
            '>' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::GreaterEqual
                } else {
                    Token::Greater
                }
            }
            '\"' => Token::StringLiteral(self.read_string()),
            '\0' => Token::EOF,
            _ => {
                if self.ch.is_alphabetic() || self.ch == '_' {
                    let ident = self.read_identifier();
                    return match ident.as_str() {
                        "Function" => Token::Function,
                        "End" => Token::End,
                        "If" => Token::If,
                        "Else" => Token::Else,
                        "ElseIf" => Token::ElseIf,
                        "Then" => Token::Then,
                        "Loop" => Token::Loop,
                        "While" => Token::While,
                        "Do" => Token::Do,
                        "Until" => Token::Until,
                        "Exit" => Token::Exit,
                    "ForEach" => Token::ForEach,
                        "For" => Token::For,
                        "To" => Token::To,
                        "Step" => Token::Step,
                        "In" => Token::In,
                        "Each" => Token::Each,
                        "And" => Token::And,
                        "Or" => Token::Or,
                        "Not" => Token::Not,
                        "Is" => Token::Is,
                        "Mod" => Token::Mod,
                        "Xor" => Token::Xor,
                        "Eqv" => Token::Eqv,
                        "Imp" => Token::Imp,
                        "Like" => Token::Like,
                        "Type" => Token::Type,
                        "With" => Token::With,
                        "Property" => Token::Property,
                        "Get" => Token::Get,
                        "Let" => Token::Let,
                        "Return" => Token::Return,
                        "True" => Token::True,
                        "False" => Token::False,
                        "Nothing" => Token::Nothing,
                        "Null" => Token::Null,
                        "Try" => Token::Try,
                        "Catch" => Token::Catch,
                        "Finally" => Token::Finally,
                        "Throw" => Token::Throw,
                        "AsResult" => Token::AsResult,
                        "Match" => Token::Match,
                        "Case" => Token::Case,
                        "CaseElse" => Token::CaseElse,
                        "Select" => Token::Select,
                        "Continue" => Token::Continue,
                        "InStr" => Token::InStr,
                        "Len" => Token::Len,
                        "Left" => Token::Left,
                        "Right" => Token::Right,
                        "Mid" => Token::Mid,
                        "UCase" => Token::UCase,
                        "LCase" => Token::LCase,
                        "Trim" => Token::Trim,
                        "Replace" => Token::Replace,
                        "Str" => Token::Str,
                        "Val" => Token::Val,
                        "Abs" => Token::Abs,
                        "Sqr" => Token::Sqr,
                        "Int" => Token::Int,
                        "Round" => Token::Round,
                        "Sin" => Token::Sin,
                        "Cos" => Token::Cos,
                        "Tan" => Token::Tan,
                        "Log" => Token::Log,
                        "Exp" => Token::Exp,
                        "Print" => Token::Print,
                        "InputBox" => Token::InputBox,
                        "Const" => Token::Const,
                        "Public" => Token::Public,
                        "Private" => Token::Private,
                        "Static" => Token::Static,
                        "ReDim" => Token::ReDim,
                        "Preserve" => Token::Preserve,
                        "Option" => Token::Option,
                        "Base" => Token::Base,
                        "Variant" => Token::Variant,
                        "Currency" => Token::Currency,
                        "LongLong" => Token::LongLong,
                        "Long" => Token::Long,
                        "Integer" => Token::Integer,
                        "Single" => Token::Single,
                        "Double" => Token::Double,
                        "Boolean" => Token::Boolean,
                        "Byte" => Token::Byte,
                        "Date" => Token::Date,
                        "String" => Token::StringType,
                        "HashMap" => Token::HashMap,
                        "Vec" => Token::Vec,
                        "Error" => Token::Error,
                        "Warning" => Token::Warning,
                        "Info" => Token::Info,
                        _ => Token::Ident(ident),
                    };
                } else {
                    Token::Error
                }
            }
        };
        self.next_token();
        token
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Ident(s) => write!(f, "Ident({})", s),
            Token::Number(s) => write!(f, "Number({})", s),
            Token::StringLiteral(s) => write!(f, "StringLiteral({})", s),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::Comma => write!(f, ","),
            Token::Dot => write!(f, "."),
            Token::Colon => write!(f, ":"),
            Token::Semicolon => write!(f, ";"),
            Token::Equal => write!(f, "="),
            Token::NotEqual => write!(f, "<>"),
            Token::Less => write!(f, "<"),
            Token::LessEqual => write!(f, "<="),
            Token::Greater => write!(f, ">"),
            Token::GreaterEqual => write!(f, ">="),
            Token::Arrow => write!(f, "->"),
            Token::As => write!(f, "As"),
            Token::ByVal => write!(f, "ByVal"),
            Token::ByRef => write!(f, "ByRef"),
            Token::Dim => write!(f, "Dim"),
            Token::New => write!(f, "New"),
            Token::Set => write!(f, "Set"),
            Token::Mut => write!(f, "Mut"),
            Token::If => write!(f, "If"),
            Token::Else => write!(f, "Else"),
            Token::ElseIf => write!(f, "ElseIf"),
            Token::Then => write!(f, "Then"),
            Token::End => write!(f, "End"),
            Token::Function => write!(f, "Function"),
            Token::Loop => write!(f, "Loop"),
            Token::While => write!(f, "While"),
            Token::Do => write!(f, "Do"),
            Token::Until => write!(f, "Until"),
            Token::ForEach => write!(f, "ForEach"),
            Token::New => write!(f, "New"),
            Token::Exit => write!(f, "Exit"),
            Token::For => write!(f, "For"),
            Token::To => write!(f, "To"),
            Token::Step => write!(f, "Step"),
            Token::In => write!(f, "In"),
            Token::Each => write!(f, "Each"),
            Token::And => write!(f, "And"),
            Token::Or => write!(f, "Or"),
            Token::Not => write!(f, "Not"),
            Token::Is => write!(f, "Is"),
            Token::Mod => write!(f, "Mod"),
            Token::Xor => write!(f, "Xor"),
            Token::Eqv => write!(f, "Eqv"),
            Token::Imp => write!(f, "Imp"),
            Token::Like => write!(f, "Like"),
            Token::Type => write!(f, "Type"),
            Token::With => write!(f, "With"),
            Token::Property => write!(f, "Property"),
            Token::Get => write!(f, "Get"),
            Token::Let => write!(f, "Let"),
            Token::SetAccessor => write!(f, "SetAccessor"),
            Token::Return => write!(f, "Return"),
            Token::True => write!(f, "True"),
            Token::False => write!(f, "False"),
            Token::Nothing => write!(f, "Nothing"),
            Token::Null => write!(f, "Null"),
            Token::Try => write!(f, "Try"),
            Token::Catch => write!(f, "Catch"),
            Token::Finally => write!(f, "Finally"),
            Token::Throw => write!(f, "Throw"),
            Token::AsResult => write!(f, "AsResult"),
            Token::Match => write!(f, "Match"),
            Token::Case => write!(f, "Case"),
            Token::CaseElse => write!(f, "CaseElse"),
            Token::Select => write!(f, "Select"),
            Token::Continue => write!(f, "Continue"),
            Token::InStr => write!(f, "InStr"),
            Token::Len => write!(f, "Len"),
            Token::Left => write!(f, "Left"),
            Token::Right => write!(f, "Right"),
            Token::Mid => write!(f, "Mid"),
            Token::UCase => write!(f, "UCase"),
            Token::LCase => write!(f, "LCase"),
            Token::Trim => write!(f, "Trim"),
            Token::Replace => write!(f, "Replace"),
            Token::Str => write!(f, "Str"),
            Token::Val => write!(f, "Val"),
            Token::Abs => write!(f, "Abs"),
            Token::Sqr => write!(f, "Sqr"),
            Token::Int => write!(f, "Int"),
            Token::Round => write!(f, "Round"),
            Token::Sin => write!(f, "Sin"),
            Token::Cos => write!(f, "Cos"),
            Token::Tan => write!(f, "Tan"),
            Token::Log => write!(f, "Log"),
            Token::Exp => write!(f, "Exp"),
            Token::Print => write!(f, "Print"),
            Token::InputBox => write!(f, "InputBox"),
            Token::Const => write!(f, "Const"),
            Token::Public => write!(f, "Public"),
            Token::Private => write!(f, "Private"),
            Token::Static => write!(f, "Static"),
            Token::ReDim => write!(f, "ReDim"),
            Token::Preserve => write!(f, "Preserve"),
            Token::Option => write!(f, "Option"),
            Token::Base => write!(f, "Base"),
            Token::Variant => write!(f, "Variant"),
            Token::Currency => write!(f, "Currency"),
            Token::LongLong => write!(f, "LongLong"),
            Token::Long => write!(f, "Long"),
            Token::Integer => write!(f, "Integer"),
            Token::Single => write!(f, "Single"),
            Token::Double => write!(f, "Double"),
            Token::Boolean => write!(f, "Boolean"),
            Token::Byte => write!(f, "Byte"),
            Token::Date => write!(f, "Date"),
            Token::StringType => write!(f, "String"),
            Token::HashMap => write!(f, "HashMap"),
            Token::New => write!(f, "New"),
            Token::Vec => write!(f, "Vec"),
            Token::Error => write!(f, "Error"),
            Token::Warning => write!(f, "Warning"),
            Token::Info => write!(f, "Info"),
            Token::EOF => write!(f, "EOF"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let input = "Dim x As Long = 5".to_string();
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next_token(), Token::Dim);
        assert_eq!(lexer.next_token(), Token::Ident("x".to_string()));
        assert_eq!(lexer.next_token(), Token::As);
        assert_eq!(lexer.next_token(), Token::Long);
        assert_eq!(lexer.next_token(), Token::Equal);
        assert_eq!(lexer.next_token(), Token::Number("5".to_string()));
        assert_eq!(lexer.next_token(), Token::EOF);
    }

    #[test]
    fn test_lexer_strings() {
        let input = r#"Dim s As String = "hello""#.to_string();
        let mut lexer = Lexer::new(input);
        lexer.next_token(); // Dim
        lexer.next_token(); // s
        lexer.next_token(); // As
        assert_eq!(lexer.next_token(), Token::StringType);
        lexer.next_token(); // =
        assert_eq!(lexer.next_token(), Token::StringLiteral("hello".to_string()));
    }
}