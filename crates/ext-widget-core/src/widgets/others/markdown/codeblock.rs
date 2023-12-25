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
