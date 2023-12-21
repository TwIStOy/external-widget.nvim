use super::WidgetKey;

pub struct BuildContext {}

impl BuildContext {
    pub fn is_dirty(&self, key: WidgetKey) -> bool {
        false
    }
}
