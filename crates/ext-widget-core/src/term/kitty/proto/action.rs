use std::fmt::{self, Display, Formatter};

use super::actions::*;

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub enum Action {
    /// Transmit image data
    Transmit(ActionTransmission),
    /// Transmist image data and display it
    TransmitAndDisplay(ActionTransmission, ActionPut),
    /// Make a query
    Query,
    /// Display image data
    Put(ActionPut),
    /// Load animation frames
    AnimationFrameLoading(ActionAnimationFrameLoading),
    /// Compose animation frames
    AnimationFrameComposition(ActionAnimationFrameComposition),
    /// Control animation frames
    AnimationFrameControl(ActionAnimationFrameControl),
    /// Delete
    Delete(ActionDelete),
}

impl Action {
    pub fn char(&self) -> char {
        match self {
            Action::Transmit(_) => 't',
            Action::TransmitAndDisplay(_, _) => 'T',
            Action::Query => 'q',
            Action::Put(_) => 'p',
            Action::AnimationFrameLoading(_) => 'f',
            Action::AnimationFrameComposition(_) => 'c',
            Action::AnimationFrameControl(_) => 'a',
            Action::Delete(_) => 'd',
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Action::Transmit(x) => write!(f, "a=t,{x}"),
            Action::TransmitAndDisplay(t, p) => write!(f, "a=T,{t}{p}"),
            Action::Query => write!(f, "a=q,"),
            Action::Put(x) => write!(f, "a=p,{x}"),
            Action::AnimationFrameLoading(x) => write!(f, "a=f,{x}"),
            Action::AnimationFrameComposition(x) => write!(f, "a=c,{x}"),
            Action::AnimationFrameControl(x) => write!(f, "a=a,{x}"),
            Action::Delete(x) => write!(f, "a=d,{x}"),
        }
    }
}
