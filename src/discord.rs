//! Discord Rich Presence integration for Digital Vigital (DV) Launcher.
//!
//! Provides non-blocking, asynchronous IPC communication with the local Discord client.
//! Displays "Minecraft via Digital Vigital" with current player activity and elapsed time.

use crate::branding;
use serde_json::json;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tracing::{debug, info};

pub const DEFAULT_DISCORD_CLIENT_ID: &str = "1341050215758069811";
pub const DISCORD_LOGO_URL: &str =
    "https://raw.githubusercontent.com/SYN606/dv-client/main/assets/logo.png";

#[derive(Clone, Debug)]
pub enum DiscordAction {
    SetActivity {
        state: String,
        details: String,
        start_time: Option<u64>,
    },
    ClearActivity,
    Shutdown,
}

#[derive(Clone)]
pub struct DiscordRpcHandle {
    tx: mpsc::UnboundedSender<DiscordAction>,
}

impl DiscordRpcHandle {
    pub fn new(client_id: Option<String>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let cid = client_id.unwrap_or_else(|| DEFAULT_DISCORD_CLIENT_ID.to_string());

        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                run_discord_ipc_loop(cid, rx).await;
            });
        } else {
            std::thread::spawn(move || {
                if let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() {
                    rt.block_on(run_discord_ipc_loop(cid, rx));
                }
            });
        }

        Self { tx }
    }

    pub fn set_in_launcher(&self) {
        let _ = self.tx.send(DiscordAction::SetActivity {
            state: "In Launcher (Idle)".to_string(),
            details: branding::DISCORD_RPC_DETAILS.to_string(),
            start_time: None,
        });
    }

    pub fn set_playing(&self, mc_version: &str) {
        self.set_playing_with_options(mc_version, true, true);
    }

    pub fn set_playing_with_options(
        &self,
        mc_version: &str,
        show_version: bool,
        show_elapsed: bool,
    ) {
        let now = if show_elapsed {
            Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            )
        } else {
            None
        };

        let state_text = if show_version && !mc_version.trim().is_empty() {
            format!("Playing Minecraft {}", mc_version.trim())
        } else {
            "Playing Minecraft".to_string()
        };

        let _ = self.tx.send(DiscordAction::SetActivity {
            state: state_text,
            details: branding::DISCORD_RPC_DETAILS.to_string(),
            start_time: now,
        });
    }

    pub fn clear(&self) {
        let _ = self.tx.send(DiscordAction::ClearActivity);
    }
}

fn get_pipe_path(pipe_num: u8) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        PathBuf::from(format!(r"\\.\pipe\discord-ipc-{}", pipe_num))
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            let p = PathBuf::from(runtime_dir).join(format!("discord-ipc-{}", pipe_num));
            if p.exists() {
                return p;
            }
        }
        if let Ok(tmp_dir) = std::env::var("TMPDIR") {
            let p = PathBuf::from(tmp_dir).join(format!("discord-ipc-{}", pipe_num));
            if p.exists() {
                return p;
            }
        }
        PathBuf::from(format!("/tmp/discord-ipc-{}", pipe_num))
    }
}

#[cfg(not(target_os = "windows"))]
type IpcStream = tokio::net::UnixStream;

#[cfg(target_os = "windows")]
type IpcStream = tokio::net::windows::named_pipe::NamedPipeClient;

async fn try_connect_ipc() -> Option<IpcStream> {
    for i in 0..10 {
        let path = get_pipe_path(i);
        #[cfg(not(target_os = "windows"))]
        {
            if let Ok(stream) = tokio::net::UnixStream::connect(&path).await {
                return Some(stream);
            }
        }
        #[cfg(target_os = "windows")]
        {
            if let Ok(client) = tokio::net::windows::named_pipe::ClientOptions::new().open(&path) {
                return Some(client);
            }
        }
    }
    None
}

