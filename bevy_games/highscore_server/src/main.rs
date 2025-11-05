use axum::{Json, Router, routing::post};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/submit-score", post(submit_score));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3030")
        .await
        .unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct HighScoreEntry {
    name: String,
    score: u32,
}

async fn submit_score(high_score: Json<HighScoreEntry>) {
    println!("Received high score {:?}", high_score);
}
