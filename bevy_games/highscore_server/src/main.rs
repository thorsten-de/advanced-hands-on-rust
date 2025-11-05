use std::sync::Arc;

use axum::{Json, Router, extract::State, response::Html, routing::get, routing::post};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/submit-score", post(submit_score))
        .route("/", get(high_scores_html))
        .route("/highscores", get(high_scores_json))
        .with_state(Arc::new(Mutex::new(HighScoreTable::new())));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3030")
        .await
        .unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn submit_score(
    State(table): State<Arc<Mutex<HighScoreTable>>>,
    high_score: Json<HighScoreEntry>,
) {
    println!("Received high score {:?}", high_score);
    let mut lock = table.lock().await;
    lock.add_entry(HighScoreEntry {
        name: high_score.name.clone(),
        score: high_score.score,
    });
}

async fn high_scores_json(State(table): State<Arc<Mutex<HighScoreTable>>>) -> Json<HighScoreTable> {
    let lock = table.lock().await;
    let table = lock.clone();
    Json(table)
}
async fn high_scores_html(State(table): State<Arc<Mutex<HighScoreTable>>>) -> Html<String> {
    let mut html = String::from("<h1>High Scores</h1>");
    html.push_str("<table>");
    html.push_str("<tr><th>Name</th><th>Score</th></tr>");
    for entry in &table.lock().await.entries {
        html.push_str("<tr>");
        html.push_str("<td>");
        html.push_str(&entry.name);
        html.push_str("</td>");
        html.push_str("<td>");
        html.push_str(&entry.score.to_string());
        html.push_str("</td>");
        html.push_str("</tr>");
    }
    html.push_str("</table>");

    Html(html)
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct HighScoreEntry {
    name: String,
    score: u32,
}

/// A table of high-score entries
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct HighScoreTable {
    entries: Vec<HighScoreEntry>,
}

const HIGHSCORES: &str = "high_scores.json";

impl HighScoreTable {
    fn new() -> Self {
        if std::path::Path::new(HIGHSCORES).exists() {
            let file = std::fs::File::open(HIGHSCORES).unwrap();
            serde_json::from_reader(file).unwrap()
        } else {
            Self {
                entries: Vec::new(),
            }
        }
    }

    fn add_entry(&mut self, entry: HighScoreEntry) {
        self.entries.push(entry);
        self.entries.sort_by(|a, b| b.score.cmp(&a.score));
        self.entries.truncate(10);
        self.save();
    }

    fn save(&self) {
        let file = std::fs::File::create(HIGHSCORES).unwrap();
        serde_json::to_writer(file, self).unwrap();
    }
}
