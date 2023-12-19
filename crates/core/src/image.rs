use std::{
    borrow::BorrowMut,
    collections::HashMap,
    num::NonZeroU32,
    sync::{atomic::AtomicU32, Arc},
};

use parking_lot::Mutex;

use crate::{
    kitty::{
        delete_image, transmit_image, Action, ActionPut, Command, Placement,
        Quietness, ID,
    },
    TermWriter,
};

static IMAGE_ID: AtomicU32 = AtomicU32::new(1);
pub static IMAGE_MANAGER: once_cell::sync::Lazy<Mutex<ImageManager>> =
    once_cell::sync::Lazy::new(|| Mutex::new(ImageManager::new()));

#[derive(Debug)]
pub struct ImageManager {
    images: HashMap<NonZeroU32, Arc<Image>>,
}

#[derive(Debug)]
pub struct Image {
    id: NonZeroU32,
    buffer: Vec<u8>,
    transmitted: Mutex<bool>,
}

impl ImageManager {
    fn new() -> Self {
        Self {
            images: HashMap::new(),
        }
    }

    pub fn alloc_id() -> NonZeroU32 {
        let id: NonZeroU32 = IMAGE_ID
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            .try_into()
            .unwrap();
        id
    }

    pub fn new_image_from_buffer(&mut self, buffer: Vec<u8>) -> Arc<Image> {
        let image = Image::from_buffer(buffer);
        let image = Arc::new(image);
        self.images.insert(image.id, image.clone());
        image
    }

    pub fn new_image_from_id_buffer(
        &mut self, id: NonZeroU32, buffer: Vec<u8>,
    ) -> Arc<Image> {
        let image = Image::from_id_buffer(id, buffer);
        let image = Arc::new(image);
        self.images.insert(image.id, image.clone());
        image
    }
}

impl Image {
    fn from_id_buffer(id: NonZeroU32, buffer: Vec<u8>) -> Self {
        Self {
            id,
            buffer,
            transmitted: Mutex::new(false),
        }
    }

    fn from_buffer(buffer: Vec<u8>) -> Self {
        let id: NonZeroU32 = IMAGE_ID
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            .try_into()
            .unwrap();

        Self {
            id,
            buffer,
            transmitted: Mutex::new(false),
        }
    }

    pub async fn transmit(&self) -> anyhow::Result<()> {
        let mut transmitted = self.transmitted.lock();
        if *transmitted {
            return Ok(());
        }
        let mut writer = TermWriter::new().await?;
        transmit_image(&self.buffer, &mut writer, ID(self.id)).await?;
        *transmitted = true;
        writer.flush().await
    }

    pub async fn render_at(&self, x: u32, y: u32) -> anyhow::Result<()> {
        let mut writer = TermWriter::new().await?;
        let mut transmitted = self.transmitted.lock();
        if !*transmitted {
            transmit_image(&self.buffer, &mut writer, ID(self.id)).await?;
            *transmitted = true;
        }
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
        cmd.send(None, &mut writer).await?;
        writer.flush().await
    }

    pub async fn delete_image(&self) -> anyhow::Result<()> {
        let mut writer = TermWriter::new().await?;
        delete_image(&mut writer, ID(self.id)).await?;
        let mut transmitted = self.transmitted.lock();
        *transmitted = false;
        writer.flush().await
    }
}
