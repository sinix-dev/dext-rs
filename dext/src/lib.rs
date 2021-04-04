extern crate hyper;
extern crate memmap;
extern crate walkdir;

mod server;
mod utils;

use fst::Streamer;
use fst::{Map, MapBuilder};
use memmap::Mmap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::io::{Error, Read};
use std::path::Path;
use walkdir::WalkDir;

pub fn build(src: &Path, target: &Path) -> Result<(), Error> {
  let dext_path = target.with_extension("dext");
  let index_path = target.with_extension("index");
  let archive_path = target.with_extension("archive");
  {
    let index = BufWriter::new(File::create(&index_path).unwrap());
    let mut dext = BufWriter::new(File::create(dext_path).unwrap());
    let mut archive = BufWriter::new(File::create(archive_path).unwrap());

    let mut index = MapBuilder::new(index).unwrap();
    let mut offset = 0;

    dext.write_all(String::from("Dext v1").as_bytes()).unwrap();

    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
      if entry.path().is_file() {
        let mut file = File::open(entry.path().canonicalize().unwrap()).unwrap();
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).unwrap();
        let off = utils::pack_in_u64(offset, contents.len());

        archive.write_all(&contents).unwrap();
        index
          .insert(&entry.path().to_str().unwrap().as_bytes(), off)
          .unwrap();

        offset += contents.len();
      }
    }

    dext.write_all(archive.buffer()).unwrap();
    index.finish().unwrap();
  }
  let mmap = unsafe { Mmap::map(&File::open(&index_path)?)? };
  let map = Map::new(mmap).unwrap();
  let mut stream = map.stream();

  while let Some((k, v)) = stream.next() {
    let kb = utils::unpack_from_u64(v);
    println!("{:?} {:} {:} {:}", k, v, kb.0, kb.1);
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_build() {
    build(Path::new("../html"), Path::new("app")).unwrap();
  }
}
