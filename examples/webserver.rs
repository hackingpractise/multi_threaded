use std::error::Error;
use std::io::{prelude::*, BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::{fs, thread};
use threadpool::ThreadPool;

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    let pool = ThreadPool::new(3);
    println!("Server at http://0.0.0.0:7878 or http://0.0.0.0:7878/sleep");
    for stream in listener.incoming() {
        let stream = stream?;
        pool.execute(|| handle_connection(stream));
    }
    println!("Shutting down");
    Ok(())
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", r#"hello.html"#)
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
