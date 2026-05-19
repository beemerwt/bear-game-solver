use std::{fs, path::Path};

use crate::{game::GameState, graph::Graph};

pub fn render_svg(
    graph: &Graph,
    state: Option<&GameState>,
    out_path: &Path,
) -> std::io::Result<()> {
    let min_x = graph.nodes.values().map(|n| n.x).min().unwrap_or(0);
    let max_x = graph.nodes.values().map(|n| n.x).max().unwrap_or(0);
    let min_y = graph.nodes.values().map(|n| n.y).min().unwrap_or(0);
    let max_y = graph.nodes.values().map(|n| n.y).max().unwrap_or(0);

    let scale = 80;
    let pad = 80;
    let width = ((max_x - min_x + 2) * scale + pad * 2) as usize;
    let height = ((max_y - min_y + 2) * scale + pad * 2) as usize;

    let to_px = |x: i32, y: i32| ((x - min_x + 1) * scale + pad, (y - min_y + 1) * scale + pad);

    let mut svg =
        format!("<svg xmlns='http://www.w3.org/2000/svg' width='{width}' height='{height}'>\n");
    svg.push_str("<rect width='100%' height='100%' fill='white'/>\n");

    for (&from, tos) in &graph.adjacency {
        let from_node = &graph.nodes[&from];
        let (x1, y1) = to_px(from_node.x, from_node.y);
        for &to in tos {
            if from < to {
                let to_node = &graph.nodes[&to];
                let (x2, y2) = to_px(to_node.x, to_node.y);
                svg.push_str(&format!("<line x1='{x1}' y1='{y1}' x2='{x2}' y2='{y2}' stroke='#777' stroke-width='2'/>\n"));
            }
        }
    }

    for node in graph.nodes.values() {
        let (x, y) = to_px(node.x, node.y);
        let fill = match state {
            Some(s) if s.bear == node.id => "#d9534f",
            Some(s) if s.hunters.contains(&node.id) => "#0275d8",
            _ => "#f5f5f5",
        };
        svg.push_str(&format!(
            "<circle cx='{x}' cy='{y}' r='16' fill='{fill}' stroke='black'/>\n"
        ));
        svg.push_str(&format!(
            "<text x='{x}' y='{}' text-anchor='middle' font-size='10'>{}</text>\n",
            y - 22,
            node.id
        ));
    }

    svg.push_str("</svg>\n");
    fs::write(out_path, svg)
}
