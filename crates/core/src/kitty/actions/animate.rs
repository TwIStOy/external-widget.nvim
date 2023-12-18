use std::fmt::{self, Display, Formatter};

use crate::kitty::common::{AnimationMode, CompositionMode, Frame, LoopMode};

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Default, Debug)]
pub struct ActionAnimationFrameLoading {
    /// The left edge (in pixels) of the image area to display
    pub x: u32,
    /// The top edge (in pixels) of the image area to display
    pub y: u32,
    /// The 1-based frame number of the frame whose image data
    /// serves as the base data when creating a new frame, by
    /// default the base data is black, fully transparent pixels
    pub frame_number: Option<Frame>,
    /// The 1-based frame number of the frame that is being edited.
    /// By default, a new frame is created
    pub frame_edited: Option<Frame>,
    /// The gap (in milliseconds) of this frame from the next one.
    /// A value of zero is ignored. Negative values create
    /// a *gapless* frame. If not specified, frames have a default
    /// gap of `40ms`. The root frame defaults to zero gap
    pub gap: i32,
    /// The composition mode for blending pixels when creating a new frame or
    /// editing a frame's data. The default is full alpha blending.
    pub composition_mode: CompositionMode,
    /// The background color for pixels not specified in the frame data.
    /// In RGBA
    pub color: u32,
}

impl Display for ActionAnimationFrameLoading {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.x != 0 {
            write!(f, "x={},", self.x)?;
        }
        if self.y != 0 {
            write!(f, "y={},", self.y)?;
        }
        if let Some(frame_number) = self.frame_number {
            write!(f, "c={frame_number},")?;
        }
        if let Some(frame_edited) = self.frame_edited {
            write!(f, "r={frame_edited },")?;
        }
        if self.gap != 0 {
            write!(f, "z={},", self.gap)?;
        }
        if self.composition_mode != CompositionMode::default() {
            write!(f, "X={},", self.gap)?;
        }
        if self.color != 0 {
            write!(f, "Y={},", self.color)?;
        }

        Ok(())
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Default, Debug)]
pub struct ActionAnimationFrameComposition {
    /// The 1-based frame number of the frame whose image data
    /// serves as the overlaid data
    pub frame_number: Option<Frame>,
    /// The 1-based frame number of the frame that is being edited.
    /// By default, a new frame is created
    pub frame_edited: Option<Frame>,
    /// The left edge (in pixels) of the image area to display
    pub x: u32,
    /// The top edge (in pixels) of the image area to display
    pub y: u32,
    /// The width (in pixels) of the image area to display.
    /// By default, the entire width is used.
    pub w: u32,
    /// The height (in pixels) of the image area to display.
    /// By default, the entire height is used.
    pub h: u32,
    /// The left edge (in pixels) of the source rectangle
    pub source_x: u32,
    /// The top edge (in pixels) of the source rectangle
    pub source_y: u32,
    /// The composition mode for blending pixels when creating a new frame or
    /// editing a frame's data. The default is full alpha blending.
    pub composition_mode: CompositionMode,
}

impl Display for ActionAnimationFrameComposition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(frame_number) = self.frame_number {
            write!(f, "c={frame_number},")?;
        }
        if let Some(frame_edited) = self.frame_edited {
            write!(f, "c={frame_edited},")?;
        }

        if self.x != 0 {
            write!(f, "x={},", self.x)?;
        }
        if self.y != 0 {
            write!(f, "y={},", self.y)?;
        }
        if self.w != 0 {
            write!(f, "w={},", self.w)?;
        }
        if self.h != 0 {
            write!(f, "h={},", self.h)?;
        }

        if self.source_x != 0 {
            write!(f, "X={},", self.source_x)?;
        }
        if self.source_y != 0 {
            write!(f, "Y={},", self.source_y)?;
        }
        if self.composition_mode != CompositionMode::default() {
            write!(f, "C={},", self.composition_mode)?;
        }

        Ok(())
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Default, Debug)]
pub struct ActionAnimationFrameControl {
    /// The mode of this command
    pub mode: AnimationMode,
    /// The 1-based frame number of the frame that is being affected
    pub frame_number: Option<Frame>,
    /// The gap (in milliseconds) of this frame from the next one.
    /// A value of zero is ignored. Negative values create a gapless frame.
    pub gap: i32,
    /// The 1-based frame number of the frame that should be made
    /// the current frame
    pub frame: Option<Frame>,
    /// The loop mode
    pub loop_mode: Option<LoopMode>,
}

impl Display for ActionAnimationFrameControl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "s={},", self.mode)?;

        if let Some(frame_number) = self.frame_number {
            write!(f, "r={frame_number},")?;
        }
        write!(f, "z={},", self.gap)?;
        if let Some(frame) = self.frame {
            write!(f, "c={},", frame)?;
        }
        if let Some(loop_mode) = self.loop_mode {
            write!(f, "v={},", loop_mode)?;
        }

        Ok(())
    }
}
