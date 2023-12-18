mod action;
mod actions;
mod common;

use std::fmt::{Display, Formatter};

pub use action::*;
pub use actions::*;
use base64::Engine;
pub use common::*;

use crate::term::TermWriter;

/// A command to the kitty graphics protocol.
pub struct Command {
    /// What action to do
    pub action: Action,
    /// How quiet to do it
    pub quietness: Quietness,
    /// The id of the image
    pub id: Option<ID>,
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.quietness)?;
        write!(f, "{}", self.action)?;
        if let Some(id) = self.id {
            write!(f, "i={id},")?;
        }
        Ok(())
    }
}

impl Command {
    pub async fn send(
        &self, data: Option<&[u8]>, w: &mut TermWriter,
    ) -> anyhow::Result<()> {
        if data.is_none() {
            // only write the command
            let data = format!("\x1b_G{}\x1b\\", self);
            return w.write_all(data.as_bytes(), true).await;
        }
        let data = data.unwrap();
        let encoded = base64::engine::general_purpose::STANDARD.encode(data);
        let bytes = encoded.as_bytes();
        for (i, chunk) in bytes.chunks(4096).enumerate() {
            // write control part
            let control = format!(
                "\x1b_G{},m={};",
                self,
                u8::from((i + 1) * 4096 < bytes.len()),
            );
            w.write_all(control.as_bytes(), true).await?;
            w.write_all(chunk, true).await?;
            w.write_all(b"\x1b\\", true).await?;
        }
        Ok(())
    }
}
