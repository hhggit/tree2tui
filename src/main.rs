use std::{
    collections::HashMap,
    io::{stdin, BufRead, BufReader},
};

use anyhow::Result;
use indextree::{Arena, NodeId};
use once_cell::sync::Lazy;
use regex::Regex;

fn main() -> Result<()> {
    let (arena, root) = parse_tree(BufReader::new(stdin()))?;

    Ok(())
}

pub fn parse_tree(buf: impl BufRead) -> Result<(Arena<String>, NodeId)> {
    let mut arena: Arena<String> = Arena::new();
    let mut nodes = HashMap::new();

    const ROOT: usize = 0;

    for (line_idx, line) in buf.lines().enumerate() {
        let line = &line?;

        if nodes.is_empty() {
            nodes.insert(ROOT, arena.new_node(line.to_string()));
            continue;
        }
        if let Some(n) = parse_node(line) {
            let current = arena.new_node(n.data.into());

            nodes.insert(n.data_pos, current);

            nodes
                .get(&n.node_pos)
                .ok_or_else(|| anyhow::anyhow!("parse error at line {}:{}", line_idx, line))?
                .append(current, &mut arena);
        }
    }

    Ok((
        arena,
        *nodes.get(&ROOT).ok_or_else(|| anyhow::anyhow!("NO ROOT"))?,
    ))
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
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[│\s]*([├└]─*\s*)").unwrap());
    let cap = RE.captures(line)?;
    let node = cap.get(1)?;
    Some(NodeInfo {
        data: &line[node.end()..],
        node_pos: chars_count(line, node.start()),
        data_pos: chars_count(line, node.end()),
    })
}
