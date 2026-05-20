use std::sync::{Arc, Mutex};

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::services::ServeDir;

use crate::{
    api::{
        AppState, apply_best_hunter_move, apply_move_endpoint, board, evaluate, legal_moves,
        manual_bear_move, recommend_hunter_move,
    },
    graph::Graph,
    solver::Solver,
};

pub async fn serve(graph: Graph) {
    let graph_ref: &'static Graph = Box::leak(Box::new(graph));
    let state = AppState {
        graph: Arc::new(graph_ref.clone()),
        solver: Arc::new(Mutex::new(Solver::new(graph_ref))),
    };
    let app = Router::new()
        .route("/api/board", get(board))
        .route("/api/legal-moves", post(legal_moves))
        .route("/api/evaluate", post(evaluate))
        .route("/api/apply-move", post(apply_move_endpoint))
        .route("/api/recommend-hunter-move", post(recommend_hunter_move))
        .route("/api/manual-bear-move", post(manual_bear_move))
        .route("/api/apply-best-hunter-move", post(apply_best_hunter_move))
        .fallback_service(
            ServeDir::new("static/index.html")
                .append_index_html_on_directories(true)
                .fallback(ServeDir::new("static")),
        )
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("bind");
    println!("Serving on http://127.0.0.1:3000");
    axum::serve(listener, app).await.expect("server");
}
