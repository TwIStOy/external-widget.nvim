use std::fmt::{self, Display, Formatter};

use crate::kitty::common::{Placement, ID};

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Default, Debug)]
pub struct ActionPut {
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
    /// The x-offset within the first cell at which to start
    /// displaying the image
    pub x_offset: u32,
    /// The y-offset within the first cell at which to start
    /// displaying the image
    pub y_offset: u32,
    //// The number of columns to display the image over
    pub columns: u32,
    //// The number of rows to display the image over
    pub rows: u32,
    /// Whether the cursor should be moved after
    /// printing the image
    pub move_cursor: bool,
    /// Whether this is a unicode placeholder. The
    /// cursor will not be moved if it is
    pub unicode_placeholder: bool,
    /// The z-index vertical staking order of the image
    pub z_index: u32,
    /// The placement id
    pub placement: Placement,
    /// The ID of a parent image for relative placement
    pub parent_image: Option<ID>,
    /// The id of a placement in the parent image for relative placement
    pub parent_placement: Placement,
    /// The offset in cells in the horizontal direction for
    /// relative placement
    pub cell_relative_offset_horizontal: u32,
    /// The offset in cells in the vertical direction for
    /// relative placement
    pub cell_relative_offset_vertical: u32,
}

impl Display for ActionPut {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
        if self.x_offset != 0 {
            write!(f, "X={},", self.x_offset)?;
        }
        if self.y_offset != 0 {
            write!(f, "Y={},", self.y_offset)?;
        }
        if self.columns != 0 {
            write!(f, "c={},", self.columns)?;
        }
        if self.rows != 0 {
            write!(f, "r={},", self.rows)?;
        }
        if !self.move_cursor {
            write!(f, "C=1,")?;
        }
        if self.unicode_placeholder {
            write!(f, "U=1,")?;
        }
        if self.z_index != 0 {
            write!(f, "z={},", self.z_index)?;
        }
        if let Some(parent) = self.parent_image {
            write!(f, "P={parent},")?;
        }
        if let Some(parent_placement) = self.parent_placement.0 {
            write!(f, "Q={parent_placement},")?;
        }
        if self.cell_relative_offset_horizontal != 0 {
            write!(f, "H={},", self.cell_relative_offset_horizontal)?;
        }
        if self.cell_relative_offset_vertical != 0 {
            write!(f, "V={},", self.cell_relative_offset_vertical)?;
        }
        if let Some(placement) = self.placement.0 {
            write!(f, "p={placement},")?;
        }

        Ok(())
    }
}
