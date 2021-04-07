use fst::Map;
use fst::Streamer;
use hyper::header;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use memmap::Mmap;
use std::fs::File;
use std::net::SocketAddr;
use std::path::Path;

struct Config {
  index: &'static Map<Mmap>,
  archive: &'static Mmap,
}

impl Config {
  async fn handle(self: &Self, req: Request<Body>) -> Result<Response<Body>, hyper::http::Error> {
    let path = req.uri().path().strip_prefix("/");

    let key = self.index.get(path.unwrap());
    let kb = unpack_from_u64(key.unwrap());

    let content = self.archive.get(kb.0..kb.1);

    Response::builder()
      .status(StatusCode::OK)
      .header(header::CONTENT_ENCODING, "gzip")
      .header(header::CONTENT_DISPOSITION, "inline")
      .header(header::CONTENT_TYPE, "text/html")
      .body(Body::from(content.unwrap()))
  }

  fn new(path: &Path) -> Self {
    let index = File::open(path.with_extension("index")).unwrap();
    let archive = File::open(path.with_extension("archive")).unwrap();

    let archive = unsafe { Mmap::map(&archive).unwrap() };
    let archive = Box::new(archive);
    let archive = &*Box::leak(archive);

    let index = unsafe { Mmap::map(&index).unwrap() };
    let index = Box::new(Map::<Mmap>::new(index).unwrap());
    let index = &*Box::leak(index);

    Config {
      index: index,
      archive: archive,
    }
  }
}

pub async fn serve(app: &str, port: u16) {
  let addr = SocketAddr::from(([0, 0, 0, 0], port));

  let config = Box::new(Config::new(Path::new(app)));
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
