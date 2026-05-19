#[cfg(test)]
mod tests {
    use bear_game_solver::{
        boards::bear_game_board::BEAR_GAME_BOARD,
        game::GameState,
        graph::Graph,
        solver::{Solver, legal_hunter_moves},
    };

    #[test]
    fn board_builds() {
        let graph = Graph::from_board_definition(&BEAR_GAME_BOARD).expect("board should be valid");
        assert!(!graph.nodes.is_empty());
    }

    #[test]
    fn hunter_moves_exist_from_start() {
        let graph = Graph::from_board_definition(&BEAR_GAME_BOARD).expect("board should be valid");
        let state = GameState::from_start(&BEAR_GAME_BOARD.start);
        assert!(!legal_hunter_moves(&graph, &state).is_empty());
    }

    #[test]
    fn solve_is_deterministic() {
        let graph = Graph::from_board_definition(&BEAR_GAME_BOARD).expect("board should be valid");
        let state = GameState::from_start(&BEAR_GAME_BOARD.start);
        let mut solver_a = Solver::new(&graph);
        let mut solver_b = Solver::new(&graph);
        let a = solver_a.evaluate_state(state.clone());
        let b = solver_b.evaluate_state(state);
        assert_eq!(a.outcome, b.outcome);
    }
}
