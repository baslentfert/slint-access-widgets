//! A folder tree (see ui/app.slint). The contents of a folder are "loaded"
//! when it is first opened, as if from a disk or another computer.

use std::rc::Rc;

use slint_access_widgets::{TreeRowInfo, TreeRows};

slint::include_modules!();

/// The pretend disk: children of a folder, by its path.
fn listing(path: &str) -> Vec<(String, bool)> {
    let d = |n: &str| (n.to_string(), true);
    let f = |n: &str| (n.to_string(), false);
    match path {
        "Documents" => vec![d("Projects"), d("Finance"), f("notes.txt")],
        "Documents/Projects" => vec![d("2025"), d("2026"), f("ideas.md")],
        "Documents/Projects/2026" => vec![f("plan.pdf"), f("budget.xlsx"), f("timeline.docx")],
        "Documents/Projects/2025" => vec![f("report.pdf")],
        "Documents/Finance" => vec![f("invoice-01.pdf"), f("invoice-02.pdf")],
        "Pictures" => vec![d("Holiday"), f("profile.jpg")],
        "Pictures/Holiday" => vec![f("beach.jpg"), f("mountain.jpg"), f("sunset.jpg")],
        _ => vec![],
    }
}

fn main() {
    let w = AppWindow::new().unwrap();
    // The full path of a node, from its parents.
    let tree: Rc<TreeRows<AccessTreeRow>> = Rc::new_cyclic(|weak: &std::rc::Weak<TreeRows<AccessTreeRow>>| {
        let weak = weak.clone();
        TreeRows::new(
            |r: &TreeRowInfo| AccessTreeRow {
                text: r.text.clone().into(),
                level: r.level,
                has_children: r.has_children,
                expanded: r.expanded,
                // A folder stays a folder when it turns out empty.
                icon: if r.branch { AccessTreeIcon::Folder } else { AccessTreeIcon::File },
                ..Default::default()
            },
            move |node| {
                let Some(t) = weak.upgrade() else { return Vec::new() };
                let mut parts = vec![t.text_of(node).unwrap_or_default()];
                let mut n = node;
                while let Some(p) = t.parent_node(n) {
                    parts.push(t.text_of(p).unwrap_or_default());
                    n = p;
                }
                parts.reverse();
                listing(&parts.join("/"))
            },
        )
    });
    tree.set_roots(vec![("Documents".into(), true), ("Pictures".into(), true), ("Downloads".into(), true)]);
    w.set_rows(tree.model());
    let t = tree.clone();
    w.on_expand(move |row| t.expand(row as usize) as i32);
    let t = tree.clone();
    w.on_collapse(move |row| t.collapse(row as usize));
    let t = tree.clone();
    w.on_parent_of(move |row| t.parent_of(row as usize));
    let t = tree.clone();
    w.on_find(move |p, from| slint_access_widgets::find_prefix_by(t.len(), &p, from, |i| t.text_at(i)));
    let (t, ww) = (tree.clone(), w.as_weak());
    w.on_activated(move |row| {
        if let Some(text) = t.text_at(row as usize) {
            ww.unwrap().set_status(format!("Opened {text}").into());
        }
    });
    w.run().unwrap();
}
