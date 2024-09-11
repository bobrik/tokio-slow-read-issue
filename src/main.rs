use std::{io::Read, time::Instant};

use futures_util::StreamExt;
use tokio::io::AsyncReadExt;

const TOTAL_SIZE: u64 = 10_000_000_000;
const BUF_SIZE: usize = 256 * 1024;

fn main() {
    let started = Instant::now();

    copy_buf();

    eprintln!("copy buf in {}ms", started.elapsed().as_millis());

    let started = Instant::now();

    read_sync();

    eprintln!("sync read in {}ms", started.elapsed().as_millis());

    let started = Instant::now();

    read_async();

    eprintln!("async read in {}ms", started.elapsed().as_millis());
}

fn copy_buf() {
    let src = [1u8; BUF_SIZE];
    let mut dst = [0u8; BUF_SIZE];
    let mut dst_buf = tokio::io::ReadBuf::new(&mut dst);

    for _ in 0..(TOTAL_SIZE as usize / BUF_SIZE) {
        dst_buf.put_slice(&src[0..BUF_SIZE]);
        dst_buf.set_filled(0);
    }

    assert_eq!(BUF_SIZE, dst.iter().map(|e| *e as usize).sum())
}

fn read_sync() {
    let mut buf = [0; BUF_SIZE];

    let file = std::fs::File::open("/dev/zero").unwrap().take(TOTAL_SIZE);

    let mut reader = std::io::BufReader::with_capacity(BUF_SIZE, file);

    let mut read = 0;
    while let Ok(size) = reader.read(&mut buf) {
        if size == 0 {
            break;
        }

        read += size;
    }

    assert_eq!(read, TOTAL_SIZE as usize);
}

fn read_async() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let file = tokio::fs::File::open("/dev/zero")
            .await
            .unwrap()
            .take(TOTAL_SIZE);

        let mut stream = tokio_util::io::ReaderStream::with_capacity(file, BUF_SIZE);

        let mut read = 0;
        while let Some(buf) = stream.next().await {
            read += buf.unwrap().len();
        }

        assert_eq!(read, TOTAL_SIZE as usize);
    });
}
