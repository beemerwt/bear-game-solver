use std::path::Path;

use bear_game_solver::{
    boards::bear_game_board::BEAR_GAME_BOARD,
    debug::render_debug_board,
    game::{GameState, Move, Outcome, SideToMove},
    graph::Graph,
    render::render_svg,
    solver::{Solver, apply_move, legal_bear_moves, legal_hunter_moves},
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
    Serve,
}

#[tokio::main]
async fn main() {
    match Cli::parse().command {
        Command::Validate => validate(),
        Command::Moves => moves(),
        Command::Solve => solve(),
        Command::Policy { limit, extra } => policy(limit.or(parse_limit(&extra))),
        Command::RenderSvg => render_svg_cmd(),
        Command::Serve => bear_game_solver::web::serve(build()).await,
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

    match s.start_eval.outcome {
        Outcome::HuntersWin => {
            let safe = s.reachable_policy.get(&start).cloned().unwrap_or_default();
            println!("The configured starting state is HuntersWin.");
            println!(
                "Best first hunter move (minimax-optimal): {}",
                safe.first()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "(none)".to_string())
            );
            println!(
                "All first hunter moves that preserve the forced win ({}):",
                safe.len()
            );
            for mv in safe {
                println!("- {mv}");
            }
            println!(
                "Worst-case number of plies until capture (bear resists perfectly): {}",
                s.start_eval.distance
            );
            let pv = principal_variation(&graph, &mut solver, start.clone(), 24);
            println!("Principal variation (example line): {}", pv.join(" -> "));
            println!(
                "Note: the proof is the full minimax result over all legal branches, not this single line."
            );
        }
        Outcome::BearWin => {
            println!("The configured starting state is BearWin.");
            if start.side_to_move == SideToMove::Hunters {
                let hunter_moves = legal_hunter_moves(&graph, &start);
                let mut refuting = vec![];
                for mv in hunter_moves {
                    let child = solver.evaluate_state(apply_move(&start, &mv));
                    if child.outcome == Outcome::BearWin {
                        refuting.push(mv);
                    }
                }
                if !refuting.is_empty() {
                    println!("Hunter first moves that allow a bear-forced survival:");
                    for mv in &refuting {
                        println!("- {mv}");
                    }
                    println!(
                        "Decision point: after hunter move {}, the bear can force survival to the move limit.",
                        refuting[0]
                    );
                }
            }
            let pv = principal_variation(&graph, &mut solver, start.clone(), 24);
            println!(
                "Bear survival/refutation line (example): {}",
                pv.join(" -> ")
            );
            println!(
                "Note: bear only needs one survival branch against each hunter plan to refute a forced hunter win."
            );
        }
    }
}

fn principal_variation(
    graph: &Graph,
    solver: &mut Solver<'_>,
    start: GameState,
    max_plies: usize,
) -> Vec<String> {
    let mut line = Vec::new();
    let mut st = start;
    for _ in 0..max_plies {
        let ev = solver.evaluate_state(st.clone());
        if ev.distance == 0 {
            break;
        }
        let chosen = choose_pv_move(graph, solver, &st, &ev);
        let Some(mv) = chosen else {
            break;
        };
        line.push(mv.to_string());
        st = apply_move(&st, &mv);
    }
    line
}

fn choose_pv_move(
    graph: &Graph,
    solver: &mut Solver<'_>,
    st: &GameState,
    ev: &bear_game_solver::solver::StateEval,
) -> Option<Move> {
    match (st.side_to_move, ev.outcome) {
        (SideToMove::Hunters, Outcome::HuntersWin) => ev.winning_hunter_moves.first().cloned(),
        (SideToMove::Hunters, Outcome::BearWin) => {
            let moves = legal_hunter_moves(graph, st);
            moves
                .into_iter()
                .find(|mv| solver.evaluate_state(apply_move(st, mv)).outcome == Outcome::BearWin)
        }
        (SideToMove::Bear, Outcome::BearWin) => ev.refuting_bear_moves.first().cloned(),
        (SideToMove::Bear, Outcome::HuntersWin) => {
            let moves = legal_bear_moves(graph, st);
            moves
                .into_iter()
                .max_by_key(|mv| solver.evaluate_state(apply_move(st, mv)).distance)
        }
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
