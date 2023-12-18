use anyhow::Context;
use std::collections::HashMap;
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct MarkupProperties(HashMap<String, String>);

#[derive(Debug, Clone)]
pub struct MarkupSpan {
    properties: MarkupProperties,
}

#[derive(Debug, Clone)]
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

    pub fn merge(mut self, other: Self) -> Self {
        self.0.extend(other.0);
        self
    }
}

impl Default for MarkupProperties {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MarkupSpan {
    fn default() -> Self {
        Self::new()
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

    pub fn wrap_text_owned(
        &self, text: impl AsRef<str>,
    ) -> anyhow::Result<String> {
        let mut res = String::new();
        self.wrap_text(text, &mut res)?;
        Ok(res)
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

    pub fn to_markup_open_owned(&self) -> anyhow::Result<String> {
        let mut res = String::new();
        self.to_markup_open(&mut res)?;
        Ok(res)
    }

    pub fn to_markup_close(&self, res: &mut String) {
        self.stack
            .iter()
            .rev()
            .for_each(|span| span.to_markup_close(res))
    }

    pub fn wrap_text_owned(
        &self, text: impl AsRef<str>,
    ) -> anyhow::Result<String> {
        let mut res = String::new();
        self.to_markup_open(&mut res)?;
        res.push_str(text.as_ref());
        self.to_markup_close(&mut res);
        Ok(res)
    }
}

impl Default for MarkupSpanStack {
    fn default() -> Self {
        Self::new()
    }
}
