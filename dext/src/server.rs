use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use memmap::Mmap;
use std::convert::Infallible;
use std::fs::File;
use std::net::SocketAddr;

async fn handle(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
  let file = File::open("app.dext").unwrap();
  let mmap = unsafe { Mmap::map(&file).unwrap() };
  let mmap = Box::new(mmap);
  let mmap = &*Box::leak(mmap);

  Ok(Response::new(Body::from(mmap.get(0..8).unwrap())))
}

#[tokio::main]
pub async fn serve() {
  let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

  let svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });

  let server = Server::bind(&addr).serve(svc);

  if let Err(e) = server.await {
    eprintln!("server error: {}", e);
  }
}
