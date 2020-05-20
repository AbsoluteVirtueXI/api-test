#![deny(warnings)]

//use warp::Filter;

/// Provides a RESTfull web server managing some Todos
/// API will be:
///
/// - `GET /todos`: return a JSON list of Todos
/// - `POST /todos`: create a new Todo
/// - `PUT /todos/:id`: update a specific Todo.
/// - `DELETE /todos/:id`: delete a specific Todo.

#[tokio::main]
async fn main() {
    // Db intialization
    let db = models::blank_db();

    // Define api filter
    let api = filters::todos(db);

    // Define root of all our routes
    let routes = api;

    // Start server
    warp::serve(routes).run(([192, 168, 0, 10], 3030)).await;
}

mod filters {
    use super::handlers;
    use super::models::{Db, ListOptions, Todo};
    use warp::Filter;

    /// The 4 TODOs filters combined.
    pub fn todos(db: Db,) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        todos_list(db.clone())
            .or(todos_create(db.clone()))
            .or(todos_update(db.clone()))
            .or(todos_delete(db))
    }

    /// GET /todos?offset=3&limit=5
    pub fn todos_list(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("todos")
            .and(warp::get())
            .and(warp::query::<ListOptions>())
            .and(with_db(db))
            .and_then(handlers::list_todos)
    }

    /// POST /todos with JSON body
    pub fn todos_create(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("todos")
            .and(warp::post())
            .and(json_body())
            .and(with_db(db))
            .and_then(handlers::create_todos)
    }

    /// PUT /todos/:id with JSON BODY
    pub fn todos_update(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("todos" / u64)
            .and(warp::put())
            .and(json_body())
            .and(with_db(db))
            .and_then(handlers::update_todo)
    }

    /// DELETE /todos/:id
    pub fn todos_delete(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        // we'll make one of our endpoints admin-only to show how authentification filters are used
        let admin_only = warp::header::exact("authorization", "Bearer admin");

        warp::path!("todos" / u64)
            .and(admin_only)
            .and(warp::delete())
            .and(with_db(db))
            .and_then(handlers::delete_todo)
    }

    /// Make the db accessible within filter
    fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }

    fn json_body() -> impl Filter<Extract= (Todo,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }
}


/// These are ou API handlers, the ends of each filter chain.
/// Notice how thanks to using `Filter::and`, we can define a function
/// with the exact arguments we'd expect from each filter in the chain.
/// No tuples are needed, it's auto flattened for the functions.
mod handlers {
    use super::models::{Db, ListOptions, Todo};
    use std::convert::Infallible;
    use warp::http::StatusCode;

    pub async fn list_todos(opts: ListOptions, db: Db) -> Result<impl warp::Reply, Infallible> {
        // Just return a JSON array of todos, applying the limit and offset.
        let todos = db.lock().await;
        let todos: Vec<Todo> = todos
            .clone()
            .into_iter()
            .skip(opts.offset.unwrap_or(0))
            .take(opts.limit.unwrap_or(std::usize::MAX))
            .collect();
        Ok(warp::reply::json(&todos))
    }

    pub async fn create_todos(create: Todo, db: Db) -> Result<impl warp::Reply, Infallible> {
        let mut vec = db.lock().await;

        for todo in vec.iter() {
            if todo.id == create.id {
                return Ok(StatusCode::BAD_REQUEST);
            }
        }

        // No existing Todo with id, so insert and return `201 Created`
        vec.push(create);
        Ok(StatusCode::CREATED)
    }

    pub async fn update_todo(id: u64, update: Todo, db: Db) -> Result<impl warp::Reply, Infallible> {
        let mut vec = db.lock().await;

        // Look for the specified Todo..
        for todo in vec.iter_mut() {
            if todo.id == id {
                *todo = update;
                return Ok(StatusCode::OK)
            }
        }
        // If the for loop didn't return OK, then the id doesn't exist.
        Ok(StatusCode::NOT_FOUND)
    }

    pub async fn delete_todo(id: u64, db: Db) -> Result<impl warp::Reply, Infallible> {
        let mut vec = db.lock().await;
        let len = vec.len();
        vec.retain(|todo| {
            // Retain all Todos that aren't this id..
            todo.id != id
        });

        // If the vec is smaller, we found and deleted the Todo!
        let deleted = vec.len() != len;
        if deleted {
            // Respond with a `204 No Content`, which means successful
            // yet no body expected...
            Ok(StatusCode::NO_CONTENT)
        } else {
            Ok(StatusCode::NOT_FOUND)
        }
    }
}

mod models {
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // So we don't have to tackle how different database work, we'll just use
    // a simple in-memory DB, a vector synchronized by Mutex
    pub type Db = Arc<Mutex<Vec<Todo>>>;

    #[derive(Debug, Deserialize, Serialize, Clone)] // TODO: why clone?
    pub struct Todo {
        pub id: u64,
        pub text: String,
        pub completed: bool
    }

    // The query parameters for list_todos.
    #[derive(Debug, Deserialize)]
    pub struct ListOptions {
        pub offset: Option<usize>,
        pub limit: Option<usize>,
    }

    pub fn blank_db() -> Db {
        Arc::new(Mutex::new(Vec::new()))
    }
}