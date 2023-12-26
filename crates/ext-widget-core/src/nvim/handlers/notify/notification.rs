use std::sync::atomic::AtomicUsize;

static NOTIFICATION_ID: AtomicUsize = AtomicUsize::new(0);

pub struct NotificationID(usize);

pub enum AnimationStyle {
    FadeInSlideOut,
    Fade,
    Slide,
    Static,
}

pub trait Notification {
    /// Get the unique id of the notification.
    fn id(&self) -> NotificationID;

    // fn should_update(&self, now: std::time::Instant) -> bool;
}

// struct Notification {
//     id: NotificationID,
//     title: String,
//     message: String,
//     timeout: Option<u64>,
//     level: Option<String>,

//     last_update: Option<std::time::Instant>,
// }

// impl Notification {
//     pub fn render(&mut self) {
//         todo!()
//     }
// }
