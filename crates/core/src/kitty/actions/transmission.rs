use std::fmt::{Display, Formatter};

use crate::kitty::common::{Format, Medium, Placement};

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Default, Debug)]
pub struct ActionTransmission {
    /// The format in which the image data is sent
    pub format: Format,
    /// The transmission medium used
    pub medium: Medium,
    /// The width of the image being sent
    pub width: u32,
    /// The height of the image being sent
    pub height: u32,
    /// The size of data to read from a file (if applicable)
    pub size: u32,
    /// The offset from which to read data from a file (if applicable)
    pub offset: u32,
    /// The image number
    pub number: u32,
    /// The placement id
    pub placement: Placement,
    /// Whether the data is in zlib compression
    pub compression: bool,
}

impl Display for ActionTransmission {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "f={},", self.format)?;
        write!(f, "t={},", self.medium)?;
        write!(f, "s={},", self.width)?;
        write!(f, "v={},", self.height)?;
        write!(f, "S={},", self.size)?;
        write!(f, "O={},", self.offset)?;
        write!(f, "I={},", self.number)?;
        if let Some(placement) = self.placement.0 {
            write!(f, "p={placement},")?;
        }
        if self.compression {
            write!(f, "o=z,")?;
        }

        Ok(())
    }
}
