extern crate hyper;

use std::io::{self, Read, Cursor};
use std::collections::HashMap;
use std::sync::Mutex;

use hyper::server::{self, Server};
use hyper::client::Client;
use hyper::status::StatusCode;
use hyper::uri::RequestUri;

const CACHING_URI: &'static str = "localhost:3000";
const LISTENING_PORT: &'static str = "20603";

fn main() {
    let map = HashMap::new();
    let mutex = Mutex::new(map);
    Server::http(format!("0.0.0.0:{}", LISTENING_PORT).as_str())
        .unwrap()
        .handle(move |req: server::Request, mut answer_res: server::Response| {
            if let hyper::Get = req.method {
                if let RequestUri::AbsolutePath(uri) = req.uri {
                    println!("Requested {}", uri);
                    let (s, status) = {
                        let mut map = mutex.lock().unwrap();
                        map.entry(uri.clone())
                            .or_insert_with(|| {
                                println!("Requesting original");
                                let client = Client::new();
                                let mut res = client.get(&format!("http://{}{}", CACHING_URI, uri))
                                    .send()
                                    .unwrap();
                                let mut s = String::new();
                                res.read_to_string(&mut s).unwrap();
                                (s, res.status)
                            })
                            .clone()
                    };
                    *answer_res.status_mut() = status;
                    io::copy(&mut Cursor::new(s), &mut answer_res.start().unwrap()).unwrap();
                    println!("Success!");
                } else {
                    println!("Non absolute path was requested: {:#?}", req.uri);
                }
            } else {
                *answer_res.status_mut() = StatusCode::MethodNotAllowed
            }
        })
        .unwrap();
}
