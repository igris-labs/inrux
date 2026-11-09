//! Text normalization shared by descriptors, clipboard and platform input bridges.
use std::borrow::Cow;
/// Normalize hard line breaks, preserving multiline text only for TextArea.
/// CRLF is one break; ordinary Unicode and whitespace remain untouched.
pub fn normalize_text(text: &str, multiline: bool) -> Cow<'_, str> {
    let hard_break = |c| {
        matches!(
            c,
            '\r' | '\n' | '\u{b}' | '\u{c}' | '\u{85}' | '\u{2028}' | '\u{2029}'
        )
    };
    if !text
        .chars()
        .any(|c| hard_break(c) && (!multiline || c != '\n'))
    {
        return Cow::Borrowed(text);
    }
    let mut output = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\r' && chars.peek() == Some(&'\n') {
            chars.next();
        }
        output.push(if hard_break(c) {
            if multiline { '\n' } else { ' ' }
        } else {
            c
        });
    }
    Cow::Owned(output)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn breaks_are_consistent_across_platforms() {
        let text = "é\r\n🙂\rA\nB\u{b}C\u{c}D\u{85}E\u{2028}F\u{2029}G";
        assert_eq!(normalize_text(text, false), "é 🙂 A B C D E F G");
        assert_eq!(normalize_text(text, true), "é\n🙂\nA\nB\nC\nD\nE\nF\nG");
        assert!(matches!(normalize_text("é🙂", false), Cow::Borrowed(_)));
        assert!(matches!(normalize_text("a\nb", true), Cow::Borrowed(_)));
    }
}
