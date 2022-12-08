#[macro_use]
extern crate rocket;
use rocket::{http::Status, State};
use shuttle_service::ShuttleRocket;
use sqlx::{Executor, PgPool};

#[shuttle_service::main]
async fn rocket(#[shuttle_aws_rds::Postgres] pool: PgPool) -> ShuttleRocket {
    pool.execute(include_str!("../Schema.sql"))
        .await
        .map_err(|_| shuttle_service::Error::Database("Failed executing Schema.sql".into()))?;

    let rocket = rocket::build()
        .manage(pool)
        .mount("/urls", routes![post_url]);

    Ok(rocket)
}

#[post("/", data = "<input>")]
fn post_url(input: String, pool: &State<PgPool>) -> Result<String, Status> {
    let hex_id = "nano_id::base62::<12>()";

    Ok(format!("https://shorlurl.shuttle.app/{hex_id}"))
}
