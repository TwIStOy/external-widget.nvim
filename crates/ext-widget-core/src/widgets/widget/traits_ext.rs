use super::Widget;

const TREE_MARKER_LAST: &str = "└── ";
const TREE_MARKER_MIDDLE: &str = "├── ";
const TREE_MARKER_VERTICAL: &str = "│   ";

fn append_marker(lines: &mut Vec<String>, lasts: &[bool]) {
    if lines.is_empty() {
        lines.push(String::new());
    }

    for (i, last) in lasts.iter().enumerate() {
        if i == lasts.len() - 1 {
            if *last {
                lines.last_mut().unwrap().push_str(TREE_MARKER_LAST);
            } else {
                lines.last_mut().unwrap().push_str(TREE_MARKER_MIDDLE);
            }
        } else if *last {
            lines.last_mut().unwrap().push_str(&" ".repeat(4));
        } else {
            lines.last_mut().unwrap().push_str(TREE_MARKER_VERTICAL);
        }
    }
}

pub(crate) trait WidgetExt: Widget {
    fn debug_tree(&self, lasts: &mut Vec<bool>, lines: &mut Vec<String>) {
        append_marker(lines, lasts);
        lines.last_mut().unwrap().push_str(self.type_name());
        lines.push(String::new());
        for (i, child) in self.children().iter().enumerate() {
            let last = i == self.children().len() - 1;
            lasts.push(last);
            child.debug_tree(lasts, lines);
            lasts.pop();
        }
    }
}

impl<T: ?Sized + Widget> WidgetExt for T {}
