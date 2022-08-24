#[allow(unused_variables)]
use std::env;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use poem::web::Path;
use poem::{get, handler, listener::TcpListener, Route, Server};

use poem_casbin_auth::casbin::function_map::key_match2;
use poem_casbin_auth::casbin::{CoreApi, DefaultModel, FileAdapter};
use poem_casbin_auth::CasbinService;

fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

#[handler]
fn hello(Path(name): Path<String>) -> String {
    format!("hello: {}", name)
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "poem=debug");
    }
    let _connection = establish_connection();

    let m = DefaultModel::from_file("rbac_with_pattern_model.conf")
        .await
        .unwrap();
    let a = FileAdapter::new("rbac_with_pattern_policy.csv");

    let casbin_middleware = CasbinService::new(m, a).await.unwrap();

    casbin_middleware
        .write()
        .await
        .get_role_manager()
        .write()
        .matching_fn(Some(key_match2), None);

    // HOST SERVER HERE
    let app = Route::new().at("/hello/:name", get(hello));
    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(app)
        .await
}
