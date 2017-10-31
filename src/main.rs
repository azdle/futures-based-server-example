#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;
extern crate hyper;
extern crate futures;

use futures::future::Future;

use hyper::{Method, StatusCode};
use hyper::server::{Http, Request, Response, Service};

mod errors {
    error_chain!{
        foreign_links {
            Io(::std::io::Error);
            NetAddrParse(::std::net::AddrParseError);
            Hyper(::hyper::Error);
        }
    }
}

use errors::*;
quick_main!(run);

struct ExampleServer;

impl Service for ExampleServer {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let mut response = Response::new();

         match (req.method(), req.path()) {
            (&Method::Get, "/") => {
                response.set_body("Hello, world!");
            },
            (&Method::Post, "/echo") => {
                response.set_body(req.body());
            },
            _ => {
                response.set_status(StatusCode::NotFound);
            },
        };

        Box::new(futures::future::ok(response))
    }
}

fn run() -> Result<()> {
    let addr = "127.0.0.1:3000".parse()?;
    let server = Http::new().bind(&addr, || Ok(ExampleServer))?;
    server.run()?;

    Ok(())
}
