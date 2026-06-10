
mod route;
use crate::route::{create_posts, create_uu, delete_users, find_posts, login, published_posts, show_all_posts};
use axum::routing::{ post};
use axum::{routing::get, Router};
use diesql::{create_pool, DbPool};
use http::Method;
use tower_http::cors::{Any, CorsLayer};



#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
}


#[tokio::main]
async fn main() {
    // 配置 CORS 策略
    let cors = CorsLayer::new()
        // 允许的跨域来源，开发阶段可以允许任意来源。生产环境建议指定具体域名如 "http://localhost:5173".parse().unwrap()
        .allow_origin(Any)
        // 允许的请求方法
        .allow_methods([Method::POST, Method::GET])
        // 允许的请求头（发送 JSON 必须允许 Content-Type）
        .allow_headers([http::header::CONTENT_TYPE]);

    let pool = create_pool();
    let shared_state=  AppState{pool: pool.clone()};

    // build our application with a single route
    let app = Router::new().route("/api/Register", post(create_uu))
        .route("/",get(|| async { "hello,world" }))
        .route("/api/Login", post(login))
        .route("/api/CreatePost", post(create_posts))
        .route("/api/FindPost", post(find_posts))
        .route("/api/AllPosts", get(show_all_posts))
        .route("/api/Publish", post(published_posts))
        .route("/api/DeleteUser", post(delete_users))
        .with_state(shared_state)
        .layer(cors);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}