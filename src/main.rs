use std::path::Path;

use bear_game_solver::{
    boards::bear_game_board::BEAR_GAME_BOARD,
    debug::render_debug_board,
    game::GameState,
    graph::Graph,
    render::render_svg,
    solver::{Solver, legal_bear_moves, legal_hunter_moves},
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
    RenderSvg,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Validate => validate(),
        Command::Moves => moves(),
        Command::Solve => solve(),
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
    if graph.warnings.is_empty() {
        println!("Warnings: none");
    } else {
        for w in graph.warnings {
            println!("{w}");
        }
    }
}

fn moves() {
    let graph = build();
    let state = GameState::from_start(&BEAR_GAME_BOARD.start);
    println!("{}", render_debug_board(&graph, Some(&state)));
    println!("Legal hunter moves from start:");
    for mv in legal_hunter_moves(&graph, &state) {
        println!("- {mv}");
    }
    let mut bear_state = state.clone();
    bear_state.side_to_move = bear_game_solver::game::SideToMove::Bear;
    println!("Legal bear moves from start (if bear to move):");
    for mv in legal_bear_moves(&graph, &bear_state) {
        println!("- {mv}");
    }
}

fn solve() {
    let graph = build();
    let state = GameState::from_start(&BEAR_GAME_BOARD.start);
    let mut solver = Solver::new(&graph);
    let result = solver.solve(state.clone());

    println!("Board: {}", BEAR_GAME_BOARD.name);
    println!("Start bear: {}", state.bear);
    println!("Start hunters: {:?}", state.hunters);
    println!("Turn limit (hunter turns): {}", BEAR_GAME_BOARD.turn_limit);
    println!("Outcome: {:?}", result.outcome);
    println!("Distance (plies): {}", result.distance);
    println!("Best first move: {:?}", result.best_move);
    println!("Principal variation:");
    for (i, mv) in result.principal_variation.iter().enumerate() {
        println!("  {}. {}", i + 1, mv);
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
