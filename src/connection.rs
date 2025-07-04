use askama::Template;
use axum::{extract::State, response::{Html, IntoResponse}, routing::get, Router};
use axum::http::StatusCode;
use tokio::net::TcpListener;
use sqlx::{FromRow, Pool};
use std::{fs, net::SocketAddr};

#[derive(Debug, FromRow)]
pub struct Task {
    pub id: i64,
    pub description: String,
    pub urgent: bool,
    pub important: bool,
}

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomeTemplate<'a> {
    pub tasks: &'a [Task],
}

pub async fn run() {
    let pool = Pool::connect("sqlite://tasks.db").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let app = Router::new()
        .route("/", get(home_handler))
        .fallback(not_found_handler)
        .with_state(pool.clone());

    let addr = SocketAddr::from(([127, 0, 0, 1], 7878));
    println!("Listening on http://{}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn home_handler(
    State(pool): State<Pool<sqlx::Sqlite>>,
) -> impl IntoResponse {
    let tasks = sqlx::query_as::<_, Task>(
        "SELECT id, description, urgent, important FROM tasks"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_else(|_| vec![]);

    let template = HomeTemplate { tasks: &tasks };
    match template.render() {
        Ok(contents) => Html(contents).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error"
        ).into_response(),
    }
}

async fn not_found_handler() -> impl IntoResponse {
    match fs::read_to_string("404.html") {
        Ok(contents) => Html(contents).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error"
        ).into_response(),
    }
}

