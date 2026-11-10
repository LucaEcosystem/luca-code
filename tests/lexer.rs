use luca_code::lexer::{lex, TokenKind};

fn indent_tokens(source: &str) -> (usize, usize, usize) {
    let tokens = lex(source).expect("source should lex");
    let indents = tokens
        .iter()
        .filter(|t| matches!(t.kind, TokenKind::Indent))
        .count();
    let dedents = tokens
        .iter()
        .filter(|t| matches!(t.kind, TokenKind::Dedent))
        .count();
    (indents, dedents, tokens.len())
}

#[test]
fn comment_only_lines_do_not_change_indentation() {
    let (indents, dedents, _) =
        indent_tokens("if true then :\n    print(1)\n        -- deeper comment\n    print(2)\n");
    assert_eq!((indents, dedents), (1, 1));

    let (indents, dedents, _) =
        indent_tokens("if true then :\n    print(1)\n-- column zero comment\n    print(2)\n");
    assert_eq!((indents, dedents), (1, 1));
}

#[test]
fn block_comments_can_start_at_any_indent() {
    let (indents, dedents, _) = indent_tokens(
        "if true then :\n    print(1)\n        [[ deeper\n        comment ]]\n    print(2)\n",
    );
    assert_eq!((indents, dedents), (1, 1));
}

#[test]
fn unterminated_block_comment_is_an_error() {
    let error = lex("print(1)\n[[ never closed\n").unwrap_err();
    assert_eq!(error.message, "Unterminated multiline comment");
}

#[test]
fn line_comment_runs_to_end_of_line_only() {
    let tokens = lex("print(1) -- ignored\nprint(2)\n").unwrap();
    assert_eq!(
        tokens
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::Print))
            .count(),
        2
    );
}
