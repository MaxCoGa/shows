use app::app;
use app::database::init_db;
use dotenvy::dotenv;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    dotenv().ok(); // Load environment variables from .env file

    let db_pool = init_db().await.expect("Failed to initialize database");

    let app = app(db_pool);

    // run it with hyper on localhost:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
