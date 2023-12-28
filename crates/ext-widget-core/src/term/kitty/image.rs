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
static IMAGE_SET_ID: AtomicU32 = AtomicU32::new(1);
pub static IMAGE_MANAGER: once_cell::sync::Lazy<Mutex<ImageManager>> =
    once_cell::sync::Lazy::new(|| Mutex::new(ImageManager::new()));

#[derive(Debug)]
pub struct ImageManager {
    image_sets: HashMap<NonZeroU32, Arc<ImageSet>>,
}

#[derive(Debug)]
pub struct Image {
    id: NonZeroU32,
    buffer: Vec<u8>,
    transmitted: Mutex<bool>,
}

#[derive(Debug)]
struct ImageSetDisplayState {
    index: usize,
    last_zindex: u32,
    last_rendered_pos: Option<(u32, u32)>,
}

#[derive(Debug)]
pub struct ImageSet {
    id: NonZeroU32,
    images: Vec<Arc<Image>>,
    state: Mutex<ImageSetDisplayState>,
}

impl ImageManager {
    fn new() -> Self {
        Self {
            image_sets: HashMap::new(),
        }
    }

    pub fn alloc_set_id() -> NonZeroU32 {
        let id: NonZeroU32 = IMAGE_SET_ID
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            .try_into()
            .unwrap();
        id
    }

    pub fn new_image_set(
        &mut self, images: Vec<Arc<Image>>,
    ) -> anyhow::Result<Arc<ImageSet>> {
        let image_set = ImageSet::new(images)?;
        let id = image_set.id;
        self.image_sets.insert(id, Arc::new(image_set));
        Ok(self.image_sets.get(&id).unwrap().clone())
    }

    pub fn new_image_set_with_id(
        &mut self, id: NonZeroU32, images: Vec<Arc<Image>>,
    ) -> anyhow::Result<Arc<ImageSet>> {
        let image_set = ImageSet::new_with_id(id, images)?;
        self.image_sets.insert(id, Arc::new(image_set));
        Ok(self.image_sets.get(&id).unwrap().clone())
    }

    pub fn find_image_set(&self, id: NonZeroU32) -> Option<Arc<ImageSet>> {
        self.image_sets.get(&id).cloned()
    }
}

impl Image {
    pub fn new_from_buffer_with_id(id: NonZeroU32, buffer: Vec<u8>) -> Self {
        Self {
            id,
            buffer,
            transmitted: Mutex::new(false),
        }
    }

    pub fn new_from_buffer(buffer: Vec<u8>) -> Self {
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
        &self, writer: &mut TermWriter, hard: bool,
    ) -> anyhow::Result<()> {
        delete_image(writer, ID(self.id), hard).await?;
        {
            let mut transmitted = self.transmitted.lock();
            *transmitted = false;
        }
        writer.flush().await
    }
}

impl ImageSetDisplayState {
    fn new() -> Self {
        Self {
            index: 0,
            last_zindex: 1,
            last_rendered_pos: None,
        }
    }
}

impl ImageSet {
    pub fn new(images: Vec<Arc<Image>>) -> anyhow::Result<Self> {
        if images.is_empty() {
            bail!("ImageSet must have at least one image");
        }
        Ok(Self {
            id: NonZeroU32::try_from(
                IMAGE_SET_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            )
            .unwrap(),
            images,
            state: Mutex::new(ImageSetDisplayState::new()),
        })
    }

    pub fn new_with_id(
        id: NonZeroU32, images: Vec<Arc<Image>>,
    ) -> anyhow::Result<Self> {
        if images.is_empty() {
            bail!("ImageSet must have at least one image");
        }
        Ok(Self {
            id,
            images,
            state: Mutex::new(ImageSetDisplayState::new()),
        })
    }

    pub async fn delete_image(
        &self, writer: &mut TermWriter, hard: bool,
    ) -> anyhow::Result<()> {
        for image in &self.images {
            image.delete_image(writer, hard).await?;
        }
        Ok(())
    }

    pub async fn render_at(
        &self, writer: &mut TermWriter, x: u32, y: u32,
    ) -> anyhow::Result<()> {
        let (image, z) = {
            let mut state = self.state.lock();
            state.last_rendered_pos = Some((x, y));
            let image = &self.images[state.index];
            (image, state.last_zindex)
        };
        image.render_at(writer, x, y, z).await
    }

    pub async fn next_image(
        &self, writer: &mut TermWriter,
    ) -> anyhow::Result<()> {
        let (previous, image, (x, y, z)) = {
            let mut state = self.state.lock();

            if state.last_rendered_pos.is_none() {
                bail!("Must render at least once then call next_image");
            }

            let (x, y) = state.last_rendered_pos.unwrap();
            let previous_index = state.index;
            state.index = (state.index + 1) % self.images.len();
            if previous_index == state.index {
                return Ok(());
            }
            state.last_zindex += 1;
            (
                &self.images[previous_index],
                &self.images[state.index],
                (x, y, state.last_zindex),
            )
        };

        image.render_at(writer, x, y, z).await?;
        previous.delete_image(writer, false).await?;

        Ok(())
    }

    pub async fn previous_image(
        &self, writer: &mut TermWriter,
    ) -> anyhow::Result<()> {
        let (previous, image, (x, y, z)) = {
            let mut state = self.state.lock();

            if state.last_rendered_pos.is_none() {
                bail!("Must render at least once then call next_image");
            }

            let (x, y) = state.last_rendered_pos.unwrap();
            let previous_index = state.index;
            state.index =
                (state.index + self.images.len() - 1) % self.images.len();
            if previous_index == state.index {
                return Ok(());
            }
            state.last_zindex += 1;
            (
                &self.images[previous_index],
                &self.images[state.index],
                (x, y, state.last_zindex),
            )
        };

        // render image first
        image.render_at(writer, x, y, z).await?;
        previous.delete_image(writer, false).await?;

        Ok(())
    }
}
