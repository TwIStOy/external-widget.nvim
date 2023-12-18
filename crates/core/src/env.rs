use tokio::process::Command;

pub fn in_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}

pub fn in_ssh() -> bool {
    std::env::var("SSH_CLIENT").is_ok() || std::env::var("SSH_TTY").is_ok()
}

pub async fn get_tty() -> anyhow::Result<String> {
    let ret = Command::new("tty").output().await?;
    let ret = ret.stdout;
    let ret = String::from_utf8(ret)?;
    let ret = ret.trim().to_string();
    Ok(ret)
}
