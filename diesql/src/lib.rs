mod models;
mod schema;


use crate::models::{NewPost, NewUser, Post, User};
use crate::schema::posts::dsl::posts;
use crate::schema::posts::{published, title};
use crate::schema::users;
use crate::schema::users::{ username};
use diesel::associations::HasTable;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenvy::dotenv;
use std::env;
use argon2::{Argon2, PasswordHasher};

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn create_pool() -> DbPool {
    dotenv().ok();
    let database_url =
        env::var("DATABASE_URL").unwrap();

    let manager =
        ConnectionManager::<PgConnection>::new(database_url);

    let pool = Pool::builder()
        .max_size(80)
        .build(manager)
        .unwrap();
    pool
}

// pub fn establish_connection() -> PgConnection {
//     dotenv().ok();
//     let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//     PgConnection::establish(&database_url)
//         .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
// }


pub fn hash_password(hash_password: &str) -> Result<String, argon2::password_hash::Error> {
    let test_password = hash_password.as_bytes();
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(test_password)?.to_string();
    Ok(password_hash)
}


// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         let test_password = "password";
//         let hash_result=hash_password(test_password).unwrap();
//         println!("{}", test_password);
//         println!("{}", hash_result);
//     }
// }



// create user
pub fn create_user(conn: &mut PgConnection, user_name: &str, user_password: &str) -> Option<User> {

    let result_password =hash_password(user_password).ok()?;

    let new_user= NewUser{
        username:user_name,
        password: &result_password,
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .returning(User::as_returning())
        .on_conflict(username)    // 发生冲突时
        .do_nothing()             // PostgresSQL 层面什么都不做，这会导致底层返回给 diesel 的是一个空结果集
        .get_result::<User>(conn) // 此时 diesel 拿不到结果，会产生一个 NotFound 错误
        .optional()               // 关键核心：.optional() 会把 NotFound 自动吞掉，变成 Ok(None)
        .expect("Error saving new user") // 此时解包出来的是 Option<User>

}

pub fn delete_user(conn: &mut PgConnection, user_name: &str) ->bool {
     diesel::delete(users::table.filter(username.eq(user_name)))
    .execute(conn)
         .map(|rows_affected| rows_affected > 0)
    .expect("Error deleting users")

}

// find user in table that table name is User
pub fn verify_user(conn: &mut PgConnection, user_name: &str, user_password: &str) ->Result<User, String> {

    let result_password =hash_password(user_password).map_err(|err| err.to_string())?;
    //在表中查找username
    let user_result = users::table
    .filter(username.eq(user_name))
    .first::<User>(conn);

    match user_result {
        Ok(user) => {
            if user.password == &*result_password {
                Ok(user)
            }else {
                Err("Wrong password".to_string())
            }
        }
        _ => {
            Err("Wrong username".to_string())
        }
    }
}


// create post
pub fn create_post(conn: &mut PgConnection, post_title: &str, post_body: &str) -> Option<Post> {

    let new_post = NewPost { title:post_title, body:post_body };

    diesel::insert_into(posts::table())
        .values(&new_post)
        .returning(Post::as_returning())
        .get_result(conn)
        .optional()
        .expect("Error saving new post")
}
//find post
pub fn verify_post(conn: &mut PgConnection, post_title: &str) ->Result<Post, String> {
    let post_result = posts
        .filter(title.eq(post_title))// 注意：title 是列名，post_title 是变量
        .filter(published.eq(true))
        .select(Post::as_select())
        .first::<Post>(conn)
        .optional();
    match post_result {
        Ok(Some(post)) => {Ok(post)},
        Ok(None) => Err(format!("No published post found with title: {}", post_title)),
        _ => {
            Err("Wrong post".to_string())
        }
    }
}

pub fn get_publish_post(conn: &mut PgConnection,post_id:&i32) ->QueryResult<bool> {
    Ok(diesel::update(posts.find(post_id))
        .set(published.eq(true))
        .execute(conn)
        .map(|rows| rows > 0) // 如果成功，将行数转换为 bool
        .unwrap_or(false)) // 如果失败（Err），直接返回 false


}

pub fn get_all_posts(conn: &mut PgConnection) -> QueryResult<Vec<Post>> {
    posts.load::<Post>(conn)   // 自动选择所有列
}