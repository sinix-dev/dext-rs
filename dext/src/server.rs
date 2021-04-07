use fst::Map;
use fst::Streamer;
use hyper::header;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use memmap::Mmap;
use std::fs::File;
use std::net::SocketAddr;

struct Config {
  map: &'static Map<Mmap>,
  mmap: &'static Mmap,
}

impl Config {
  async fn handle(self: &Self, req: Request<Body>) -> Result<Response<Body>, hyper::http::Error> {
    let path = req.uri().path().strip_prefix("/");
    println!("ME: {:?}", path);

    let key = self.map.get(path.unwrap());
    let kb = unpack_from_u64(key.unwrap());
    println!("ME: {:?}", kb);

    let content = self.mmap.get(kb.0..kb.1);
    println!("{:?}", content);

    Response::builder()
      .status(StatusCode::OK)
      .header(header::CONTENT_ENCODING, "gzip")
      .header(header::CONTENT_DISPOSITION, "inline")
      // .header(header::CONTENT_TYPE, "text/html")
      .body(Body::from(content.unwrap()))
  }

  fn new() -> Self {
    let file = File::open("app.archive").unwrap();
    let mmap = unsafe { Mmap::map(&file).unwrap() };
    let mmap = Box::new(mmap);
    let mmap = &*Box::leak(mmap); // mmap.get(0..8).unwrap()

    let index = unsafe { Mmap::map(&File::open("app.index").unwrap()).unwrap() };
    let map = Box::new(Map::<Mmap>::new(index).unwrap());
    let map = &*Box::leak(map);

    Config {
      map: map,
      mmap: mmap,
    }
  }
}

pub async fn serve() {
  let addr = SocketAddr::from(([0, 0, 0, 0], 4000));
  let mmap = unsafe { Mmap::map(&File::open("app.index").unwrap()).unwrap() };
  let map = Map::new(mmap).unwrap();
  let mut stream = map.stream();

  while let Some((k, v)) = stream.next() {
    let kb = unpack_from_u64(v);
    println!("{:?} {:} {:} {:}", std::str::from_utf8(k), v, kb.0, kb.1);
  }

  let config = Box::new(Config::new());
  let config: &'static Config = &*Box::leak(config);

  let make_svc = make_service_fn(move |_| async move {
    Ok::<_, hyper::http::Error>(service_fn(move |req: Request<Body>| config.handle(req)))
  });

  let server = Server::bind(&addr).serve(make_svc);

  if let Err(e) = server.await {
    eprintln!("server error: {}", e);
  }
}

fn unpack_from_u64(input: u64) -> (usize, usize) {
  (
    ((input & 0xFFFF_FFFF_0000_0000) >> 32) as usize,
    (input & 0x0000_0000_FFFF_FFFF) as usize,
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_serve() {
    serve();
  }
}
