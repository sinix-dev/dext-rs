extern crate html_minifier;
extern crate hyper;
extern crate memmap;
extern crate walkdir;

mod utils;

use fst::Streamer;
use fst::{Map, MapBuilder};
use memmap::Mmap;
use std::fs::read;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufWriter, Error};
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
        let content = get_compressed_content(entry.path()).unwrap();
        let off = utils::pack_in_u64(offset, content.len());

        archive.write_all(&content).unwrap();
        index
          .insert(&entry.path().to_str().unwrap().as_bytes(), off)
          .unwrap();

        offset += content.len();
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

fn get_compressed_content(path: &Path) -> Result<Vec<u8>, Error> {
  use deflate::write::GzEncoder;
  use deflate::Compression;
  use std::str;

  let data = read(path)?;

  let data = match path.extension().unwrap().to_str().unwrap() {
    "js" => html_minifier::js::minify(str::from_utf8(&data).unwrap()),
    "css" => html_minifier::css::minify(str::from_utf8(&data).unwrap()).unwrap(),
    "html" => html_minifier::minify(str::from_utf8(&data).unwrap()).unwrap(),
    _ => String::from_utf8(data).unwrap(),
  };

  let data = data.as_bytes();

  let mut encoder = GzEncoder::new(Vec::new(), Compression::Best);
  encoder.write_all(&data)?;
  let compressed_data = encoder.finish()?;

  Ok(compressed_data)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_build() {
    build(Path::new("../html"), Path::new("app")).unwrap();
  }
}
