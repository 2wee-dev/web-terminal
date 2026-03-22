use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::{
    io::{Read, Write},
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

mod frontend;

/// How long a session can be idle before the PTY is killed.
const INACTIVITY_TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// Per-session state. Created when a WebSocket connection is established,
/// cleaned up (including temp HOME directory) when the session ends.
struct Session {
    id: Uuid,
    server_url: String,
    home_dir: PathBuf,
    last_activity: Arc<Mutex<Instant>>,
}

impl Session {
    fn new(server_url: String) -> std::io::Result<Self> {
        let id = Uuid::new_v4();
        let home_dir = PathBuf::from(format!("/tmp/2wee-sessions/{}", id));
        std::fs::create_dir_all(&home_dir)?;
        Ok(Self {
            id,
            server_url,
            home_dir,
            last_activity: Arc::new(Mutex::new(Instant::now())),
        })
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.home_dir);
        tracing::info!(session = %self.id, "Session ended, cleaned up {}", self.home_dir.display());
    }
}

/// Resolve the server URL for a session:
/// - If TWO_WEE_SERVER is set, always use it (org lock-in mode).
/// - Otherwise use the ?server= query param if provided.
/// - Otherwise return None (landing page should be shown instead).
fn resolve_server(query_server: Option<String>) -> Option<String> {
    if let Ok(locked) = std::env::var("TWO_WEE_SERVER") {
        if !locked.is_empty() {
            return Some(locked);
        }
    }
    query_server.filter(|s| !s.is_empty())
}

#[derive(serde::Deserialize)]
struct TerminalQuery {
    server: Option<String>,
    cols: Option<u16>,
    rows: Option<u16>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/terminal.js", get(frontend::js_handler))
        .route("/ws", get(ws_handler))
        .layer(CorsLayer::permissive());

    let port = std::env::var("TWO_WEE_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(7681);
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("2Wee web terminal listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index_handler(Query(query): Query<TerminalQuery>) -> impl IntoResponse {
    match resolve_server(query.server) {
        Some(server_url) => frontend::terminal_page(&server_url).await,
        None => frontend::landing_page().await,
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<TerminalQuery>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_session(socket, query))
}

async fn handle_session(socket: WebSocket, query: TerminalQuery) {
    let server_url = match resolve_server(query.server) {
        Some(url) => url,
        None => {
            tracing::warn!("WebSocket connection rejected: no server URL");
            return;
        }
    };

    let session = match Session::new(server_url) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to create session directory: {}", e);
            return;
        }
    };

    tracing::info!(session = %session.id, server = %session.server_url, "Session started");

    let cols = query.cols.unwrap_or(220);
    let rows = query.rows.unwrap_or(50);

    let pty_system = NativePtySystem::default();
    let pair = match pty_system.openpty(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 }) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(session = %session.id, "Failed to open PTY: {}", e);
            return;
        }
    };

    let client_bin = std::env::var("TWO_WEE_CLIENT_BIN")
        .unwrap_or_else(|_| "two_wee_client".to_string());

    let mut cmd = CommandBuilder::new(&client_bin);
    cmd.arg(&session.server_url);
    cmd.env("HOME", &session.home_dir);

    let mut child = match pair.slave.spawn_command(cmd) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(session = %session.id, "Failed to spawn two_wee_client: {}", e);
            return;
        }
    };

    // Watch for child exit in a blocking thread and signal via a channel.
    let (exit_tx, exit_rx) = tokio::sync::oneshot::channel::<()>();
    std::thread::spawn(move || {
        let _ = child.wait();
        let _ = exit_tx.send(());
    });

    let pty_reader = match pair.master.try_clone_reader() {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(session = %session.id, "Failed to clone PTY reader: {}", e);
            return;
        }
    };
    let pty_writer = Arc::new(Mutex::new(pair.master.take_writer().unwrap()));
    let pty_master = Arc::new(Mutex::new(pair.master));

    let (ws_sender, ws_receiver) = socket.split();
    let ws_sender: Arc<Mutex<futures_util::stream::SplitSink<WebSocket, Message>>> =
        Arc::new(Mutex::new(ws_sender));

    let last_activity = session.last_activity.clone();

    // PTY → WebSocket
    let sender_clone = ws_sender.clone();
    let activity_clone = last_activity.clone();
    let pty_to_ws = tokio::task::spawn_blocking(move || {
        let mut reader = pty_reader;
        let mut buf = [0u8; 4096];
        let rt = tokio::runtime::Handle::current();
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let data = buf[..n].to_vec();
                    let sender = sender_clone.clone();
                    let activity = activity_clone.clone();
                    rt.block_on(async move {
                        *activity.lock().await = Instant::now();
                        let mut s = sender.lock().await;
                        let _ = s.send(Message::Binary(data.into())).await;
                    });
                }
                Err(_) => break,
            }
        }
    });

    // WebSocket → PTY
    let writer_clone = pty_writer.clone();
    let master_clone = pty_master.clone();
    let activity_clone = last_activity.clone();
    let ws_to_pty = tokio::spawn(async move {
        let mut receiver: futures_util::stream::SplitStream<WebSocket> = ws_receiver;
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Binary(data) => {
                    *activity_clone.lock().await = Instant::now();
                    let mut writer = writer_clone.lock().await;
                    let _ = writer.write_all(&data);
                }
                Message::Text(text) => {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        if json["type"] == "resize" {
                            if let (Some(cols), Some(rows)) =
                                (json["cols"].as_u64(), json["rows"].as_u64())
                            {
                                let master = master_clone.lock().await;
                                let _ = master.resize(PtySize {
                                    rows: rows as u16,
                                    cols: cols as u16,
                                    pixel_width: 0,
                                    pixel_height: 0,
                                });
                                tracing::debug!("Resized PTY to {}x{}", cols, rows);
                            }
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Inactivity watchdog — aborts the session if idle too long
    let activity_clone = last_activity.clone();
    let watchdog = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            let idle = activity_clone.lock().await.elapsed();
            if idle >= INACTIVITY_TIMEOUT {
                tracing::info!("Session idle for {:?}, terminating", idle);
                break;
            }
        }
    });

    tokio::select! {
        _ = pty_to_ws => {}
        _ = ws_to_pty => {}
        _ = watchdog => {}
        _ = exit_rx => {
            tracing::info!(session = %session.id, "Child process exited, closing session");
        }
    }

    // Send an explicit close frame so the browser fires onclose reliably.
    {
        let mut sender = ws_sender.lock().await;
        let _ = sender.send(Message::Close(None)).await;
    }

    // session dropped here → Drop impl removes the HOME directory
}
