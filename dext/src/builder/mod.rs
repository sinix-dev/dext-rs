extern crate html_minifier;
extern crate hyper;
extern crate memmap;
extern crate walkdir;
extern crate lexical_sort;

use fst::{MapBuilder,Map};
use std::fs::read;
use std::fs::File;
use std::io::{Write, Result as IOResult};
use std::io::{BufWriter, Error};
use std::path::Path;
use walkdir::WalkDir;
use memmap::MmapOptions;
use lexical_sort::{StringSort, natural_lexical_cmp};

#[derive(Debug)]
struct DextBuilder {
  name: String,
  files: Vec<String>,
  buffer: Option<BufWriter<File>>,
  writer: DextWriter,
}

#[derive(Clone,Debug)]
struct DextWriter {
  content: Vec<u8>
}

impl Write for DextWriter {
  // write into the archive
  fn write(self: &mut Self, buf: &[u8]) -> IOResult<usize> {
    self.content.extend(buf);
    Ok(buf.len())
  }

  fn flush(&mut self) -> IOResult<()> {
    Ok(())
  }
}

impl DextWriter {
  fn new() -> Self {
    DextWriter { content: Vec::new() }
  }
}

impl DextBuilder {
  fn add(self: Self, content: &str) -> DextBuilder {
    println!("SELF: {}", content);
    self
  }
}

pub fn build(src: &Path, target: &Path) -> Result<(), Error> {
  let dext = DextBuilder {
    name: String::from("app"),
    files: Vec::new(),
    buffer: None,
    writer: DextWriter::new()
  };

  // dext.add("hello");

  let mut writer_clone = dext.writer.clone();
  let mut build = MapBuilder::new(writer_clone).unwrap();

  build.insert("bruce", 1).unwrap();
  build.insert("clarence", 2).unwrap();
  build.insert("stevie", 3).unwrap();

  build.finish().unwrap();

  let content = dext.writer.content;
  println!("{:?}", content);
  // let map = Map::new(content).unwrap();
  // println!("{:?}", map.len());

  return Ok(());

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
    let mut files: Vec<String> = WalkDir::new(src).into_iter().map(|e| String::from(e.unwrap().path().to_str().unwrap())).collect();
    println!("{:?}", files);
    files.string_sort_unstable(natural_lexical_cmp);
    println!("{:?}", files);

    for entry in files.into_iter() {
      let path = Path::new(&entry);
      if path.is_file() {
        println!("{:?}", path);
        let content = read(path).unwrap();
        // let content = get_compressed_content(path).unwrap();

        println!("{:?}", std::str::from_utf8(&content).unwrap());
      }
      // if entry.path().is_file() {
      //   let content = get_compressed_content(entry.path()).unwrap();
      //   let off = pack_in_u64(offset, content.len());

      //   archive.write_all(&content).unwrap();
      //   index
      //     .insert(
      //       &entry
      //         .path()
      //         .strip_prefix(src)
      //         .unwrap()
      //         .to_str()
      //         .unwrap()
      //         .as_bytes(),
      //       off,
      //     )
      //     .unwrap();

      //   offset += content.len();
      // }
    }

    dext.write_all(archive.buffer()).unwrap();
    index.finish().unwrap();
  }

  Ok(())
}

fn get_compressed_content(path: &Path) -> Result<Vec<u8>, Error> {
  use deflate::write::GzEncoder;
  use deflate::Compression;
  use std::str;

  let data = read(path)?;

  println!("{:?}", path);

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
