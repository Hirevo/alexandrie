use futures::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    smol::run(async {
        let (mut s, mut r) = piper::chan(10);
        s.send_all(&mut stream::once(async { Ok(7i32) }).boxed()).await?;

        dbg!(r.next().await);
        Ok(())
    })
}
