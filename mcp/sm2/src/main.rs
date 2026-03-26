use std::error::Error;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, ReadBuf};

mod tool_service;
use rmcp::serve_server;

use crate::tool_service::ToolService;

struct LoggingReader<R> {
    inner: R,
}

impl<R: AsyncRead + Unpin> AsyncRead for LoggingReader<R> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        let before = buf.filled().len();
        let result = Pin::new(&mut self.inner).poll_read(cx, buf);
        let bytes = &buf.filled()[before..];
        if !bytes.is_empty() {
            eprintln!("[MCP RAW] {}", String::from_utf8_lossy(bytes));
        }
        result
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let io = (LoggingReader { inner: tokio::io::stdin() }, tokio::io::stdout());

    serve_server(ToolService::new(), io).await?.waiting().await?;
    Ok(())
}
