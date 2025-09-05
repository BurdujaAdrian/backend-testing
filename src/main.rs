use rocket::{get, routes};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/echo/<msg>")]
fn echo(msg: &str) -> &str {
    return msg;
}

#[shuttle_runtime::main]
async fn main() -> shuttle_rocket::ShuttleRocket {
    let rocket = rocket::build().mount("/", routes![index, echo]);

    Ok(rocket.into())
}
