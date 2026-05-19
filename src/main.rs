use std::path::Path;

use bear_game_solver::{
    boards::bear_game_board::BEAR_GAME_BOARD,
    debug::render_debug_board,
    game::{GameState, Outcome, SideToMove},
    graph::Graph,
    render::render_svg,
    solver::{Solver, legal_hunter_moves},
};
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Validate,
    Moves,
    Solve,
    Policy {
        #[arg(long)]
        limit: Option<usize>,
        #[arg(last = true)]
        extra: Vec<String>,
    },
    RenderSvg,
}

fn main() {
    match Cli::parse().command {
        Command::Validate => validate(),
        Command::Moves => moves(),
        Command::Solve => solve(),
        Command::Policy { limit, extra } => policy(limit.or(parse_limit(&extra))),
        Command::RenderSvg => render_svg_cmd(),
    }
}
fn build() -> Graph {
    Graph::from_board_definition(&BEAR_GAME_BOARD).unwrap_or_else(|e| {
        eprintln!("Board validation failed: {e}");
        std::process::exit(1);
    })
}
fn validate() {
    let graph = build();
    println!("Board is valid: {}", graph.name);
    println!("Nodes: {}", graph.nodes.len());
    println!(
        "Edges: {}",
        graph.adjacency.values().map(Vec::len).sum::<usize>() / 2
    );
}
fn moves() {
    let graph = build();
    let state = GameState::from_start(&BEAR_GAME_BOARD.start);
    println!("{}", render_debug_board(&graph, Some(&state)));
    println!("Legal hunter moves from start:");
    for mv in legal_hunter_moves(&graph, &state) {
        println!("- {mv}");
    }
}
fn solve() {
    let graph = build();
    let start = GameState::from_start(&BEAR_GAME_BOARD.start);
    let mut solver = Solver::new(&graph);
    let s = solver.solve_with_summary(start.clone());
    println!("Board: {}", BEAR_GAME_BOARD.name);
    println!("Node count: {}", graph.nodes.len());
    println!("Total valid states: {}", s.total_valid_states);
    println!(
        "Total reachable states from start: {}",
        s.total_reachable_states
    );
    println!("HuntersWin states: {}", s.hunters_win_states);
    println!("BearWin states: {}", s.bear_win_states);
    println!(
        "Start guaranteed hunter win: {}",
        s.start_eval.outcome == Outcome::HuntersWin
    );
    if s.start_eval.outcome == Outcome::HuntersWin {
        let safe = s.reachable_policy.get(&start).cloned().unwrap_or_default();
        println!("Safe first hunter moves: {}", safe.len());
        for mv in safe {
            println!("- {mv}");
        }
        println!("Worst-case trap distance: {}", s.start_eval.distance);
    } else {
        println!(
            "Bear can force escape/timeout within turn limit. First-layer refutations from bear-turn start: {:?}",
            s.reachable_refutations
                .get(&GameState {
                    side_to_move: SideToMove::Bear,
                    ..start
                })
                .cloned()
                .unwrap_or_default()
        );
    }
}
fn parse_limit(extra: &[String]) -> Option<usize> {
    extra.windows(2).find_map(|w| {
        if w[0] == "--limit" {
            w[1].parse::<usize>().ok()
        } else {
            None
        }
    })
}

fn policy(limit: Option<usize>) {
    let graph = build();
    let start = GameState::from_start(&BEAR_GAME_BOARD.start);
    let mut solver = Solver::new(&graph);
    let s = solver.solve_with_summary(start);
    let mut n = 0usize;
    for st in s
        .reachable_set
        .iter()
        .filter(|x| x.side_to_move == SideToMove::Hunters)
    {
        if let Some(max) = limit
            && n >= max
        {
            break;
        }
        let ev = solver.evaluate_state(st.clone());
        if ev.outcome == Outcome::HuntersWin {
            println!(
                "state: bear={}, hunters={:?}, turns={}",
                st.bear, st.hunters, st.hunter_turns_used
            );
            for mv in ev.winning_hunter_moves {
                println!("  - {mv}");
            }
            n += 1;
        }
    }
}
fn render_svg_cmd() {
    let graph = build();
    let state = GameState::from_start(&BEAR_GAME_BOARD.start);
    let path = Path::new("target/bear_game_board.svg");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("failed to create output directory");
    }
    render_svg(&graph, Some(&state), path).expect("failed to render svg");
    println!("Wrote {}", path.display());
}
