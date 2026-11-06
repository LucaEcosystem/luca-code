use crate::{error::LucaError, types::Dec};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier(String), Int(i64), Dec(Dec), String(String),
    Def, Var, Const, Print, True, False, None, Forget, Change,
    Is, To, And, Or, Not, If, Then, Else, Func, Return, For, While, Times, Repeat, From, Stop, Skip,
    List, Tuple, Set, Dict, Import, Try, Capture, Finally, Error, Ask, Raise,
    Plus, Minus, Star, Slash, Percent,
    Equal, EqualEqual, NotEqual, Greater, GreaterEqual, Less, LessEqual,
    Dot, LeftParen, RightParen, LeftBracket, RightBracket, LeftBrace, RightBrace, Comma, Colon, Indent, Dedent, Newline, Eof,
}

#[derive(Debug, Clone)]
pub struct Token { pub kind: TokenKind, pub line: usize, pub column: usize }

pub fn lex(source: &str) -> Result<Vec<Token>, LucaError> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let (mut index, mut line, mut column) = (0, 1, 1);
    let mut indent_stack: Vec<usize> = vec![0];
    let mut at_line_start = true;
    while index < chars.len() {
        if at_line_start {
            at_line_start = false;
            // Calculate indent for this line, ignoring empty and comment lines
            let mut indent = 0usize;
            let mut temp_index = index;
            while temp_index < chars.len() && (chars[temp_index] == ' ' || chars[temp_index] == '\t') {
                indent += 1;
                temp_index += 1;
            }
            let is_empty = temp_index >= chars.len() || chars[temp_index] == '\n' || chars[temp_index] == '\r';
            if !is_empty {
                let current_indent = *indent_stack.last().unwrap();
                if indent > current_indent {
                    indent_stack.push(indent);
                    tokens.push(Token { kind: TokenKind::Indent, line, column: 1 });
                } else if indent < current_indent {
                    while indent < *indent_stack.last().unwrap() {
                        indent_stack.pop();
                        tokens.push(Token { kind: TokenKind::Dedent, line, column: 1 });
                    }
                    if indent != *indent_stack.last().unwrap() {
                        return Err(LucaError::new("Inconsistent indentation", line, 1));
                    }
                }
                // Consume the indent whitespace
                let consumed = indent;
                index = temp_index;
                column = 1 + consumed;
                if index >= chars.len() {
                    continue;
                }
            } else {
                // For empty/comment lines, don't affect indent; let normal handling consume leading spaces
                // Keep at_line_start false so we don't re-evaluate, but next newline will reset it
            }
        }
        let current = chars[index];
        if current == ' ' || current == '\t' || current == '\r' {
            index += 1; column += 1; continue;
        }
        if current == '\n' {
            tokens.push(Token { kind: TokenKind::Newline, line, column });
            index += 1; line += 1; column = 1;
            at_line_start = true;
            continue;
        }
        if current == '-' && chars.get(index + 1) == Some(&'-') {
            while index < chars.len() && chars[index] != '\n' { index += 1; column += 1; }
            continue;
        }
        if current == '[' && chars.get(index + 1) == Some(&'[') {
            let token_line = line;
            let token_column = column;
            index += 2;
            column += 2;
            let mut closed = false;
            while index < chars.len() {
                if chars[index] == ']' && chars.get(index + 1) == Some(&']') {
                    index += 2;
                    column += 2;
                    closed = true;
                    break;
                }
                if chars[index] == '\n' {
                    index += 1;
                    line += 1;
                    column = 1;
                } else {
                    index += 1;
                    column += 1;
                }
            }
            if !closed {
                return Err(LucaError::new("Unterminated multiline comment", token_line, token_column));
            }
            continue;
        }
        let token_line = line;
        let token_column = column;
        let kind = if current == '"' {
            index += 1; column += 1;
            let mut value = String::new();
            let mut closed = false;
            while index < chars.len() {
                match chars[index] {
                    '"' => { index += 1; column += 1; closed = true; break; }
                    '\n' => return Err(LucaError::new("String literal cannot contain a raw newline", line, column)),
                    '\\' => {
                        index += 1; column += 1;
                        let escaped = *chars.get(index).ok_or_else(|| LucaError::new("Unterminated escape sequence", line, column))?;
                        value.push(match escaped { 'n' => '\n', 'r' => '\r', 't' => '\t', '"' => '"', '\\' => '\\', other => other });
                        index += 1; column += 1;
                    }
                    ch => { value.push(ch); index += 1; column += 1; }
                }
            }
            if !closed { return Err(LucaError::new("Unterminated string literal", token_line, token_column)); }
            TokenKind::String(value)
        } else if current.is_ascii_digit() {
            let start = index;
            while index < chars.len() && chars[index].is_ascii_digit() { index += 1; column += 1; }
            let is_dec = chars.get(index) == Some(&'.') && chars.get(index + 1).is_some_and(char::is_ascii_digit);
            if is_dec {
                index += 1; column += 1;
                while index < chars.len() && chars[index].is_ascii_digit() { index += 1; column += 1; }
            }
            let text: String = chars[start..index].iter().collect();
            if is_dec {
                let dec = Dec::parse_literal(&text).map_err(|msg| LucaError::new(msg, token_line, token_column))?;
                TokenKind::Dec(dec)
            } else {
                TokenKind::Int(text.parse().map_err(|_| LucaError::new("Integer literal is out of range", token_line, token_column))?)
            }
        } else if current.is_ascii_alphabetic() || current == '_' {
            let start = index;
            while index < chars.len() && (chars[index].is_ascii_alphanumeric() || chars[index] == '_') { index += 1; column += 1; }
            match chars[start..index].iter().collect::<String>().as_str() {
                "def" => TokenKind::Def, "var" => TokenKind::Var, "const" => TokenKind::Const,
                "print" => TokenKind::Print, "true" => TokenKind::True, "false" => TokenKind::False,
                "none" => TokenKind::None,
                "forget" => TokenKind::Forget, "change" => TokenKind::Change,
                "is" => TokenKind::Is, "to" => TokenKind::To,
                "and" => TokenKind::And, "or" => TokenKind::Or, "not" => TokenKind::Not,
                "if" => TokenKind::If, "then" => TokenKind::Then, "else" => TokenKind::Else,
                "func" => TokenKind::Func, "return" => TokenKind::Return,
                "for" => TokenKind::For, "while" => TokenKind::While, "times" => TokenKind::Times, "repeat" => TokenKind::Repeat, "from" => TokenKind::From, "stop" => TokenKind::Stop, "skip" => TokenKind::Skip,
                "list" => TokenKind::List, "tuple" => TokenKind::Tuple, "set" => TokenKind::Set, "dict" => TokenKind::Dict,
                "import" => TokenKind::Import,
                "try" => TokenKind::Try, "capture" => TokenKind::Capture, "finally" => TokenKind::Finally, "error" => TokenKind::Error,
                "raise" => TokenKind::Raise,
                "ask" => TokenKind::Ask,
                name => TokenKind::Identifier(name.to_owned()),
            }
        } else {
            // Handle two-character operators first
            if current == '=' && chars.get(index + 1) == Some(&'=') {
                index += 2;
                column += 2;
                tokens.push(Token { kind: TokenKind::EqualEqual, line: token_line, column: token_column });
                continue;
            }
            if current == '!' && chars.get(index + 1) == Some(&'=') {
                index += 2;
                column += 2;
                tokens.push(Token { kind: TokenKind::NotEqual, line: token_line, column: token_column });
                continue;
            }
            if current == '>' && chars.get(index + 1) == Some(&'=') {
                index += 2;
                column += 2;
                tokens.push(Token { kind: TokenKind::GreaterEqual, line: token_line, column: token_column });
                continue;
            }
            if current == '<' && chars.get(index + 1) == Some(&'=') {
                index += 2;
                column += 2;
                tokens.push(Token { kind: TokenKind::LessEqual, line: token_line, column: token_column });
                continue;
            }
            index += 1; column += 1;
            match current {
                '+' => TokenKind::Plus, '-' => TokenKind::Minus, '*' => TokenKind::Star,
                '/' => TokenKind::Slash, '%' => TokenKind::Percent, '=' => TokenKind::Equal,
                '>' => TokenKind::Greater, '<' => TokenKind::Less,
                '(' => TokenKind::LeftParen, ')' => TokenKind::RightParen, '.' => TokenKind::Dot,
                '[' => TokenKind::LeftBracket, ']' => TokenKind::RightBracket,
                '{' => TokenKind::LeftBrace, '}' => TokenKind::RightBrace,
                ',' => TokenKind::Comma, ':' => TokenKind::Colon,
                other => return Err(LucaError::new(format!("Unexpected character '{other}'"), token_line, token_column)),
            }
        };
        tokens.push(Token { kind, line: token_line, column: token_column });
    }
    while indent_stack.len() > 1 {
        indent_stack.pop();
        tokens.push(Token { kind: TokenKind::Dedent, line, column });
    }
    tokens.push(Token { kind: TokenKind::Eof, line, column });
    Ok(tokens)
}
