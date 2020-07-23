#![feature(test)]

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    extern crate test;

    use std::thread;
    use test::Bencher;

    #[bench]
    fn spsc_1000000_smol(b: &mut Bencher) {
        let (tx, rx) = piper::chan(1000);
        thread::spawn(move || {
            smol::block_on(async {
                let tx = tx;
                for i in 0..1u32 {
                    tx.send(i).await;
                }
            });
        });

        b.iter(move || {
            smol::block_on(async {
                for _i in 0..1_000_000u32 {
                    rx.recv().await;
                }
            });
        });
    }

    #[bench]
    fn spsc_1000000_tokio(b: &mut Bencher) {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1000);
        thread::spawn(move || {
            smol::block_on(async {
                let mut tx = tx;
                for i in 0..1u32 {
                    tx.send(i).await.unwrap();
                }
            });
        });

        b.iter(move || {
            smol::block_on(async {
                for _i in 0..1_000_000u32 {
                    let _ = rx.recv().await;
                }
            });
        });
    }

    #[bench]
    fn spsc_1000000_async_std(b: &mut Bencher) {
        let (tx, mut rx) = async_std::sync::channel(1000);
        thread::spawn(move || {
            smol::block_on(async {
                let mut tx = tx;
                for i in 0..1_000_000u32 {
                    tx.send(i).await;
                }
            });
        });

        b.iter(move || {
            smol::block_on(async {
                for _i in 0..1_000_000u32 {
                    let _ = rx.recv().await;
                }
            });
        });
    }
}
