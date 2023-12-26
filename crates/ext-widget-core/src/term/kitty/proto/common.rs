use std::{
    fmt::{self, Display, Formatter},
    num::NonZeroU32,
};

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Default, Debug)]
pub struct Placement(pub Option<NonZeroU32>);

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub struct ID(pub NonZeroU32);

impl Display for ID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.get())
    }
}

/// The medium to use
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Default, Debug)]
pub enum Medium {
    /// The file is stored in the escape code itsel
    #[default]
    Direct,
    /// A simple file (regular files only, not named pipes, device files, &c)
    ///
    /// The filepath should be stored in the data section
    File,
    /// A temporary file
    ///
    /// The terminal emulator will delete the file after reading the pixel data.
    /// For security reasons, the terminal emulator should only delete the file if
    /// in a known temporary directory, such as `/tmp`, `/dev/shm`, `TMPDIR env if present`
    /// and any platform specific temporary directories and the file has the string `tty-graphics-protocol`
    /// in its full path
    TemporaryFile,
    /// A shared memory object, which on POSIX systems is a POSIX shared memory object,
    /// and on Windows is a Named shared memory object. The terminal emulator must read
    /// the data from the memory object and then unlink and close it on POSIX and just
    /// close it on windows.
    SharedMemoryObject,
}

impl Display for Medium {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Medium::Direct => "d",
            Medium::File => "f",
            Medium::TemporaryFile => "t",
            Medium::SharedMemoryObject => "s",
        })
    }
}

/// The format of the data
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Default, Debug)]
pub enum Format {
    /// 24 bit rgb
    Rgb24,
    /// 32 bit rgba
    #[default]
    Rgba32,
    /// Png
    Png,
}

impl Display for Format {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Format::Rgb24 => "24",
            Format::Rgba32 => "32",
            Format::Png => "100",
        })
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum CompositionMode {
    /// Blend between pixels
    #[default]
    AlphaBlend,
    /// Overwite pixels
    Overwrite,
}

impl Display for CompositionMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            CompositionMode::AlphaBlend => "0",
            CompositionMode::Overwrite => "1",
        })
    }
}

/// A frame
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub struct Frame(pub NonZeroU32);

impl Display for Frame {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// How to loop an animation
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub enum LoopMode {
    /// Loop forever
    Infinite,
    /// Loop a finite amount of times
    Finite(NonZeroU32),
}

impl Display for LoopMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Infinite => write!(f, "1"),
            Self::Finite(x) => write!(f, "{}", x.get() - 1),
        }
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Default, Debug)]
pub enum AnimationMode {
    /// Stop the animation
    #[default]
    Stop,
    /// Run the animation, but wait for new frames
    RunWithNewFrames,
    /// Run the animation
    Run,
}

impl Display for AnimationMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            AnimationMode::Stop => "1",
            AnimationMode::RunWithNewFrames => "2",
            AnimationMode::Run => "3",
        })
    }
}

/// The quiteness of the operation
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Default, Debug)]
pub enum Quietness {
    /// Full volume
    #[default]
    None,
    /// Supresses 'ok' messages
    SupressOk,
    /// Suppressed 'ok' and error messagsde
    SuppressAll,
}

impl Display for Quietness {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Quietness::None => Ok(()),
            Quietness::SupressOk => write!(f, "q=1,"),
            Quietness::SuppressAll => write!(f, "q=2,"),
        }
    }
}
