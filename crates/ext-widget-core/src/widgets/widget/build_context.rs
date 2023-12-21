use super::WidgetKey;

pub struct BuildContext {}

impl BuildContext {
    pub fn is_dirty(&self, _key: WidgetKey) -> bool {
        false
    }
}
