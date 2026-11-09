//! Handle only capabilities that the server advertises and can validate.
use crate::documents::{Documents, position};
use inrux_compiler::{SourceFile, editor_document, format_source};
use lsp_server::{Request, Response};
use serde_json::{Value, json};
/// Format embedded Rust atomically; unsupported requests receive the standard method-not-found error.
pub(crate) fn respond(request: Request, documents: &Documents) -> Response {
    if request.method == "inrux/documentInfo" {
        let result = document_info(&request.params, documents);
        return match result {
            Ok(info) => Response::new_ok(request.id, info),
            Err(error) => Response::new_err(request.id, -32602, error.to_string()),
        };
    }
    if request.method != "textDocument/formatting" {
        return Response::new_err(
            request.id,
            -32601,
            format!("Unsupported method {}", request.method),
        );
    }
    let result = format_document(&request.params, documents);
    match result {
        Ok(edits) => Response::new_ok(request.id, edits),
        Err(error) => Response::new_err(request.id, -32603, error.to_string()),
    }
}

/// Return compiler-derived regions/imports rather than maintaining a second SFC parser in TypeScript.
fn document_info(
    params: &Value,
    documents: &Documents,
) -> Result<Value, Box<dyn std::error::Error>> {
    let uri = params["textDocument"]["uri"]
        .as_str()
        .ok_or("Missing document URI")?;
    let text = params["text"]
        .as_str()
        .or_else(|| documents.open.get(uri).map(|doc| doc.text.as_str()))
        .ok_or("Document is not open")?;
    let info = editor_document(SourceFile::new(uri, text))?;
    let range = |range: std::ops::Range<usize>| json!({"start":position(text, range.start),"end":position(text, range.end)});
    Ok(
        json!({"rust":range(info.rust),"view":range(info.view),"styles":info.styles.into_iter().map(range).collect::<Vec<_>>(),"imports":info.imports,
            "bindings":info.bindings.into_iter().map(|(name, span)|(name,range(span))).collect::<std::collections::BTreeMap<_,_>>()}),
    )
}
/// Preserve all non-Rust bytes, comments, and original document ownership until formatting succeeds.
fn format_document(
    params: &Value,
    documents: &Documents,
) -> Result<Value, Box<dyn std::error::Error>> {
    let uri = params["textDocument"]["uri"]
        .as_str()
        .ok_or("Missing document URI")?;
    let text = params["text"]
        .as_str()
        .or_else(|| documents.open.get(uri).map(|doc| doc.text.as_str()))
        .ok_or("Document is not open")?;
    let formatted = format_source(SourceFile::new(uri, text))?;
    if formatted == text {
        return Ok(json!([]));
    }
    Ok(
        json!([{"range":{"start":{"line":0,"character":0},"end":position(text,text.len())},"newText":formatted}]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A supplied editor snapshot wins over delayed full-sync notifications.
    #[test]
    fn formatting_uses_the_requested_snapshot_without_an_open_document() {
        let text = "<view>\n<Text />\n</view>\n<rust>\npub struct State{pub value:u32}\n</rust>\n";
        let params = json!({"textDocument":{"uri":"file:///App.iru"},"text":text});
        let result = format_document(&params, &Documents::default()).unwrap();
        assert!(
            result[0]["newText"]
                .as_str()
                .unwrap()
                .contains("pub value: u32")
        );
    }

    /// Incomplete code still exposes regions through the real compiler parser.
    #[test]
    fn editor_info_returns_unsaved_ranges_without_requiring_valid_rust() {
        let text = "<view>\n<Text />\n</view>\n<rust>\nfn incomplete(\n</rust>\n";
        let info = document_info(
            &json!({"textDocument":{"uri":"file:///App.iru"},"text":text}),
            &Documents::default(),
        )
        .unwrap();
        assert_eq!(info["rust"]["start"]["line"], 3);
        assert_eq!(info["rust"]["end"]["line"], 5);
        assert_eq!(info["imports"], json!({}));
    }
}
