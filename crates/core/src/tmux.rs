use anyhow::bail;
use tokio::{
    io::{AsyncWrite, AsyncWriteExt},
    process::Command,
};

use crate::in_tmux;

pub async fn tmux_escape_write<W: AsyncWrite + Unpin>(
    data: &[u8], w: &mut W,
) -> anyhow::Result<()> {
    let mut escaped = vec![];
    escaped.extend(b"\x1bPtmux;");
    for byte in data {
        match byte {
            b'\x1b' => {
                escaped.push(b'\x1b');
                escaped.push(b'\x1b');
            }
            _ => {
                escaped.push(*byte);
            }
        }
    }
    escaped.extend(b"\x1b\\");
    w.write_all(&escaped).await?;
    Ok(())
}

pub async fn enable_tmux_pass_through() -> anyhow::Result<bool> {
    if !in_tmux() {
        bail!("Not in tmux env");
    }
    let output = Command::new("tmux")
        .args(["show", "-Apv", "allow-passthrough"])
        .output()
        .await?;
    let stdout = String::from_utf8(output.stdout).unwrap();
    Ok(stdout.ends_with("on\n"))
}

async fn tmux_display_message(name: &str) -> anyhow::Result<String> {
    if !in_tmux() {
        bail!("Not in tmux env");
    }
    let output = Command::new("tmux")
        .args(["display-message", "-p", format!("#{{{}}}", name).as_str()])
        .output()
        .await?;
    let stdout = String::from_utf8(output.stdout).unwrap();
    Ok(stdout.trim().to_string())
}

pub async fn tmux_pane_tty() -> anyhow::Result<String> {
    tmux_display_message("pane_tty").await
}

pub fn readable_buf(buf: &[u8]) -> String {
    let mut ret: Vec<u8> = vec![];
    for byte in buf {
        match byte {
            b'\x1b' => {
                ret.extend("<Esc>".as_bytes());
            }
            _ => {
                ret.push(*byte);
            }
        }
    }
    String::from_utf8_lossy(&ret).to_string()
}
