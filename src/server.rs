use crate::{request::Request, Result};
use phetch::gopher;
use std::{
    io::{prelude::*, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
};
use threadpool::ThreadPool;

/// This is only for running locally, so only allow a low number of
/// simultaneous connections.
const MAX_WORKERS: usize = 10;

/// Starts a web server locally.
pub fn start(listener: TcpListener) -> Result<()> {
    let pool = ThreadPool::new(MAX_WORKERS);
    let addr = listener.local_addr()?;

    println!("┌ Listening at {}", addr);
    for stream in listener.incoming() {
        let req = Request::from(addr.clone());
        let stream = stream?;
        println!("┌ Connection from {}", stream.peer_addr()?);
        pool.execute(move || {
            if let Err(e) = handle_request(stream, req) {
                eprintln!("└ {}", e);
            }
        });
    }
    Ok(())
}

/// Reads from the client and responds.
fn handle_request(mut stream: TcpStream, mut req: Request) -> Result<()> {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();
    let reader = BufReader::new(buffer.as_ref());
    if let Some(Ok(line)) = reader.lines().nth(0) {
        println!("│ {}", line);
        req.parse(&line);
        write_response(&mut stream, req)?;
    }
    Ok(())
}

/// Writes a response to a client based on a Request.
fn write_response<'a, W>(mut w: &'a W, req: Request) -> Result<()>
where
    &'a W: Write,
{
    let layout = std::fs::read_to_string("./html/layout.html")?;
    let response = match gopher::fetch_url(&req.path) {
        Ok(content) => {
            let rendered = layout
                .replace("{{content}}", &content)
                .replace("{{title}}", "🦀");
            println!("│ {}", "200 OK");
            format!("HTTP/1.1 200 OK\r\n\r\n{}", rendered)
        }
        Err(e) => {
            println!("│ path: {}", req.path);
            println!("├ {}: {}", "500 Internal Server Error", req.path);
            println!("└ {}", e);
            format!("HTTP/1.1 500 Internal Server Error\r\n\r\n{}", e)
        }
    };

    w.write(response.as_bytes()).unwrap();
    w.flush().unwrap();
    Ok(())
}
