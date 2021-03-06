use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
    io::{stdin, BufRead, BufReader},
    rc::Rc,
};

use anyhow::Result;
use clap::Clap;
use cursive::{
    event::Key,
    traits::*,
    views::{Dialog, ScrollView},
    Cursive, CursiveExt,
};
use cursive_tree_view::{Placement, TreeView};
use indextree::{Arena, NodeId};
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug, Clap)]
#[clap(name = "tree2tui", about, version)]
struct Opt {
    /// enable folding/expanding duplicate nodes of "cargo tree"
    #[clap(short, long)]
    cargo: bool,

    /// regex to capture the tree's node
    #[clap(short, long, default_value = r"[│\s]*([├└]─*\s*)")]
    regex: String,

    /// group selected from regex as tree's node
    #[clap(short, long, default_value = "1")]
    node: usize,

    /// group selected from regex as tree's data, if not set using all data after node
    #[clap(short, long)]
    data: Option<usize>,

    #[clap(short = 'h', long)]
    skip_head: bool,

    #[clap(short = 's', long, default_value = "0")]
    skip_lines: usize,
}

static OPT: Lazy<Opt> = Lazy::new(Opt::parse);

fn main() -> Result<()> {
    Lazy::force(&OPT);

    let (arena, root) = parse_tree(BufReader::new(stdin()))?;

    #[derive(Debug)]
    struct TreeEntry {
        node: NodeId,
        arena: Rc<Arena<String>>,
    }

    impl TreeEntry {
        fn as_str(&self) -> &str {
            self.arena.get(self.node).unwrap().get()
        }
    }

    impl std::fmt::Display for TreeEntry {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.write_str(self.as_str())
        }
    }

    fn expand_tree(tree: &mut TreeView<TreeEntry>, row: usize) {
        let e = tree.borrow_item(row).unwrap();
        let v: Vec<_> = e
            .node
            .children(&e.arena)
            .map(|i| TreeEntry {
                node: i,
                arena: e.arena.clone(),
            })
            .collect();
        for i in v {
            if i.node.children(&i.arena).next().is_some() {
                tree.insert_container_item(i, Placement::LastChild, row);
            } else {
                if OPT.cargo {
                    i.as_str()
                        .strip_suffix(" (*)")
                        .and_then(|d| i.arena.iter().find(|n| n.get() == d))
                        .and_then(|n| i.arena.get_node_id(n))
                        .and_then(|node| {
                            tree.insert_container_item(
                                TreeEntry {
                                    node,
                                    arena: i.arena.clone(),
                                },
                                Placement::LastChild,
                                row,
                            )
                        });
                }

                tree.insert_item(i, Placement::LastChild, row);
            }
        }
    }

    let mut tree = TreeView::<TreeEntry>::new();

    tree.insert_item(
        TreeEntry {
            node: root,
            arena: Rc::new(arena),
        },
        Placement::After,
        0,
    );

    expand_tree(&mut tree, 0);

    const TREE_NAME: &str = "tree";

    tree.set_on_collapse(|siv: &mut Cursive, row, is_collapsed: bool, children| {
        if !is_collapsed && children == 0 {
            siv.call_on_name(TREE_NAME, |tree: &mut TreeView<TreeEntry>| {
                expand_tree(tree, row);
            });
        }
    });

    let mut siv = Cursive::default();
    siv.add_layer(Dialog::around(ScrollView::new(tree.with_name(TREE_NAME))));

    siv.add_global_callback('q', Cursive::quit);

    siv.add_global_callback(Key::Left, |siv| {
        siv.call_on_name(TREE_NAME, |tree: &mut TreeView<TreeEntry>| {
            if let Some(parent) = tree.row().and_then(|row| tree.item_parent(row)) {
                tree.set_selected_row(parent);
            }
        });
    });

    siv.run();

    Ok(())
}

pub fn parse_tree(buf: impl BufRead) -> Result<(Arena<String>, NodeId)> {
    let mut arena: Arena<String> = Arena::new();
    let mut nodes = HashMap::new();

    let mut root = None;
    let mut root_data: Option<String> = None;

    for (line_idx, line) in buf.lines().enumerate().skip(OPT.skip_lines) {
        let line = &line?;

        let line = &console::strip_ansi_codes(line);

        if let Some(n) = parse_node(line) {
            let current = arena.new_node(n.data.into());

            if nodes.is_empty() {
                let r = arena.new_node(root_data.take().unwrap_or_else(|| "<...>".to_string()));
                r.append(current, &mut arena);

                nodes.insert(n.node_pos, r);
                root = Some(r);
            } else if let Some(p) = nodes.get(&n.node_pos) {
                p.append(current, &mut arena);
            } else {
                anyhow::bail!("dangling node at line {}:{}", line_idx, line)
            }

            nodes.insert(n.data_pos, current);
        } else if !OPT.skip_head && nodes.is_empty() {
            root_data = Some(line.trim().to_string());
        }
    }

    Ok((arena, root.ok_or_else(|| anyhow::anyhow!("empty tree"))?))
}

#[derive(Debug)]
struct NodeInfo<'t> {
    node_pos: usize,
    data_pos: usize,
    data: &'t str,
}

fn chars_count(line: &str, idx: usize) -> usize {
    line[..idx].chars().count()
}

fn parse_node(line: &str) -> Option<NodeInfo> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(&OPT.regex).unwrap());
    let cap = RE.captures(line)?;
    let node = cap.get(OPT.node)?;

    let (data, pos) = OPT
        .data
        .and_then(|d| cap.get(d))
        .map(|d| (d.as_str(), d.start()))
        .unwrap_or_else(|| (&line[node.end()..], node.end()));

    Some(NodeInfo {
        data,
        node_pos: chars_count(line, node.start()),
        data_pos: chars_count(line, pos),
    })
}
