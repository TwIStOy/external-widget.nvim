use std::{num::NonZeroU32, sync::atomic::AtomicU32};

use crate::{
    kitty::{
        delete_image, transmit_image, Action, ActionPut, Command, Placement,
        Quietness, ID,
    },
    TermWriter,
};

static IMAGE_ID: AtomicU32 = AtomicU32::new(1);

#[derive(Debug)]
pub struct Image {
    id: NonZeroU32,
    buffer: Vec<u8>,
}

impl Image {
    pub fn from_buffer(buffer: Vec<u8>) -> Self {
        let id: NonZeroU32 = IMAGE_ID
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            .try_into()
            .unwrap();

        Self { id, buffer }
    }

    pub async fn transmit(&self, w: &mut TermWriter) -> anyhow::Result<()> {
        transmit_image(&self.buffer, w, ID(self.id)).await
    }

    pub async fn render_at(
        &self, w: &mut TermWriter, x: u32, y: u32,
    ) -> anyhow::Result<()> {
        let action = ActionPut {
            x_offset: x,
            y_offset: y,
            move_cursor: false,
            placement: Placement(Some(self.id)),
            ..Default::default()
        };
        let cmd = Command {
            action: Action::Put(action),
            quietness: Quietness::None,
            id: Some(ID(self.id)),
        };
        cmd.send(None, w).await
    }

    pub async fn delete_image(&self, w: &mut TermWriter) -> anyhow::Result<()> {
        delete_image(w, ID(self.id)).await
    }
}
