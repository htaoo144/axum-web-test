use crate::AppState;
use axum::extract::{State};
use axum::response::IntoResponse;
use axum::Json;
use diesql::{create_post, create_user, delete_user, get_all_posts, get_publish_post, verify_post, verify_user};
use http::StatusCode;

use serde::{Deserialize, Serialize};


#[derive(Deserialize)]
pub struct LoginInfo{
    pub username: String,
    pub password: String,
}
#[derive(Deserialize)]
pub struct DeleteInfo{
    pub username: String,
}
#[derive(Deserialize)]
pub struct PostInfo{
    pub title: String,
    pub body: String,
}
#[derive(Deserialize)]
pub struct PublishInfo{
    pub id:i32,
}
#[derive(Deserialize)]
pub struct FindPostInfo{
    pub title:String,
}

#[derive(Serialize)]
pub struct PostResponse{
    pub id: i32,
    pub title: String,
    pub body: String,
}

#[derive(Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
}



pub(crate) async fn create_uu(State(state):State<AppState>,Json(info):Json<LoginInfo>)->impl IntoResponse{
    // 使用 spawn_blocking 将阻塞任务丢给线程池，并通过 move 转移 info 的所有权
    let user_result = tokio::task::spawn_blocking(move || {
        let mut conn = state.pool.get()
            .expect("Database connection timeout");
        create_user(&mut conn, &info.username, &info.password)
    }).await;// 得到 Result<Option<User>, JoinError>
    // 正确处理两层嵌套：第一层是 Tokio 的 JoinResult，第二层是数据库 Option
    match user_result {
        // 1. 线程池正常执行完毕
        Ok(maybe_user) => match maybe_user {
            // 2. 数据库成功插入，返回了用户
            Some(_) => (
                StatusCode::CREATED,
                Json(ApiResponse {
                success: true,
                message: "注册成功".to_string(),
            }),
            ),
            // 3. 数据库触发唯一约束冲突，返回了 None
            None => (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse {
                success: false,
                message: "用户名已存在".to_string(),
            }),
            ),
        },
        // 4. Tokio 线程池自身发生异常（极少发生）
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
            success: false,
            message: "服务器内部线程错误".to_string(),
        }),
        )
    }
}

pub(crate) async fn delete_users(State(state):State<AppState>, Json(info):Json<DeleteInfo>) ->impl IntoResponse{
    let user_result = tokio::task::spawn_blocking(move || {
        let mut conn = state.pool.get()
        .expect("Database connection timeout");
        delete_user(&mut conn, &info.username)
    }).await;

    match user_result {
        Ok(user_real) => match user_real {
            true => Json(
                ApiResponse{
                    success: true,
                    // 可以动态地把数据库查出来的用户名拼进去
                    message: "删除成功! ，希望我们下次再见".to_string(),
            }),
            false =>Json(ApiResponse {
                success: false,
                message: "失败".to_string(),
            }),
        },
        Err(_) => Json(ApiResponse {
            success: false,
            message: "服务器内部错误：线程池异常".to_string(),
        }),

    }
}


pub(crate) async fn login(State(state):State<AppState>,Json(info):Json<LoginInfo>)->impl IntoResponse{
    let user_ok = tokio::task::spawn_blocking(move || {
        let mut conn = state.pool.get()
            .expect("Database connection timeout");
        verify_user(&mut conn, &info.username, &info.password)
    }).await; // 得到 Result<Result<User, String>, JoinError>
    // 2. 正确解包两层 Result
    match user_ok {
        // 第一层：Tokio 线程池正常调度完成了任务
        Ok(verify_result) => match verify_result {
            // 第二层：数据库验证成功，密码正确
            Ok(user) => Json(ApiResponse {
                success: true,
                // 可以动态地把数据库查出来的用户名拼进去
                message: format!("登录成功，欢迎 {}！", user.username),
            }),
            // 第二层：验证失败（用户不存在、或者密码错误）
            Err(err_msg) =>Json(ApiResponse {
                success: false,
                // 把 verify_user 传出来的具体错误原因（"密码错误"）返回给前端
                message: format!("登录失败: {}", err_msg),
            }),

        },
        // 第一层：Tokio 线程池自身发生异常（几乎遇不到，属于严谨性编写）
        Err(_) => Json(ApiResponse {
            success: false,
            message: "服务器内部错误：线程池异常".to_string(),
        }),

    }
}


pub(crate) async fn create_posts(State(state):State<AppState>,Json(info):Json<PostInfo>) ->impl IntoResponse{
    let posts_ok = tokio::task::spawn_blocking(move || {
        let mut conn = state.pool.get()
            .expect("Database connection timeout");
        create_post(&mut conn, &info.title, &info.body)
    }).await;
    match posts_ok {
        Ok(maybe_posts) => match maybe_posts {
            Some(_) => Json(ApiResponse{
                success: true,
                message: "OK!".to_string(),
            }),
            None => Json(ApiResponse {
                success: false,
                message: "NO!".to_string(),
            })
        },
        Err(_)=> Json(ApiResponse {
            success: false,
            message: "服务器内部线程错误".to_string(),
        }),

    }
}

pub(crate) async fn find_posts(State(state):State<AppState>,Json(info):Json<FindPostInfo>)->impl IntoResponse {
    let find_posts_ok = tokio::task::spawn_blocking(move || {
        let mut conn = state.pool.get()
            .expect("Database connection timeout");

        verify_post(&mut conn, &info.title)
    }).await;

    match find_posts_ok {
        Ok(Ok(post_result)) =>
            Json(PostResponse {
            id: post_result.id,
            title: post_result.title,
            body: post_result.body,
        }),
        _ => Json(PostResponse {
            id: 404,
            title: "not find".to_string(),
            body: "not find".to_string(),
        }),
    }
}

pub(crate) async fn show_all_posts(State(state):State<AppState>)->impl IntoResponse {
    let posts_ok = tokio::task::spawn_blocking(move || {
        let mut conn = state.pool.get()
            .expect("Database connection timeout");
        get_all_posts(&mut conn)
    }).await
        .map_err(|e| format!("Task join error: {e}")) // 处理外层 spawn_blocking 错误
        .and_then(|db_res| db_res.map_err(|e| format!("Database error: {e}"))); // 处理内层 DB 错误

    match posts_ok {
        Ok(posts) => Json(posts).into_response(),
        Err(err_msg) => (StatusCode::INTERNAL_SERVER_ERROR, err_msg).into_response(),
    }

}

pub(crate) async fn published_posts(State(state):State<AppState>,Json(info):Json<PublishInfo>)->impl IntoResponse{
    let posts_published_result = tokio::task::spawn_blocking(move || {
        // 获取数据库连接，这里不再 expect，而是返回 Result
        let mut conn = state.pool.get().map_err(|e| format!("Connection error: {}", e))?;
        get_publish_post(&mut conn, &info.id).map_err(|e| format!("Database error: {}", e))

    })
        .await;
    match posts_published_result {
        Ok(Ok(true)) => (
            StatusCode::NO_CONTENT,
            Json(ApiResponse {
            success: true,
            message: "OK!".to_string(),

        }),
        ),
        Ok(Ok(false)) =>(
            StatusCode::OK,
            Json(ApiResponse {
            success: false,
            message: "not find!".to_string(),

        }),
        ),
        Ok(Err(db_err)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
            success: false,
            message: db_err.to_string(),
        }),
        ),
        Err(join_err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
            success: false,
            message: format!("Internal task error: {}", join_err),
        }),
        ),
    }
}
