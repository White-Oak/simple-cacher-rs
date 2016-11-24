#![feature(proc_macro)]
#![feature(custom_attribute)]
#![feature(custom_derive)]

#[macro_use]
extern crate serde_derive;
extern crate hyper;
#[macro_use] extern crate diesel;
extern crate dotenv;
#[macro_use] extern crate diesel_codegen;

use std::io::{self, Read, Cursor};
use std::collections::HashMap;
use std::sync::Mutex;
use std::env;

use hyper::server::{self, Server};
use hyper::client::Client;
use hyper::status::StatusCode;
use hyper::uri::RequestUri;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;

pub mod schema;
pub mod models;

use models::*;

const CACHING_URI: &'static str = "api.oshs-mvd.sed.a-soft.org";
const LISTENING_PORT: &'static str = "80";

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn get_from_db_or_insert(uri: String) -> (String, StatusCode) {
    use schema::responses::dsl::*;

    let connection = establish_connection();
    let result = responses.filter(request.eq(uri.clone()))
        .first::<CachedResponseDB>(&connection);
    let cached = result.ok().map(CachedResponseDB::into).unwrap_or_else(|| {
        println!("Requesting original");
        let client = Client::new();
        let mut res = client.get(&format!("http://{}{}", CACHING_URI, uri.clone()))
            .send()
            .unwrap();
        let mut s = String::new();
        res.read_to_string(&mut s).unwrap();
        let new_response = CachedResponse{
            status: res.status.to_u16() as i16,
            request: uri,
            body: s
        };
        println!("Saving to DB!");
        diesel::insert(&new_response).into(responses)
        .execute(&connection)
        .expect("Error saving new response");
        new_response
    });
    (cached.body, StatusCode::from_u16(cached.status as u16))
}

fn main() {
    let map = HashMap::new();
    let mutex = Mutex::new(map);
    Server::http(format!("0.0.0.0:{}", LISTENING_PORT).as_str())
        .unwrap()
        .handle(move |req: server::Request, mut answer_res: server::Response| {
            if let hyper::Get = req.method {
                if let RequestUri::AbsolutePath(uri) = req.uri {
                    // println!("Requested {}", uri);
                    let (s, status) = {
                        let mut map = mutex.lock().unwrap();
                        map.entry(uri.clone())
                            .or_insert_with(|| get_from_db_or_insert(uri))
                            .clone()
                    };
                    *answer_res.status_mut() = status;
                    io::copy(&mut Cursor::new(s), &mut answer_res.start().unwrap()).unwrap();
                    // println!("Success!");
                } else {
                    println!("Non absolute path was requested: {:#?}", req.uri);
                }
            } else {
                *answer_res.status_mut() = StatusCode::MethodNotAllowed
            }
        })
        .unwrap();
}
