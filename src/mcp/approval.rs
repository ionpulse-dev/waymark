//! Approval by MCP elicitation (see `write-approval`). All `rmcp` code of approval is here.

use crate::error::{Code, Error};
use rmcp::model::{
    ElicitRequestParams, ElicitationAction, ProgressNotificationParam, ProgressToken,
};
use rmcp::service::{Peer, ServiceError};
use rmcp::RoleServer;
use serde_json::json;
use std::time::Duration;
use tokio::runtime::Handle;

/// What a tool call needs to ask the person: the connection to the client.
#[derive(Clone)]
pub struct Channel {
    pub peer: Peer<RoleServer>,
    pub handle: Handle,
    pub progress_token: Option<ProgressToken>,
}

const DEFAULT_TIMEOUT: u64 = 120;

fn timeout() -> Duration {
    let secs = std::env::var("WAYMARK_APPROVAL_TIMEOUT_SECONDS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|v| (1..=600).contains(v))
        .unwrap_or(DEFAULT_TIMEOUT);
    Duration::from_secs(secs)
}

fn unavailable(message: &str) -> Error {
    Error::new(Code::ApprovalUnavailable, message).with_next(
        "Use a client that supports elicitation, or ask the owner to remove the approval rule.",
    )
}

fn denied(message: &str) -> Error {
    Error::new(Code::ApprovalDenied, message).with_next("Nothing was written.")
}

/// Ask the person. `Ok` means yes. Must run on a blocking thread, not on a runtime thread.
pub fn ask(channel: Option<&Channel>, message: String) -> Result<(), Error> {
    let Some(ch) = channel else {
        return Err(unavailable(
            "This write needs the approval of the person, and the client cannot ask for it.",
        ));
    };
    if !ch
        .peer
        .supported_elicitation_modes()
        .contains(&rmcp::service::ElicitationMode::Form)
    {
        return Err(unavailable(
            "This write needs the approval of the person, and the client cannot ask for it.",
        ));
    }
    let params: ElicitRequestParams = serde_json::from_value(json!({
        "mode": "form",
        "message": message,
        "requestedSchema": {
            "type": "object",
            "properties": {
                "approve": {"type": "boolean", "title": "Approve this write", "default": false}
            },
            "required": ["approve"]
        }
    }))
    .map_err(|e| unavailable(&format!("The approval request could not be built: {e}")))?;
    let limit = timeout();
    let peer = ch.peer.clone();
    let token = ch.progress_token.clone();
    let answer = ch.handle.block_on(async move {
        // Progress messages keep the call alive in clients that cut a silent call.
        let ticker = token.map(|token| {
            let peer = peer.clone();
            let every = Duration::from_secs(10)
                .min(limit / 8)
                .max(Duration::from_millis(100));
            tokio::spawn(async move {
                let mut n = 0.0;
                loop {
                    tokio::time::sleep(every).await;
                    n += 1.0;
                    let mut p = ProgressNotificationParam::new(token.clone(), n);
                    p.message = Some("Waiting for approval".to_string());
                    if peer.notify_progress(p).await.is_err() {
                        break;
                    }
                }
            })
        });
        let r = peer
            .create_elicitation_with_timeout(params, Some(limit))
            .await;
        if let Some(t) = ticker {
            t.abort();
        }
        r
    });
    match answer {
        Ok(r) => match r.action {
            ElicitationAction::Accept => {
                let yes = r
                    .content
                    .as_ref()
                    .and_then(|c| c.get("approve"))
                    .and_then(|v| v.as_bool())
                    == Some(true);
                if yes {
                    Ok(())
                } else {
                    Err(denied("The person did not approve."))
                }
            }
            ElicitationAction::Decline => Err(denied("The person said no.")),
            _ => Err(denied("The person closed the question.")),
        },
        Err(ServiceError::Timeout { .. }) => Err(denied("No answer in 2 minutes.")),
        Err(_) => Err(unavailable(
            "The client could not show the approval request.",
        )),
    }
}
