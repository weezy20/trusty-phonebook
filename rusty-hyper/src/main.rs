use hyper::service::make_service_fn;
use hyper::service::service_fn;
use hyper::{Body, Request, Response, Server};
use std::convert::Infallible;
use std::net::ToSocketAddrs;

static PORT: u16 = 3001;

#[tokio::main]
async fn main() {
    let addr = format!("127.0.0.1:{PORT}")
        .to_socket_addrs()
        .expect("IP address format error")
        .next()
        .unwrap();

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let make_svc = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(online))
    });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    println!("Rusty server running at localhost:{PORT}");
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn online(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new("Rusty server online and ready for business".into()))
}
