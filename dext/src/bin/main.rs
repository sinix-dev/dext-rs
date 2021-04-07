extern crate dext;

#[tokio::main]
async fn main() {
  dext::serve().await;
}
