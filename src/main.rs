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
use std::thread;

pub fn setup_connection_pool() -> Pool<ConnectionManager<PgConnection>> {
    match dotenv::dotenv() {
        Ok(_) => {}
        Err(_) => println!("No .env file found"),
    };

    let database_url = env::var("DATABASE_URL").expect("Need a valid database URL");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .max_size(20)
        .build(manager)
        .expect("Failed to create pool.")
}

lazy_static! {
    pub static ref DB_POOL: Pool<ConnectionManager<PgConnection>> = setup_connection_pool();
}

use std::sync::RwLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub struct CountCahe {
    count: String,
    updated_at: Instant,
}

fn get_cache() -> CountCahe {
    CountCahe {
        count: get_count(),
        updated_at: Instant::now(),
    }
}

lazy_static! {
    pub static ref COUNT_CACHE: RwLock<CountCahe> = RwLock::new(get_cache());
}

// Application logic begins here
use self::diesel::prelude::*;
mod schema;
use diesel::result::Error;

#[derive(Debug, Queryable)]
struct Device {
    id: i32,
    /// The SHA256 of the UDID, used to ensure that there are no duplicates in the DB
    /// Hashing is performed on-device
    udid: String,
    ios_version: String,
    electra_version: String,
    device_model: String,
    num_checkins: i32,
    last_checkin: i64,
}

#[derive(Deserialize)]
struct RequestDevice {
    /// The SHA256 of the UDID, used to ensure that there are no duplicates in the DB
    /// Hashing is performed on-device
    udid: String,
    ios_version: String,
    electra_version: String,
    device_model: String,
}

#[get("/")]
fn index() -> &'static str {
    "This is Collectra, the Electra statistics collector."
}

#[post(
    "/devices",
    format = "application/json",
    data = "<request_device>"
)]
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
                    num_checkins.eq(d.num_checkins + 1),
                    last_checkin.eq(SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64),
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
                    num_checkins.eq(1),
                    last_checkin.eq(SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64),
                ))
                .execute(&*connection)
                .expect("Couldn't insert");

            Custom(Status::new(201, ""), "Added device")
        }
        Err(e) => panic!(e.to_string()),
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
    let count: String;

    match COUNT_CACHE.read() {
        Ok(c) => {
            count = c.count.clone();
            if c.updated_at.elapsed() > Duration::new(5, 0) {
                thread::spawn(move || {
                    let mut inner = COUNT_CACHE
                        .write()
                        .expect("Couldn't unwrap cache for writing");
                    let new = get_cache();
                    inner.count = new.count;
                    inner.updated_at = new.updated_at;
                    println!("Busted cache!")
                });
            }
        }
        Err(_) => panic!("Can't get cache"),
    };

    let js_snippet = "window.num_devices=".to_owned() + &count + ";";

    Response::build()
        .status(Status::Ok)
        .header(AccessControlAllowOrigin::Any)
        .raw_header("Content-Type", "application/javascript")
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
