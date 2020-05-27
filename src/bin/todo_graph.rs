//#![deny(warnings)]

use warp::Filter;


/// Provides a RESTfull web server managing some Todos
/// API will be:
///
/// - `GET /todos`: return a JSON list of Todos
/// - `POST /todos`: create a new Todo
/// - `PUT /todos/:id`: update a specific Todo.
/// - `DELETE /todos/:id`: delete a specific Todo.

use juniper::{FieldResult, EmptySubscription};
use std::sync::Arc;
#[tokio::main]
async fn main() {
    // Db intialization, i keep the workd blank_db, but this database is not blank anymore
    let db = models::blank_db().await;


    let context = warp::any().and(filters::with_db(db.clone())).map(|db: models::Db|
        gql::Context{pool: db}
    );

    let graphql_filter = juniper_warp::make_graphql_filter(gql::schema(), context.boxed());

    // Define api filter
    let api = filters::todos(db);


    // Define root of all our routes
    let routes = api;


    let routes = routes.or(warp::path("graphql").and(graphql_filter));

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
        warp::path("todos")
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
        warp::path!("todos" / i32)
            .and(warp::put())
            .and(json_body())
            .and(with_db(db))
            .and_then(handlers::update_todo)
    }

    /// DELETE /todos/:id
    pub fn todos_delete(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        // we'll make one of our endpoints admin-only to show how authentification filters are used
        let admin_only = warp::header::exact("authorization", "Bearer admin");

        warp::path!("todos" / i32)
            .and(admin_only)
            .and(warp::delete())
            .and(with_db(db))
            .and_then(handlers::delete_todo)
    }


    /// Make the db accessible within filter
    pub fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
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
    use super::models::{Db, ListOptions, Todo, db_list_todos, db_create_todos, db_update_todo, db_delete_todo};
    use std::convert::Infallible;
    use warp::http::StatusCode;

    pub async fn list_todos(opts: ListOptions, db: Db) -> Result<impl warp::Reply, Infallible> {
        let todos_list = db_list_todos(opts.offset.unwrap_or(0), opts.limit.unwrap_or(std::i32::MAX), &db).await;
        Ok(warp::reply::json(&todos_list))
    }

    pub async fn create_todos(create: Todo, db: Db) -> Result<impl warp::Reply, Infallible> {
        let row = db_create_todos(create.id, create.text, create.completed, &db).await;
        if row != 0 {
            Ok(StatusCode::CREATED)
        } else {
            Ok(StatusCode::BAD_REQUEST)
        }
    }

    pub async fn update_todo(id: i32, update: Todo, db: Db) -> Result<impl warp::Reply, Infallible> {
        let rows = db_update_todo(id, update.text, update.completed, &db).await;
        if rows != 0 {
            Ok(StatusCode::OK)
        } else {
            Ok(StatusCode::NOT_FOUND)
        }
    }

    pub async fn delete_todo(id: i32, db: Db) -> Result<impl warp::Reply, Infallible> {
        let rows = db_delete_todo(id, &db).await;
        if rows != 0 {
            Ok(StatusCode::NO_CONTENT)
        } else {
            Ok(StatusCode::NOT_FOUND)
        }
    }
}

mod models {
    use serde::{Deserialize, Serialize};
    use sqlx::PgPool;
    use std::env;

    // So we don't have to tackle how different database work, we'll just use
    // a simple in-memory DB, a vector synchronized by Mutex
    //1 pub type Db = Arc<Mutex<PgPool>>;
    pub type Db = PgPool;

    #[derive(Debug, Deserialize, Serialize, Clone)] //, sqlx::FromRow
    #[derive(juniper::GraphQLObject)]
    pub struct Todo {
        pub id: i32,
        pub text: String,
        pub completed: bool
    }

    #[derive(juniper::GraphQLInputObject)]
    #[graphql(description="A todo list")]
    pub struct NewTodo {
        pub text: String,
        pub completed: bool
    }

    // The query parameters for list_todos.
    #[derive(Debug, Deserialize)]
    #[derive(juniper::GraphQLInputObject)]
    pub struct ListOptions {
        pub offset: Option<i32>,
        pub limit: Option<i32>,
    }

    pub async fn blank_db() -> Db {
        dotenv::dotenv().ok();
        //Arc::new(Mutex::new(Vec::new()))
        //let pool = SqlitePool::new("sqlite:///Users/akersof/CLionProjects/warp-tutorial/todos.db").await.unwrap();
        let pool = PgPool::builder().max_size(10).build(&env::var("DATABASE_URL").unwrap()).await.unwrap();
        //1 Arc::new(Mutex::new(pool))
        pool
    }

    // Here perform various known request, they will be called by the corresponding handler
    pub async fn db_list_todos(offset: i32, limit: i32, db: &Db) -> Vec<Todo> {
        let todos_list = sqlx::query_as!(Todo, "SELECT * FROM todos WHERE id BETWEEN $1 and $2", offset, limit)
            .fetch_all(db).await.unwrap();
        todos_list
    }

    pub async fn db_create_todos(id: i32, text: String, completed: bool, db: &Db) -> u64 {
        let rows = sqlx::query!("INSERT INTO todos VALUES($1, $2, $3)", id, text, completed)
            .execute(db).await.unwrap();
        rows
    }

    pub async fn db_update_todo(id: i32, text: String, completed: bool, db: &Db) -> u64 {
        let rows = sqlx::query!("UPDATE todos SET text = $1, completed = $2 where id = $3", text, completed, id)
            .execute(db).await.unwrap();
        rows
    }

    pub async fn db_delete_todo(id: i32, db: &Db) -> u64 {
        let rows = sqlx::query!("DELETE FROM todos WHERE id = $1", id)
            .execute(db).await.unwrap();
        rows
    }
}

mod gql {
    use juniper::{FieldResult, EmptyMutation ,EmptySubscription};
    use serde::{Deserialize, Serialize};
    use super::models::{Db, Todo, db_list_todos};

    #[derive(juniper::GraphQLInputObject)]
    pub struct NewTodo {
        pub text: String,
        pub completed: bool
    }

    #[derive(Debug, Deserialize)]
    #[derive(juniper::GraphQLInputObject)]
    pub struct ListOptions {
        pub offset: Option<i32>,
        pub limit: Option<i32>,
    }

    pub struct Context {
        pub pool: Db
    }

    impl juniper::Context for Context {}

    pub struct Query;

    #[juniper::graphql_object(Context = Context,)]
    impl Query {
        fn apiVersion() -> &str {
            "1.0"
        }

        async fn todosList(context: &Context, opt: ListOptions) -> FieldResult<Vec<Todo>> {
            let res = db_list_todos(opt.offset.unwrap_or(0), opt.limit.unwrap_or(1000), &context.pool).await;
            Ok(res)
        }
    }

    pub type Schema = juniper::RootNode<'static, Query, EmptyMutation<Context>, EmptySubscription<Context>>;

    pub fn schema() -> Schema {
        Schema::new(Query, EmptyMutation::<Context>::new(), EmptySubscription::<Context>::new())
    }
}