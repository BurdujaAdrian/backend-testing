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
                salons::update,
                salons::delete_by_id,
                // salons::replace,
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

    use chrono::NaiveDateTime;
    use sqlx::{postgres::PgQueryResult, prelude::Type, Row};

    use super::*;

    #[derive(Debug, Serialize, Type, Deserialize)]
    pub enum Sector {
        buiucani,
        botanica,
    }

    #[derive(Debug, Serialize, FromRow)]
    pub struct Salons {
        pub id: String,
        pub ownerId: String,
        pub name: String,
        pub description: String,
        pub address: String,
        pub region: Sector,
        pub phone: String,
        pub email: String,
        pub createdAt: NaiveDateTime,
        pub updatedAt: NaiveDateTime,
    }

    #[derive(Debug, Serialize, FromRow)]
    pub struct Salon_QR {
        pub id: Option<String>,
        pub ownerId: Option<String>,
        pub name: Option<String>,
        pub description: Option<String>,
        pub address: Option<String>,
        pub region: Option<Sector>,
        pub phone: Option<String>,
        pub email: Option<String>,
        pub createdAt: Option<NaiveDateTime>,
        pub updatedAt: Option<NaiveDateTime>,
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
                    createdAt timestamp not null default current_timestamp,
                    updatedAt timestamp not null default current_timestamp
            );
            "#,
            )
            .await
            .map_err(CustomError::new)?)
    }

    // the collumns corresponding to the response
    #[derive(Debug, Serialize, FromRow)]
    pub struct InsertReturnCols {
        pub id: String,
        pub msg: String,
    }
    #[derive(Debug, Deserialize)]
    pub struct NewSalone {
        pub ownerId: String,
        pub name: String,
        pub description: String,
        pub address: String,
        pub region: Sector,
        pub phone: String,
        pub email: String,
    }

    /* post /salons/
     * data: {/*the fields in NewSalone*/}
     *
     * returns {"id":<the id of the inserted field>,"msg":<the name of the inserted field>}
     */

    #[post("/", data = "<data>")]
    pub async fn create(
        data: Json<NewSalone>,
        state: &State<MyState>,
    ) -> Result<Json<InsertReturnCols>, BadRequest<String>> {
        let salon = sqlx::query_as(
            r#"
        INSERT INTO salons(ownerId,name,description,address,region,phone,email) 
        VALUES ($1,$2,$3,$4,$5,$6,$7) returning id, name as msg
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

    /* get /salons/
     *   returns <json list of all the entries in salons>
     */

    #[get("/")]
    pub async fn list(state: &State<MyState>) -> Result<Json<Vec<Salon_QR>>, BadRequest<String>> {
        let salone_rows = sqlx::query("select * from salons")
            .fetch_all(&state.pool)
            .await
            .map_err(|err| BadRequest(err.to_string()))?;

        let salone_list = salone_rows
            .iter()
            .map(|row| {
                rocket::debug!("{:#?}", row);
                Salon_QR {
                    id: row.try_get(0).ok(),
                    ownerId: row.try_get(1).ok(),
                    name: row.try_get(2).ok(),
                    description: row.try_get(3).ok(),
                    address: row.try_get(4).ok(),
                    region: row.try_get(5).ok(),
                    phone: row.try_get(6).ok(),
                    email: row.try_get(7).ok(),
                    createdAt: match row.try_get::<NaiveDateTime, _>(8) {
                        Ok(val) => Some(val),
                        Err(err) => unreachable!("{}", err),
                    },
                    updatedAt: match row.try_get::<NaiveDateTime, _>(9) {
                        Ok(val) => Some(val),
                        Err(err) => unreachable!("{}", err),
                    },
                    // id: row.try_get("id").ok(),
                    // ownerId: row.try_get("ownerid").ok(),
                    // name: row.try_get("name").ok(),
                    // description: row.try_get("description").ok(),
                    // address: row.try_get("address").ok(),
                    // region: row.try_get("region").ok(),
                    // phone: row.try_get("phone").ok(),
                    // email: row.try_get("email").ok(),
                    // createdAt: row.try_get("createdat").ok(),
                    // updatedAt: row.try_get("updatedat").ok(),
                }
            })
            .collect::<Vec<_>>();

        Ok(Json(salone_list))
    }

    /* get /salons/<id>
     * returns {<the fields in the sturct Salons as json key value pairs>}
     */

    #[get("/<id>")]
    pub async fn read(
        id: String,
        state: &State<MyState>,
    ) -> Result<Json<Salons>, BadRequest<String>> {
        let salone = sqlx::query_as("select * from salons where id=$1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
            .map_err(|err| BadRequest(err.to_string()))?;

        Ok(Json(salone))
    }

    #[derive(Debug, Deserialize)]
    pub struct UpdateSalone {
        pub ownerId: Option<String>,
        pub name: Option<String>,
        pub description: Option<String>,
        pub address: Option<String>,
        pub region: Option<Sector>,
        pub phone: Option<String>,
        pub email: Option<String>,
    }

    /* patch /salons/<id>
     *  data: {<the fields of the UpdateSalone struct, they all are optional>}
     *  returns {<the same as for create>}
     */

    #[patch("/<id>", data = "<data>")]
    pub async fn update(
        id: String,
        data: Json<UpdateSalone>,
        state: &State<MyState>,
    ) -> Result<Json<InsertReturnCols>, BadRequest<String>> {
        // TODO: Optimise this by using a string builder instead of needlesly
        // reasigning values
        let response = sqlx::query_as(
            r#"
        update salons 
        set name = coalesce(nullif($2,''), name)
          , ownerId = coalesce(nullif($3,''), ownerId)
          , description =coalesce(nullif($4,''),  description)
          , address =coalesce(nullif($5,''),  address)
          , phone =coalesce(nullif($6,''),  phone)
          , email =coalesce(nullif($7,''),  email)
          , updatedAt = now()
          , region = coalesce($8::sector, region)
        where id = $1
        returning id , 'update succesfull' as msg
        "#,
        )
        .bind(&id)
        .bind(&data.name)
        .bind(&data.ownerId)
        .bind(&data.description)
        .bind(&data.address)
        .bind(&data.phone)
        .bind(&data.email)
        .bind(&data.region)
        .fetch_one(&state.pool)
        .await
        .map_err(|err| BadRequest(err.to_string()))?;

        Ok(Json(response))
    }

    // TODO: Implement this if a use case appears
    //
    // #[put("/<id>", data = "<data>")]
    // pub async fn replace(
    //     id: i32,
    //     data: Json<NewSalone>,
    //     state: &State<MyState>,
    // ) -> Result<Json<Salons>, BadRequest<String>> {
    //     let _ = id;
    //     let _ = data;
    //     let _ = state;
    //     todo!("Implement 1. the endpoint 2.the input data 3. The function itself")
    // }

    #[delete("/<id>")]
    pub async fn delete_by_id(
        id: &str,
        state: &State<MyState>,
    ) -> Result<Json<Option<Salons>>, BadRequest<String>> {
        let response: Option<Salons> = sqlx::query!(
            r#"
        delete from salons where id = $1
        returning id,ownerId,name,description,region::text,address,phone,email,createdAt,updatedAt
        "#,
            id
        )
        .fetch_optional(&state.pool)
        .await
        .map_err(|err| BadRequest(err.to_string()))?
        .map(|row| Salons {
            id: row.id,
            ownerId: row.ownerid,
            name: row.name,
            description: row.description.or_else(|| Some("".to_string())).unwrap(),
            address: row.address,
            phone: row.phone.or_else(|| Some("".to_string())).unwrap(),
            email: row.email.or_else(|| Some("".to_string())).unwrap(),
            createdAt: row.createdat,
            updatedAt: row.updatedat,
            region: match row.region.unwrap().as_str() {
                "buiucani" => Sector::buiucani,
                "botanica" => Sector::botanica,
                &_ => unreachable!("everything should have been covered"),
            },
        });

        rocket::info!("===========\nShit got deleted lol \n=========");
        rocket::debug!("{:#?}", response);
        Ok(Json(response))
    }
}
