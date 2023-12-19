use anyhow::Context;
use external_widget_core::treesitter::TREE_SITTER;
use tree_sitter::{QueryCursor, TextProvider};

pub struct MdClodeBlock {
    code: String,
    lang: String,
}

impl MdClodeBlock {
    pub fn new(code: String, lang: String) -> Self {
        Self { code, lang }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(super) enum HighlightMarkerType {
    Start,
    End,
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct HighlightMarker {
    pub group: String,
    pub offset: usize,
    pub kind: HighlightMarkerType,
}

impl PartialOrd for HighlightMarker {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.offset != other.offset {
            self.offset.partial_cmp(&other.offset)
        } else if self.kind == other.kind {
            match self.kind {
                HighlightMarkerType::Start => {
                    self.group.partial_cmp(&other.group)
                }
                HighlightMarkerType::End => {
                    other.group.partial_cmp(&self.group)
                }
            }
        } else {
            Some(match self.kind {
                HighlightMarkerType::Start => std::cmp::Ordering::Greater,
                HighlightMarkerType::End => std::cmp::Ordering::Less,
            })
        }
    }
}

impl Ord for HighlightMarker {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.offset != other.offset {
            self.offset.cmp(&other.offset)
        } else if self.kind == other.kind {
            match self.kind {
                HighlightMarkerType::Start => self.group.cmp(&other.group),
                HighlightMarkerType::End => other.group.cmp(&self.group),
            }
        } else {
            match self.kind {
                HighlightMarkerType::Start => std::cmp::Ordering::Greater,
                HighlightMarkerType::End => std::cmp::Ordering::Less,
            }
        }
    }
}

pub(super) fn get_all_captures(
    code: &str, lang: &str,
) -> anyhow::Result<Vec<HighlightMarker>> {
    let mut parser = TREE_SITTER.lock().unwrap().get_parser(lang)?;
    let tree = parser.parse(code, None).context("Parse tree failed")?;
    let query = TREE_SITTER.lock().unwrap().get_query(lang, "highlights")?;
    let mut cursor = QueryCursor::new();
    let all_captures =
        cursor.captures(&query, tree.root_node(), code.as_bytes());

    let mut ret = vec![];
    for (m, _) in all_captures {
        for (_, capture) in m.captures.iter().enumerate() {
            let start_byte = capture.node.start_byte();
            let end_byte = capture.node.end_byte();
            if start_byte < end_byte {
                ret.push(HighlightMarker {
                    group: query.capture_names()[capture.index as usize]
                        .to_string(),
                    kind: HighlightMarkerType::Start,
                    offset: start_byte,
                });
                ret.push(HighlightMarker {
                    group: query.capture_names()[capture.index as usize]
                        .to_string(),
                    offset: end_byte,
                    kind: HighlightMarkerType::End,
                })
            }
        }
    }
    Ok(ret)
}