async fn send_ipc_frame<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    opcode: u32,
    payload: &str,
) -> Result<(), std::io::Error> {
    let len = payload.len() as u32;
    writer.write_all(&opcode.to_le_bytes()).await?;
    writer.write_all(&len.to_le_bytes()).await?;
    writer.write_all(payload.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

async fn read_ipc_frame<R: tokio::io::AsyncReadExt + Unpin>(
    reader: &mut R,
) -> Result<(u32, Vec<u8>), std::io::Error> {
    let mut header = [0u8; 8];
    reader.read_exact(&mut header).await?;
    let opcode = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
    let length = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;

    let mut buf = vec![0u8; length];
    reader.read_exact(&mut buf).await?;
    Ok((opcode, buf))
}

async fn run_discord_ipc_loop(client_id: String, mut rx: mpsc::UnboundedReceiver<DiscordAction>) {
    let pid = std::process::id();
    let mut current_stream: Option<IpcStream> = None;
    let mut last_activity: Option<DiscordAction> = None;

    let mut retry_interval = tokio::time::interval(std::time::Duration::from_secs(10));
    retry_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            action_opt = rx.recv() => {
                let action = match action_opt {
                    Some(a) => a,
                    None => break,
                };

                if matches!(action, DiscordAction::Shutdown) {
                    break;
                }

                last_activity = Some(action.clone());

                if current_stream.is_none() {
                    current_stream = try_connect_ipc().await;
                    if let Some(ref mut stream) = current_stream {
                        let handshake = json!({ "v": 1, "client_id": client_id }).to_string();
                        if send_ipc_frame(stream, 0, &handshake).await.is_ok() {
                            let _ = read_ipc_frame(stream).await;
                            info!("Connected to Discord IPC.");
                        } else {
                            current_stream = None;
                        }
                    }
                }

                if let Some(ref mut stream) = current_stream {
                    let success = dispatch_activity(stream, pid, &action).await;
                    if !success {
                        current_stream = None;
                    }
                }
            }

            _ = retry_interval.tick() => {
                if current_stream.is_none() {
                    if let Some(mut stream) = try_connect_ipc().await {
                        let handshake = json!({ "v": 1, "client_id": client_id }).to_string();
                        if send_ipc_frame(&mut stream, 0, &handshake).await.is_ok() {
                            let _ = read_ipc_frame(&mut stream).await;
                            info!("Connected to Discord IPC on retry interval.");

                            if let Some(ref action) = last_activity {
                                let _ = dispatch_activity(&mut stream, pid, action).await;
                            }
                            current_stream = Some(stream);
                        }
                    }
                }
            }
        }
    }

    if let Some(mut stream) = current_stream {
        let _ = send_ipc_frame(&mut stream, 2, "{}").await;
    }
}

async fn dispatch_activity<S: tokio::io::AsyncWriteExt + tokio::io::AsyncReadExt + Unpin>(
    stream: &mut S,
    pid: u32,
    action: &DiscordAction,
) -> bool {
    let payload = match action {
        DiscordAction::SetActivity {
            state,
            details,
            start_time,
        } => {
            let mut activity = json!({
                "details": details,
                "state": state,
                "assets": {
                    "large_image": DISCORD_LOGO_URL,
                    "large_text": "Digital Vigital (DV) Launcher",
                    "small_image": DISCORD_LOGO_URL,
                    "small_text": "Digital Vigital"
                },
                "buttons": [
                    {
                        "label": "Play Minecraft",
                        "url": "https://digitalvigital.fun/minecraft"
                    },
                    {
                        "label": "Join Discord",
                        "url": "https://discord.gg/flex-dv"
                    }
                ]
            });

            if let Some(start) = start_time {
                activity["timestamps"] = json!({ "start": start });
            }

            json!({
                "cmd": "SET_ACTIVITY",
                "args": {
                    "pid": pid,
                    "activity": activity
                },
                "nonce": uuid::Uuid::new_v4().to_string()
            })
            .to_string()
        }
        DiscordAction::ClearActivity => json!({
            "cmd": "SET_ACTIVITY",
            "args": {
                "pid": pid,
                "activity": null
            },
            "nonce": uuid::Uuid::new_v4().to_string()
        })
        .to_string(),
        DiscordAction::Shutdown => return true,
    };

    if send_ipc_frame(stream, 1, &payload).await.is_ok() {
        let _ = read_ipc_frame(stream).await;
        debug!("Sent Discord presence update.");
        true
    } else {
        false
    }
}
