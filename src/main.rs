//! The `waymark` binary: an MCP server over stdio. All logic is in the library.
//! Logs go to stderr only, because stdout carries the protocol.

use waymark::mcp::{args::Args, context::Context, serve_stdio, Server};

#[tokio::main]
async fn main() {
    let args = Args::parse_lenient();
    let server = Server::new(Context::new(args), waymark::tools::registry());
    server.start_join();
    if let Err(e) = serve_stdio(server).await {
        eprintln!("waymark: {e}");
        std::process::exit(1);
    }
}
