#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate rocket;
extern crate rocket_contrib;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate lazy_static;
extern crate r2d2;
extern crate r2d2_diesel;

use rocket::http::hyper::header::AccessControlAllowOrigin;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::response::Response;
use rocket_contrib::Json;

use diesel::pg::PgConnection;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;
use std::env;
use std::io::Cursor;

pub fn setup_connection_pool() -> Pool<ConnectionManager<PgConnection>> {
    match dotenv::dotenv() {
        Ok(_) => {}
        Err(_) => println!("No .env file found"),
    };

    let database_url = env::var("DATABASE_URL").expect("Need a valid database URL");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::new(manager).expect("Failed to create pool.")
}

lazy_static! {
    pub static ref DB_POOL: Pool<ConnectionManager<PgConnection>> = setup_connection_pool();
}

// Application logic begins here
use self::diesel::prelude::*;
mod schema;
use diesel::result::Error;

#[derive(Debug, Queryable)]
struct Device {
    id: i32,
    udid: String,
    ios_version: String,
    electra_version: String,
    device_model: String,
}

#[derive(Deserialize)]
struct RequestDevice {
    udid: String,
    ios_version: String,
    electra_version: String,
    device_model: String,
}

#[get("/")]
fn index() -> &'static str {
    "This is Collectra, the Electra statistics collector."
}

#[post("/devices", format = "application/json", data = "<request_device>")]
fn create_device(request_device: Json<RequestDevice>) -> Custom<&'static str> {
    use schema::devices::dsl::*;

    let device = request_device.0;
    let connection = DB_POOL.get().unwrap();

    let device_from_db: QueryResult<Device> = devices
        .filter(udid.eq(device.udid.clone()))
        .limit(1)
        .get_result(&*connection);

    match device_from_db {
        Ok(d) => {
            diesel::update(devices.find(d.id))
                .set((
                    electra_version.eq(device.electra_version),
                    ios_version.eq(device.ios_version),
                    device_model.eq(device.device_model),
                ))
                .get_result::<Device>(&*connection)
                .expect("Couldn't update");

            Custom(Status::new(202, ""), "Updated")
        }
        Err(Error::NotFound) => {
            diesel::insert_into(devices)
                .values((
                    udid.eq(device.udid),
                    ios_version.eq(device.ios_version),
                    electra_version.eq(device.electra_version),
                    device_model.eq(device.device_model),
                ))
                .execute(&*connection)
                .expect("Couldn't insert");

            Custom(Status::new(201, ""), "Added device")
        }
        Err(e) => panic!(e),
    }
}

#[route(OPTIONS, "/count_2")]
fn count_2_options<'a>() -> Response<'a> {
    Response::build()
        .raw_header("Access-Control-Allow-Origin", "*")
        .raw_header("Access-Control-Allow-Methods", "OPTIONS, GET")
        .finalize()
}

#[get("/count_2")]
fn get_count_2<'request>() -> Response<'request> {
    let js_snippet = "window.num_devices=".to_owned() + &get_count() + ";";

    Response::build()
        .status(Status::Ok)
        .header(AccessControlAllowOrigin::Any)
        .sized_body(Cursor::new(js_snippet))
        .finalize()
}

#[get("/count")]
fn get_count() -> String {
    use schema::devices::dsl::*;

    let num_devices: i64 = devices
        .select(diesel::dsl::count_star())
        .first(&*DB_POOL.get().unwrap())
        .expect("Can't count devices");
    num_devices.to_string()
}

fn main() {
    rocket::ignite()
        .mount(
            "/",
            routes![
                index,
                create_device,
                get_count,
                count_2_options,
                get_count_2
            ],
        )
        .launch();
}
