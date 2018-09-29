extern crate hyper;
extern crate serde;
extern crate serde_json;

use hyper::{Body, Request, Response, StatusCode};
use hyper::rt::{Future, Stream};

use futures::{future};

#[derive(Serialize, Deserialize, Debug)]
struct Greet {
    name: String,
    greeting: String
}

pub type BoxFutResponse = Box<Future<Item=Response<Body>, Error=hyper::Error> + Send>;

// Simple text response to a GET.
pub fn handle_root() -> BoxFutResponse {
    let response = Response::new(Body::from("Try POSTing data to /echo"));

    Box::new(future::ok(response))
}

// Streams the POST request body to the response.
pub fn handle_echo(req: Request<Body>) -> BoxFutResponse {
    let response = Response::new(Body::from(req.into_body()));

    Box::new(future::ok(response))
}

// Maps the POST request body stream to uppercase, then to the response.
pub fn handle_uppercase(req: Request<Body>) -> BoxFutResponse {
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
pub fn handle_reverse(req: Request<Body>) -> BoxFutResponse {
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

pub fn handle_json(req: Request<Body>) -> BoxFutResponse {
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
pub fn handle_not_found() -> BoxFutResponse {
    let response = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap();

    Box::new(future::ok(response))
}
