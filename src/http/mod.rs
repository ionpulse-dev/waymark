//! The personal server: the MCP core over HTTPS, behind the sign-in door
//! (see `specs/foundation/personal-server`). The host provides TLS. Waymark speaks plain HTTP.

pub mod client;
pub mod health;
pub mod settings;

use crate::auth::{self, github::Github, Door};
use crate::gitsync::github_app::GithubApp;
use crate::mcp::args::Args;
use crate::mcp::context::Context;
use crate::mcp::Server;
use axum::extract::{Request, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use rmcp::transport::streamable_http_server::session::never::NeverSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use settings::ServerSettings;
use std::sync::Arc;

struct Gate {
    door: Arc<Door>,
    ctx: Arc<Context>,
}

/// Every request to the MCP path needs a valid access token for this server.
async fn require_token(State(gate): State<Arc<Gate>>, req: Request, next: Next) -> Response {
    match auth::authenticate(&gate.door, req.headers()) {
        Ok(claims) => {
            // The commit author is the person who signed in.
            gate.ctx.set_remote_author(&claims.name, &claims.email);
            next.run(req).await
        }
        Err(invalid) => {
            let mut r = StatusCode::UNAUTHORIZED.into_response();
            if let Ok(v) = HeaderValue::from_str(&auth::discovery::challenge(&gate.door, invalid)) {
                r.headers_mut().insert(header::WWW_AUTHENTICATE, v);
            }
            r
        }
    }
}

/// The folder for the clone. The temp folder is the writable place on every host. The clone is a
/// cache that can be deleted.
fn data_folder() -> std::path::PathBuf {
    std::env::temp_dir().join("waymark-clone")
}

/// Build the router and the server from the settings. Nothing is started.
pub fn build(settings: Arc<ServerSettings>, mut args: Args) -> Result<(Router, Server), String> {
    if args.path.is_some() {
        crate::log::event(
            "option_ignored",
            &[
                ("option", "--path"),
                ("reason", "personal-server mode uses the temp folder"),
            ],
        );
    }
    args.repo = Some(settings.repo.clone());
    args.path = Some(data_folder());

    let app = GithubApp::new(
        &settings.app_id,
        &settings.installation_id,
        &settings.app_key,
        &settings.github_api,
    );
    app.check_key()?;
    if let Some(prefix) = settings.git_credential_prefix() {
        crate::gitsync::cmd::set_credential(prefix, Box::new(app));
    }

    let server = Server::new(Context::new(args), crate::tools::registry()).with_call_log();
    let ctx = server.context().clone();
    let people_ctx = ctx.clone();
    let door = Arc::new(Door {
        settings: settings.clone(),
        identity: Arc::new(Github {
            client_id: settings.client_id.clone(),
            client_secret: settings.client_secret.clone(),
            web_base: settings.github_web.clone(),
            api_base: settings.github_api.clone(),
        }),
        people: Box::new(move || {
            people_ctx
                .memrepo_if_ready()
                .map(|k| k.settings.people)
                .unwrap_or_default()
        }),
    });

    // Stateless: every request stands alone, so the process can stop at any time.
    // The token protects the server, so the host check for local servers is off.
    let config = StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .disable_allowed_hosts();
    let service_server = server.clone();
    let mcp = StreamableHttpService::new(
        move || Ok(service_server.clone()),
        Arc::new(NeverSessionManager::default()),
        config,
    );
    let gate = Arc::new(Gate {
        door: door.clone(),
        ctx,
    });
    let protected = Router::new()
        .route_service("/mcp", mcp)
        .layer(middleware::from_fn_with_state(gate, require_token));
    let router = Router::new()
        .route("/", get(health::status_page))
        .route("/health", get(health::health))
        .merge(auth::routes(door))
        .merge(protected);
    Ok((router, server))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let term = async {
        if let Ok(mut s) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            s.recv().await;
        }
    };
    #[cfg(not(unix))]
    let term = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = term => {} }
}

/// Run `--mode personal-server` until the process is stopped.
pub async fn run(args: Args) -> Result<(), String> {
    let mut settings = ServerSettings::from_env(&args)?;
    settings.listen = args.listen.clone();
    let settings = Arc::new(settings);
    let (router, server) = build(settings.clone(), args)?;
    let address = settings
        .listen
        .clone()
        .unwrap_or_else(|| format!("0.0.0.0:{}", settings.port));
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .map_err(|e| format!("Cannot listen on {address}: {e}"))?;
    crate::log::event(
        "start",
        &[("mode", "personal-server"), ("listen", &address)],
    );
    // The first clone starts at once and runs in the background. Sign-in never waits for it.
    server.start_join();
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| e.to_string())
}
