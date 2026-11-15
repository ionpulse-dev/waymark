//! The `waymark` binary: an MCP server over stdio (local mode, the default) or over HTTPS
//! (`--mode personal-server`). All logic is in the library.
//! In local mode logs go to stderr only, because stdout carries the protocol.

use waymark::http::settings::{parse_mode, Mode};
use waymark::mcp::{args::Args, context::Context, serve_stdio, Server};

#[tokio::main]
async fn main() {
    let args = Args::parse_lenient();
    if args.print_client {
        match waymark::http::settings::print_client(&|n| std::env::var(n).ok()) {
            Ok(text) => println!("{text}"),
            Err(e) => {
                eprintln!("waymark: {e}");
                std::process::exit(2);
            }
        }
        return;
    }
    let mode = match parse_mode(args.mode.as_deref()) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("waymark: {e}");
            std::process::exit(2);
        }
    };
    match mode {
        Mode::Local => {
            let server = Server::new(Context::new(args), waymark::tools::registry());
            server.start_join();
            if let Err(e) = serve_stdio(server).await {
                eprintln!("waymark: {e}");
                std::process::exit(1);
            }
        }
        Mode::PersonalServer => {
            if let Err(e) = waymark::http::run(args).await {
                eprintln!("waymark: {e}");
                std::process::exit(2);
            }
        }
    }
}
