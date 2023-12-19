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
            w.write_all(data.as_bytes(), true).await?;
            w.flush().await?;
            return Ok(());
        }
        let data = data.unwrap();
        let encoded = base64::engine::general_purpose::STANDARD.encode(data);
        let bytes = encoded.as_bytes();
        for (i, chunk) in bytes.chunks(4096).enumerate() {
            // write control part
            let control = format!(
                "\x1b_G{}m={};",
                self,
                u8::from((i + 1) * 4096 < bytes.len()),
            );
            let mut line: Vec<u8> = vec![];
            line.extend(control.as_bytes());
            line.extend(chunk);
            line.extend(b"\x1b\\");
            w.write_all(&line, true).await?;
            w.flush().await?;
        }
        w.flush().await?;
        Ok(())
    }
}

pub async fn transmit_image(
    data: &[u8], w: &mut TermWriter, id: ID,
) -> anyhow::Result<()> {
    let action = ActionTransmission {
        format: Format::Png,
        medium: Medium::Direct,
        compression: false,
        placement: Placement(None),
        ..Default::default()
    };
    let cmd = Command {
        action: Action::Transmit(action),
        quietness: Quietness::SuppressAll,
        id: Some(id),
    };
    cmd.send(Some(data), w).await
}

pub async fn display_image(w: &mut TermWriter, id: ID) -> anyhow::Result<()> {
    let action = ActionPut {
        x: 0,
        y: 0,
        w: 0,
        h: 0,
        x_offset: 0,
        y_offset: 0,
        columns: 0,
        rows: 0,
        move_cursor: false,
        unicode_placeholder: false,
        z_index: 0,
        parent_image: None,
        parent_placement: Placement(None),
        cell_relative_offset_horizontal: 0,
        cell_relative_offset_vertical: 0,
        placement: Placement(Some(id.0)),
    };
    let cmd = Command {
        action: Action::Put(action),
        quietness: Quietness::SuppressAll,
        id: Some(id),
    };
    cmd.send(None, w).await
}

pub async fn delete_image(w: &mut TermWriter, id: ID) -> anyhow::Result<()> {
    let action = ActionDelete {
        hard: true,
        target: DeleteTarget::ID {
            placement: Placement(Some(id.0)),
        },
    };
    let cmd = Command {
        action: Action::Delete(action),
        quietness: Quietness::SuppressAll,
        id: Some(id),
    };
    cmd.send(None, w).await
}
