use async_h1::client;
use async_std::net::TcpStream;
use http_types::{Error, Method, Request, Url};

#[async_std::main]
async fn main() -> Result<(), Error> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    let peer_addr = stream.peer_addr()?;
    println!("connecting to {}", peer_addr);

    for i in 0usize..2 {
        println!("making request {}/2", i + 1);
        let url = Url::parse(&format!("http://{}/foo", peer_addr)).unwrap();
        let req = Request::new(Method::Get, url);
        let res = client::connect(stream.clone(), req).await?;
        println!("{:?}", res);
    }
    Ok(())
}
