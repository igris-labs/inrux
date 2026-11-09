//! Versioned open documents and original-source UTF-16 diagnostic ranges.
use inrux_compiler::{SourceFile, compile_source};
use lsp_server::Notification;
use serde_json::{Value, json};
use std::collections::BTreeMap;
#[derive(Default)]
pub(crate) struct Documents {
    pub(crate) open: BTreeMap<String, Document>,
}
pub(crate) struct Document {
    pub(crate) text: String,
    version: i64,
}
impl Documents {
    /// Full synchronization also diagnoses unsaved files without writing to disk.
    pub(crate) fn notify(&mut self, notification: Notification) -> Option<Notification> {
        let params = notification.params;
        let uri = params["textDocument"]["uri"].as_str()?.to_owned();
        match notification.method.as_str() {
            "textDocument/didOpen" => {
                let text = params["textDocument"]["text"].as_str()?.to_owned();
                let version = params["textDocument"]["version"].as_i64()?;
                self.open.insert(uri.clone(), Document { text, version });
            }
            "textDocument/didChange" => {
                let version = params["textDocument"]["version"].as_i64()?;
                let document = self.open.get_mut(&uri)?;
                if version <= document.version {
                    return None;
                }
                let changes = params["contentChanges"].as_array()?;
                // Full sync is advertised; reject incremental ranges rather than misapplying them.
                let change = changes.last()?;
                if change.get("range").is_some() {
                    return None;
                }
                document.text = change["text"].as_str()?.to_owned();
                document.version = version;
            }
            "textDocument/didClose" => {
                self.open.remove(&uri);
                return Some(Notification::new(
                    "textDocument/publishDiagnostics".into(),
                    json!({"uri":uri,"diagnostics":[]}),
                ));
            }
            _ => return None,
        }
        let document = self.open.get(&uri)?;
        let diagnostics = diagnose(&uri, &document.text);
        Some(Notification::new(
            "textDocument/publishDiagnostics".into(),
            json!({"uri":uri,"version":document.version,"diagnostics":diagnostics}),
        ))
    }
}
/// Convert byte offsets from the compiler into LSP's zero-based UTF-16 positions.
pub(crate) fn position(text: &str, byte: usize) -> Value {
    let mut byte = byte.min(text.len());
    while !text.is_char_boundary(byte) {
        byte -= 1;
    }
    let prefix = &text[..byte];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count();
    let character = prefix
        .rsplit('\n')
        .next()
        .unwrap_or("")
        .encode_utf16()
        .count();
    json!({"line":line,"character":character})
}
/// Expose framework syntax errors; Rust type checking remains Cargo/rustc's responsibility.
fn diagnose(uri: &str, text: &str) -> Vec<Value> {
    match compile_source(SourceFile::new(uri, text)) {
        Ok(_) => vec![],
        Err(error) => {
            let range = error.range();
            let message = error.help().map_or_else(
                || error.message().to_owned(),
                |help| format!("{}\n{help}", error.message()),
            );
            vec![
                json!({"range":{"start":position(text,range.start),"end":position(text,range.end)},
                "severity":1,"code":error.code(),"source":"inrux","message":message}),
            ]
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    /// Emoji count as two UTF-16 units, and CRLF still starts the next line correctly.
    #[test]
    fn unicode_ranges_follow_lsp_utf16_coordinates() {
        assert_eq!(
            position("😀é\r\nλ!", "😀é\r\nλ".len()),
            json!({"line":1,"character":1})
        );
        assert_eq!(position("😀é", "😀".len()), json!({"line":0,"character":2}));
    }
    /// Diagnostics clear after an unsaved correction, with the current document version.
    #[test]
    fn unsaved_documents_publish_and_clear_missing_import_errors() {
        let mut documents = Documents::default();
        let text = "<view>\n<Missing />\n</view>\n<rust>\npub struct State;\n</rust>\n";
        let result = documents
            .notify(Notification::new(
                "textDocument/didOpen".into(),
                json!({"textDocument":{"uri":"file:///App.iru","version":1,"text":text}}),
            ))
            .unwrap();
        assert_eq!(result.params["diagnostics"][0]["code"], "INRUX203");
        let result = documents.notify(Notification::new("textDocument/didChange".into(), json!({"textDocument":{"uri":"file:///App.iru","version":2},"contentChanges":[{"text":text.replace("Missing", "Text")}] }))).unwrap();
        assert_eq!(result.params["diagnostics"], json!([]));
    }
}
