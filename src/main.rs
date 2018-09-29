extern crate hyper;

use hyper::{Body, Request, Response, Server, Method, StatusCode};
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;

extern crate futures;

use futures::{future};

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

type BoxFut = Box<Future<Item=Response<Body>, Error=hyper::Error> + Send>;

#[derive(Serialize, Deserialize, Debug)]
struct Greet {
    name: String,
    greeting: String
}

// Simple text response to a GET.
fn handle_root() -> BoxFut {
    let response = Response::new(Body::from("Try POSTing data to /echo"));

    Box::new(future::ok(response))
}

// Streams the POST request body to the response.
fn handle_echo(req: Request<Body>) -> BoxFut {
    let response = Response::new(Body::from(req.into_body()));

    Box::new(future::ok(response))
}

// Maps the POST request body stream to uppercase, then to the response.
fn handle_uppercase(req: Request<Body>) -> BoxFut {
    let mapping = req
        .into_body()
        .map(|chunk| {
            chunk.iter()
                .map(|byte| byte.to_ascii_uppercase())
                .collect::<Vec<u8>>()
        });

    let response = Response::new(Body::wrap_stream(mapping));

    Box::new(future::ok(response))
}

// Buffers the POST request body, and reverses it into the response.
fn handle_reverse(req: Request<Body>) -> BoxFut {
    let concatenated = req
        .into_body()
        .concat2();

    let response = concatenated.and_then(|chunk| {
        let body = chunk.iter()
            .rev()
            .cloned()  // WHY?
            .collect::<Vec<u8>>(); // ascii only

        Ok(Response::new(Body::from(body)))
    });

    // We're directly returning a boxed future here to avoid falling
    // through to the synchronous response handler at the bottom.
    Box::new(response)
}

fn handle_json(req: Request<Body>) -> BoxFut {
    let concatenated = req
        .into_body()
        .concat2();

    let response = concatenated.and_then(|body| {
        let object: serde_json::Result<Greet> = serde_json::from_slice(&body);

        let wrapped_response = match object {
            Ok(greet) => {
                Response::builder()
                    .body(Body::from(serde_json::to_string_pretty(&greet).unwrap()))
            },
            Err(e) => {
                Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from(format!("JSON parsing error: {}!", e)))
            }
        };

        Ok(wrapped_response.unwrap())
    });

    Box::new(response)
}

// 404. Probably a better way of doing this.
fn handle_not_found() -> BoxFut {
    let response = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap();

    Box::new(future::ok(response))
}

fn router(req: Request<Body>) -> BoxFut {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => handle_root(),
        (&Method::POST, "/echo") => handle_echo(req),
        (&Method::POST, "/echo/uppercase") => handle_uppercase(req),
        (&Method::POST, "/echo/reverse") => handle_reverse(req),
        (&Method::POST, "/echo/json") => handle_json(req),
        _ => handle_not_found()
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
