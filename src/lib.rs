#[macro_use]
extern crate rocket;
use rocket::{http::Status, response::Redirect, State};
use shuttle_service::ShuttleRocket;
use sqlx::{Executor, FromRow, PgPool};
use url::Url;

#[shuttle_service::main]
async fn rocket(#[shuttle_aws_rds::Postgres] pool: PgPool) -> ShuttleRocket {
    pool.execute(include_str!("../Schema.sql"))
        .await
        .map_err(|_| shuttle_service::Error::Database("Failed executing Schema.sql".into()))?;

    let rocket = rocket::build()
        .manage(pool)
        .mount("/", routes![post_url, get_url]);

    Ok(rocket)
}

#[post("/", data = "<input>")]
async fn post_url(input: String, pool: &State<PgPool>) -> Result<String, Status> {
    let id = nanoid::nanoid!(8);
    let url = Url::parse(&input).map_err(|_| Status::UnprocessableEntity)?;

    sqlx::query("INSERT INTO URLS(id, redirect) VALUES ($1, $2)")
        .bind(&id)
        .bind(url.as_str())
        .execute(&**pool)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(format!("https://tinyurl.shuttleapp.rs/{id}"))
}

#[get("/<id>")]
async fn get_url(id: String, pool: &State<PgPool>) -> Result<Redirect, Status> {
    let url: StoredUrl = sqlx::query_as("SELECT * FROM URLS WHERE id = $1")
        .bind(id)
        .fetch_one(&**pool)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Redirect::permanent(url.redirect))
}

#[derive(FromRow)]
struct StoredUrl {
    #[allow(dead_code)]
    id: String,
    redirect: String,
}
