use axum::{routing::{get, post}, Router, Json, extract::State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

#[derive(Clone)]
struct AppState {
    votes: Arc<Mutex<HashMap<String, i64>>>,
}

#[derive(Deserialize)]
struct Vote {
    poll_id: String,
    choice: String,
}

#[derive(Serialize)]
struct VoteResponse {
    status: String,
    count: i64,
}

async fn health() -> &'static str {
    "version 0.0.2"
}

async fn submit_vote(State(state): State<AppState>, Json(vote): Json<Vote>) -> Json<VoteResponse> {
    println!("Received vote: poll_id={}, choice={}", vote.poll_id, vote.choice);
    let key = format!("poll:{}:{}", vote.poll_id, vote.choice);
    let mut votes = state.votes.lock().await;
    let count = votes.entry(key.clone()).or_insert(0);
    *count += 1;

    let status = if rand::random::<f32>() < 0.1 {
        format!("Vote for {} counted, Fort Atkinson legend!", vote.poll_id)
    } else {
        format!("Vote for {} recorded", vote.poll_id)
    };

    println!("Returning: status={}, count={}", status, count);
    Json(VoteResponse { status, count: *count })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState {
        votes: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/", get(health))
        .route("/vote", post(submit_vote))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
