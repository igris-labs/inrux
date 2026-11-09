//! Inrux compiler diagnostics and comment-preserving embedded Rust formatting over LSP.
mod documents;
mod requests;
use lsp_server::{Connection, Message};
use serde_json::json;

/// Keep protocol framing and shutdown in the established LSP transport library.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (connection, threads) = Connection::stdio();
    connection.initialize(json!({
        "positionEncoding":"utf-16", "textDocumentSync":1,
        "documentFormattingProvider":true
    }))?;
    let mut documents = documents::Documents::default();
    for message in &connection.receiver {
        match message {
            Message::Request(request) => {
                if connection.handle_shutdown(&request)? {
                    break;
                }
                let response = requests::respond(request, &documents);
                connection.sender.send(Message::Response(response))?;
            }
            Message::Notification(notification) => {
                if let Some(diagnostics) = documents.notify(notification) {
                    connection.sender.send(Message::Notification(diagnostics))?;
                }
            }
            Message::Response(_) => {}
        }
    }
    drop(connection);
    threads.join()?;
    Ok(())
}
