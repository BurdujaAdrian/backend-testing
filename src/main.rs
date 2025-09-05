#[macro_use]
extern crate rocket;

use rocket::response::status::BadRequest;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use shuttle_runtime::CustomError;
use sqlx::{Executor, FromRow, PgPool};

struct MyState {
    pool: PgPool,
}

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_rocket::ShuttleRocket {
    pool.execute(include_str!("../schema.sql"))
        .await
        .map_err(CustomError::new)?;

    let state = MyState { pool };
    let rocket = rocket::build()
        .mount(
            "/salons",
            routes![
                salons::create,
                salons::read,
                salons::delete,
                salons::update,
                salons::replace
            ],
        )
        .manage(state);

    Ok(rocket.into())
}

mod salons {

    use super::*;

    #[derive(Serialize, FromRow)]
    pub struct Salons {
        pub id: i32,
        pub name: String,
    }

    #[derive(Deserialize)]
    pub struct NewSalone {
        pub name: String,
    }

    #[post("/", data = "<data>")]
    pub async fn create(
        data: Json<NewSalone>,
        state: &State<MyState>,
    ) -> Result<Json<Salons>, BadRequest<String>> {
        let salon = sqlx::query_as("INSERT INTO salons(name) VALUES ($1) RETURNING id,name")
            .bind(&data.name)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| BadRequest(e.to_string()))?;

        Ok(Json(salon))
    }

    #[get("/<id>")]
    pub async fn read(id: i32, state: &State<MyState>) -> Result<Json<Salons>, BadRequest<String>> {
        let salone = sqlx::query_as("select * from salons where id=$1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
            .map_err(|err| BadRequest(err.to_string()))?;

        Ok(Json(salone))
    }

    #[patch("/<id>", data = "<data>")]
    pub async fn update(
        id: i32,
        data: Json<NewSalone>,
        state: &State<MyState>,
    ) -> Result<Json<Salons>, BadRequest<String>> {
        let _ = id;
        let _ = data;
        let _ = state;
        todo!("Implement 1. the endpoint 2.the input data 3. The function itself")
    }

    #[put("/<id>", data = "<data>")]
    pub async fn replace(
        id: i32,
        data: Json<NewSalone>,
        state: &State<MyState>,
    ) -> Result<Json<Salons>, BadRequest<String>> {
        let _ = id;
        let _ = data;
        let _ = state;
        todo!("Implement 1. the endpoint 2.the input data 3. The function itself")
    }
    #[delete("/<id>")]
    pub async fn delete(
        id: i32,
        state: &State<MyState>,
    ) -> Result<Json<Salons>, BadRequest<String>> {
        let _ = id;
        let _ = state;
        todo!("Implement 1. the endpoint 2. The function itself")
    }
}
