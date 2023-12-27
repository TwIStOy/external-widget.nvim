use std::{
    collections::HashMap,
    num::NonZeroU32,
    sync::{atomic::AtomicU32, Arc},
};

use anyhow::bail;
use parking_lot::Mutex;
use tracing::instrument;

use crate::term::{
    proto::{
        delete_image, transmit_image, Action, ActionPut, Command, Placement,
        Quietness, ID,
    },
    writer::TermWriter,
};

static IMAGE_ID: AtomicU32 = AtomicU32::new(1);
pub static IMAGE_MANAGER: once_cell::sync::Lazy<Mutex<ImageManager>> =
    once_cell::sync::Lazy::new(|| Mutex::new(ImageManager::new()));

#[derive(Debug)]
pub struct ImageManager {
    images: HashMap<NonZeroU32, Arc<Image>>,
    image_sets: HashMap<NonZeroU32, Arc<Mutex<ImageSet>>>,
}

#[derive(Debug)]
pub struct Image {
    id: NonZeroU32,
    buffer: Vec<u8>,
    transmitted: Mutex<bool>,
}

#[derive(Debug)]
pub struct ImageSet {
    id: NonZeroU32,
    images: Vec<Arc<Image>>,
    index: usize,
    last_zindex: u32,
    last_rendered_pos: Option<(u32, u32)>,
}

impl ImageManager {
    fn new() -> Self {
        Self {
            images: HashMap::new(),
            image_sets: HashMap::new(),
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

    pub fn find_image(&self, id: NonZeroU32) -> Option<Arc<Image>> {
        self.images.get(&id).cloned()
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

    #[instrument(skip(self,))]
    pub async fn transmit(
        self: &Arc<Self>, writer: &mut TermWriter,
    ) -> anyhow::Result<()> {
        {
            let mut transmitted = self.transmitted.lock();
            if *transmitted {
                return Ok(());
            }
            *transmitted = true;
        }
        transmit_image(&self.buffer, writer, ID(self.id)).await?;
        writer.flush().await
    }

    #[instrument(skip(self))]
    pub async fn render_at(
        &self, writer: &mut TermWriter, x: u32, y: u32, z: u32,
    ) -> anyhow::Result<()> {
        let mut should_transmit = false;
        {
            let mut transmitted = self.transmitted.lock();
            if !*transmitted {
                *transmitted = true;
                should_transmit = true;
            }
        }
        if should_transmit {
            transmit_image(&self.buffer, writer, ID(self.id)).await?;
            writer.flush().await?;
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        let action = ActionPut {
            x_offset: x,
            y_offset: y,
            move_cursor: false,
            placement: Placement(Some(self.id)),
            z_index: z,
            ..Default::default()
        };
        let cmd = Command {
            action: Action::Put(action),
            quietness: Quietness::SuppressAll,
            id: Some(ID(self.id)),
        };
        cmd.send(None, writer).await?;
        writer.flush().await
    }

    #[instrument(skip(self))]
    pub async fn delete_image(
        &self, writer: &mut TermWriter,
    ) -> anyhow::Result<()> {
        delete_image(writer, ID(self.id)).await?;
        {
            let mut transmitted = self.transmitted.lock();
            *transmitted = false;
        }
        writer.flush().await
    }
}

impl ImageSet {
    pub fn new(images: Vec<Vec<u8>>) -> Self {}

    pub async fn delte_images(
        &self, writer: &mut TermWriter,
    ) -> anyhow::Result<()> {
        for image in &self.images {
            image.delete_image(writer).await?;
        }
        Ok(())
    }

    pub async fn render_at(
        &mut self, writer: &mut TermWriter, x: u32, y: u32,
    ) -> anyhow::Result<()> {
        self.last_rendered_pos = Some((x, y));
        let image = &self.images[self.index];
        image.render_at(writer, x, y, self.last_zindex).await
    }

    pub async fn next_image(
        &mut self, writer: &mut TermWriter,
    ) -> anyhow::Result<()> {
        if self.last_rendered_pos.is_none() {
            bail!("Must render at least once then call next_image");
        }

        let (x, y) = self.last_rendered_pos.unwrap();
        self.index = (self.index + 1) % self.images.len();
        self.last_zindex += 1;

        self.images[self.index].render_at(writer, x, y, self.last_zindex);

        Ok(())
    }
}
