#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;
extern crate hyper;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_timer;

use futures::future::Future;
use futures_cpupool::CpuPool;

use tokio_timer::*;
use std::time::*;

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

struct ExampleServer {
    pool: CpuPool
}

impl ExampleServer {
    fn new(pool: CpuPool) -> ExampleServer {
        ExampleServer {
            pool: pool,
        }
    }
}

impl Service for ExampleServer {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
         match (req.method(), req.path()) {
            (&Method::Get, "/") => {
                // This just creates a response and wraps it in a future
                // that is already resolved.
                let response = Response::new().with_body("Hello, world!");
                Box::new(futures::future::ok(response))
            },
            (&Method::Get, "/slow/future") => {
                let timer = Timer::default();

                // This is a future that will resolve in 3000 ms.
                let task = timer.sleep(Duration::from_millis(3000));

                // This maps the result of the timer future into a hyper response.
                let future_response = task.then(|_| {
                    let response = Response::new().with_body(req.body());
                    Box::new(futures::future::ok(response))
                });

                Box::new(future_response)
            },
            (&Method::Get, "/slow/thread") => {
                // This spawns a thread in the CPU pool, then returns a future
                // of the result of calling the function on that thread.
                let task = self.pool.spawn_fn(|| {
                    use std::{thread, time};

                    let sleep_duration = time::Duration::from_millis(3000);

                    // This will block the thread in the CPU pool
                    thread::sleep(sleep_duration);

                    Ok(()) as Result<()>
                });

                // This maps the result from `something_slow` into a hyper response.
                let future_response = task.then(|_| {
                    let response = Response::new().with_body(req.body());
                    Box::new(futures::future::ok(response))
                });

                // Return the future chain. Note that this runs before the contents
                // of the above closures run.
                Box::new(future_response)
            },
            _ => {
                let response = Response::new().with_status(StatusCode::NotFound);
                Box::new(futures::future::ok(response))
            }
        }
    }
}

fn run() -> Result<()> {
    let addr = "127.0.0.1:3000".parse()?;
    let pool = CpuPool::new(2);
    let server = Http::new().bind(&addr, move || Ok(ExampleServer::new(pool.clone())))?;
    server.run()?;

    Ok(())
}
