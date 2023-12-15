use std::collections::HashMap;

pub struct MarkupProperties(HashMap<String, String>);

impl MarkupProperties {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.0.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|s| s.as_str())
    }

    pub fn to_markup(&self, res: &mut String) {
    }
}

pub struct MarkupSpan {
    pub properties: MarkupProperties,
}

impl MarkupSpan {
    pub fn new() -> Self {
        Self {
            properties: MarkupProperties::new(),
        }
    }

    pub fn new_with_properties(properties: MarkupProperties) -> Self {
        Self { properties }
    }

}
