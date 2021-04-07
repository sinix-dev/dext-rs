extern crate dext;

#[tokio::main]
async fn main() {
  dext::serve("app", 4000).await;
}
