use anyhow::Context;
use std::collections::HashMap;
use std::fmt::Write;

pub struct MarkupProperties(HashMap<String, String>);

pub struct MarkupSpan {
    pub properties: MarkupProperties,
}

pub struct MarkupSpanStack {
    stack: Vec<MarkupSpan>,
}

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

    pub fn to_markup(&self, res: &mut String) -> anyhow::Result<()> {
        self.0
            .iter()
            .try_for_each(|(k, v)| write!(res, "{}=\"{}\"", k, v))
            .context("Failed to write markup")
    }
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

    /// Append the opening span tag to the string
    ///
    /// @param res the string to append to
    pub fn to_markup_open(&self, res: &mut String) -> anyhow::Result<()> {
        res.push_str("<span ");
        self.properties.to_markup(res)?;
        res.push('>');
        Ok(())
    }

    pub fn to_markup_close(&self, res: &mut String) {
        res.push_str("</span>");
    }

    /// Wrap the given text in the span tag, them append it to the string
    pub fn wrap_text(
        &self, text: impl AsRef<str>, res: &mut String,
    ) -> anyhow::Result<()> {
        self.to_markup_open(res)?;
        res.push_str(text.as_ref());
        self.to_markup_close(res);
        Ok(())
    }
}

impl MarkupSpanStack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn push(&mut self, span: MarkupSpan) {
        self.stack.push(span);
    }

    pub fn pop(&mut self) -> Option<MarkupSpan> {
        self.stack.pop()
    }

    pub fn to_markup_open(&self, res: &mut String) -> anyhow::Result<()> {
        self.stack
            .iter()
            .try_for_each(|span| span.to_markup_open(res))
            .context("Failed to write markup")
    }

    pub fn to_markup_close(&self, res: &mut String) {
        self.stack
            .iter()
            .rev()
            .for_each(|span| span.to_markup_close(res))
    }
}
