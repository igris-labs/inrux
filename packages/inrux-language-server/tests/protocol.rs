//! Verify real process framing, unsaved diagnostics, formatting, and clean protocol shutdown.
use lsp_server::{Message, Notification, Request};
use serde_json::json;
use std::{
    io::BufReader,
    process::{Child, Command, Stdio},
    sync::mpsc,
    time::Duration,
};
struct Server(Child);
impl Drop for Server {
    /// Failed assertions must not leave a language server or pipe-reader thread running.
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
/// The VS Code client speaks to this same executable over the established LSP transport.
#[test]
fn real_server_handles_diagnostics_formatting_and_shutdown() {
    let mut server = Server(
        Command::new(env!("CARGO_BIN_EXE_inrux-language-server"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let mut input = server.0.stdin.take().unwrap();
    let mut output = BufReader::new(server.0.stdout.take().unwrap());
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        while let Ok(Some(message)) = Message::read(&mut output) {
            if tx.send(message).is_err() {
                break;
            }
        }
    });
    Message::Request(Request::new(
        1.into(),
        "initialize".into(),
        json!({"capabilities":{}}),
    ))
    .write(&mut input)
    .unwrap();
    let response = rx.recv_timeout(Duration::from_secs(3)).unwrap();
    assert!(matches!(response, Message::Response(response) if response.response_result.is_ok()));
    Message::Notification(Notification::new("initialized".into(), json!({})))
        .write(&mut input)
        .unwrap();
    let text =
        "<template>\n<Missing />\n</template>\n<rust>\npub struct State{pub value:u32}\n</rust>\n";
    Message::Notification(Notification::new("textDocument/didOpen".into(), json!({"textDocument":{"uri":"file:///App.iru","languageId":"inrux","version":1,"text":text}}))).write(&mut input).unwrap();
    let diagnostics = rx.recv_timeout(Duration::from_secs(3)).unwrap();
    assert!(
        matches!(diagnostics, Message::Notification(message) if message.params["diagnostics"][0]["code"] == "INRUX203")
    );
    Message::Request(Request::new(2.into(), "textDocument/formatting".into(), json!({"textDocument":{"uri":"file:///App.iru"},"options":{"tabSize":4,"insertSpaces":true}}))).write(&mut input).unwrap();
    let formatted = rx.recv_timeout(Duration::from_secs(3)).unwrap();
    assert!(
        matches!(formatted, Message::Response(response) if response.response_result.is_ok() && response.response_result.as_ref().unwrap()[0]["newText"].as_str().unwrap().contains("pub value: u32"))
    );
    Message::Request(Request::new(3.into(), "shutdown".into(), json!(null)))
        .write(&mut input)
        .unwrap();
    Message::Notification(Notification::new("exit".into(), json!(null)))
        .write(&mut input)
        .unwrap();
    drop(input);
    rx.recv_timeout(Duration::from_secs(3)).unwrap();
    assert!(server.0.wait().unwrap().success());
}
