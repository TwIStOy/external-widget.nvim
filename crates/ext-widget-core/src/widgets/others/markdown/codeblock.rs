use anyhow::Context;
use futures::AsyncWrite;
use nvim_rs::Neovim;

use crate::nvim::NeovimSession;

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
        Some(self.cmp(other))
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

pub(super) async fn get_all_captures<W>(
    nvim: &Neovim<W>, session: &NeovimSession, code: &str, lang: &str,
) -> anyhow::Result<Vec<HighlightMarker>>
where
    W: AsyncWrite + Send + Unpin + 'static,
{
    let mut parser = session
        .load_ts_parser(nvim, lang)
        .await?
        .context("Parser?")?;
    let query = session.load_ts_query(nvim, lang, "highlights").await?;
    let tree = parser.parse(code, None).context("Parse tree failed")?;
    let mut cursor = tree_sitter::QueryCursor::new();

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
