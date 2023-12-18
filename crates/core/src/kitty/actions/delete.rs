use std::fmt::{self, Display, Formatter};

use crate::kitty::common::Placement;

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub enum DeleteTarget {
    /// Deletes all placements visible on screen
    Placements,
    /// Deletes all imagees with the specified id.
    ID { placement: Placement },
    /// Deletes the newest image with the specified number
    Newest { number: u32, placement: Placement },
    /// Delete all placements that intersect with the current
    /// cursor position
    Cursor,
    /// Delete all animation frames
    Frames,
    /// Delete all placements that intersect a specific cell
    Cell { x: u32, y: u32 },
    /// Delete all placements that intersect a specific cell having a specifi z-index
    CellWithZIndex { x: u32, y: u32, z: u32 },
    /// Delete all placements that intersect a specific column
    Column(u32),
    /// Delete all placements that intersect a specific row
    Row(u32),
    /// Delete all placements that intersect a specific z-index
    ZIndex(u32),
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub struct ActionDelete {
    /// Whether to delete the storage data as well
    pub hard: bool,
    /// What to delete
    pub target: DeleteTarget,
}

impl Display for ActionDelete {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match (self.hard, &self.target) {
            (true, DeleteTarget::Placements) => write!(f, "d=A,"),
            (true, DeleteTarget::ID { placement }) => {
                write!(f, "d=I,")?;
                if let Some(placement) = placement.0 {
                    write!(f, "p={placement},")?;
                }
                Ok(())
            }
            (true, DeleteTarget::Newest { number, placement }) => {
                write!(f, "d=N,I={number},")?;
                if let Some(placement) = placement.0 {
                    write!(f, "p={placement},")?;
                }
                Ok(())
            }
            (true, DeleteTarget::Cursor) => write!(f, "d=C,"),
            (true, DeleteTarget::Frames) => write!(f, "d=D,"),
            (true, DeleteTarget::Cell { x, y }) => {
                write!(f, "d=P,x={x},y={y},")
            }
            (true, DeleteTarget::CellWithZIndex { x, y, z }) => {
                write!(f, "d=P,x={x},y={y},z={z},")
            }
            (true, DeleteTarget::Column(x)) => write!(f, "d=X,x={x}"),
            (true, DeleteTarget::Row(y)) => write!(f, "d=Y,y={y}"),
            (true, DeleteTarget::ZIndex(z)) => write!(f, "d=Z,z={z}"),
            (false, DeleteTarget::Placements) => write!(f, "d=a,"),
            (false, DeleteTarget::ID { placement }) => {
                write!(f, "d=i,")?;
                if let Some(placement) = placement.0 {
                    write!(f, "p={placement},")?;
                }
                Ok(())
            }
            (false, DeleteTarget::Newest { number, placement }) => {
                write!(f, "d=n,I={number},")?;
                if let Some(placement) = placement.0 {
                    write!(f, "p={placement},")?;
                }
                Ok(())
            }
            (false, DeleteTarget::Cursor) => write!(f, "d=c,"),
            (false, DeleteTarget::Frames) => write!(f, "d=d,"),
            (false, DeleteTarget::Cell { x, y }) => {
                write!(f, "d=p,x={x},y={y},")
            }
            (false, DeleteTarget::CellWithZIndex { x, y, z }) => {
                write!(f, "d=p,x={x},y={y},z={z},")
            }
            (false, DeleteTarget::Column(x)) => write!(f, "d=x,x={x}"),
            (false, DeleteTarget::Row(y)) => write!(f, "d=y,y={y}"),
            (false, DeleteTarget::ZIndex(z)) => write!(f, "d=z,z={z}"),
        }
    }
}
