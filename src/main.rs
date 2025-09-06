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
    salons::table_init(&pool).await?;

    let state = MyState { pool };
    let rocket = rocket::build()
        .mount(
            "/salons",
            routes![
                salons::create,
                salons::read,
                salons::delete,
                salons::update,
                salons::replace,
                salons::list,
            ],
        )
        .manage(state);

    Ok(rocket.into())
}

#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
mod salons {

    use sqlx::{postgres::PgQueryResult, prelude::Type, Row};

    use super::*;

    #[derive(Serialize, Type, Deserialize)]
    pub enum Sector {
        buiucani,
        botanica,
    }

    #[derive(Serialize, FromRow)]
    pub struct Salons {
        pub id: String,
        pub ownerId: String,
        pub name: String,
        pub description: String,
        pub address: String,
        pub region: Sector,
        pub phone: String,
        pub email: String,
        pub createdAt: String,
        pub updatedAt: String,
    }

    #[derive(Serialize, FromRow)]
    pub struct Salon_QR {
        pub id: Option<String>,
        pub ownerId: Option<String>,
        pub name: Option<String>,
        pub description: Option<String>,
        pub address: Option<String>,
        pub region: Option<Sector>,
        pub phone: Option<String>,
        pub email: Option<String>,
        pub createdAt: Option<String>,
        pub updatedAt: Option<String>,
    }
    // the collumns corresponding to the response
    #[derive(Serialize, FromRow)]
    pub struct InsertReturnCols {
        pub name: String,
    }

    #[derive(Deserialize)]
    pub struct NewSalone {
        pub ownerId: String,
        pub name: String,
        pub description: String,
        pub address: String,
        pub region: Sector,
        pub phone: String,
        pub email: String,
    }

    pub async fn table_init(pool_ref: &PgPool) -> Result<PgQueryResult, shuttle_runtime::Error> {
        Ok(pool_ref
            .execute(
                r#"
            do $$ begin
                create type sector as enum ('botanica', 'buiucani');
            exception when duplicate_object then null;
            end $$;
            
            drop table if exists salons;
            
            create table salons (
                    id text primary key default gen_random_uuid(),
                    ownerId text not null,
                    name text not null,
                    description text,
                    address text not null,
                    region sector not null,
                    phone text,
                    email text,
                    createdAt timestamp default current_timestamp,
                    updatedAt timestamp default current_timestamp
            );
            "#,
            )
            .await
            .map_err(CustomError::new)?)
    }

    #[post("/", data = "<data>")]
    pub async fn create(
        data: Json<NewSalone>,
        state: &State<MyState>,
    ) -> Result<Json<InsertReturnCols>, BadRequest<String>> {
        let salon = sqlx::query_as(
            r#"
        INSERT INTO salons(ownerId,name,description,address,region,phone,email,createdAt,updatedAt) 
        VALUES ($1,$2,$3,$4,$5,$6,$7,NOW(),NOW()) returning name
        "#,
        )
        .bind(&data.ownerId)
        .bind(&data.name)
        .bind(&data.description)
        .bind(&data.address)
        .bind(&data.region)
        .bind(&data.phone)
        .bind(&data.email)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

        Ok(Json(salon))
    }

    #[get("/")]
    pub async fn list(state: &State<MyState>) -> Result<Json<Vec<Salon_QR>>, BadRequest<String>> {
        let salone_rows = sqlx::query("select * from salons")
            .fetch_all(&state.pool)
            .await
            .map_err(|err| BadRequest(err.to_string()))?;

        let salone_list = salone_rows
            .iter()
            .map(|row| Salon_QR {
                id: row.try_get("id").ok(),
                ownerId: row.try_get("ownerId").ok(),
                name: row.try_get("name").ok(),
                description: row.try_get("description").ok(),
                address: row.try_get("address").ok(),
                region: row.try_get("region").ok(),
                phone: row.try_get("phone").ok(),
                email: row.try_get("email").ok(),
                createdAt: row.try_get("createdAt").ok(),
                updatedAt: row.try_get("updatedAt").ok(),
            })
            .collect::<Vec<_>>();

        Ok(Json(salone_list))
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
