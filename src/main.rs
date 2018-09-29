extern crate hyper;

use hyper::{Body, Request, Server, Method};
use hyper::rt::{Future};
use hyper::service::service_fn;

extern crate futures;

#[macro_use]
extern crate serde_derive;

mod handlers;

fn router(req: Request<Body>) -> handlers::BoxFutResponse {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => handlers::handle_root(),
        (&Method::POST, "/echo") => handlers::handle_echo(req),
        (&Method::POST, "/echo/uppercase") => handlers::handle_uppercase(req),
        (&Method::POST, "/echo/reverse") => handlers::handle_reverse(req),
        (&Method::POST, "/echo/json") => handlers::handle_json(req),
        _ => handlers::handle_not_found()
    }
}

fn main() {
    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr)
        .serve(|| service_fn(router))
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Listening on http://{}", addr);

    hyper::rt::run(server);
}
