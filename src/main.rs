use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use hyper::{Body, Request, Response, Server, Client, Uri};
use hyper::server::conn::AddrStream;
use hyper::error::Error;
use hyper::service::{make_service_fn, service_fn};

async fn proxy(mut req: Request<Body>, proxy_address: Uri) -> Result<Response<Body>, Error> {
    let client = Client::new();
    *req.uri_mut() = proxy_address;
    client.request(req).await
}

#[tokio::main]
pub async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let i = Arc::new(AtomicUsize::new(0));
    let app_server_uris = Arc::new(vec![
        "http://localhost:5000".parse::<Uri>().unwrap(),
        "http://localhost:5001".parse::<Uri>().unwrap(),
    ]);

    let service = make_service_fn(|_socket: &AddrStream| {
        let uri = app_server_uris[i.fetch_add(1, Ordering::Relaxed) % app_server_uris.len()].clone();
        async move {
            Ok::<_, Error>(service_fn(move |req| {
                proxy(req, uri.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(service);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
