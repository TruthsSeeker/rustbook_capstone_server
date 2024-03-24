use std::{
    collections::HashMap, fs, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}, thread, time::Duration
};
use rustbook_capstone_server::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf_reader = BufReader::new(&mut stream);
    let mut request_line = String::new();
    buf_reader.read_line(&mut request_line).expect("Expected a request line");
    request_line = request_line.trim().to_string();

    let mut headers = String::new();
    loop {
        let read_result = buf_reader.read_line(&mut headers);
        if let Ok(size) = read_result {
            if size == 2 {
                break;
            }
        }
    }

    let headers: HashMap<_, _> = headers.lines().map(|header| {
        if header.len() == 0 {
            return ("ENDHEADER".to_string(), "".to_string());
        }
        let colon = header.find(':').expect("Non empty header field should contain a colon \":\"");
        let (name, value) = (header[..colon].to_string(), header[colon+1..].to_string());
        return (name, value);
    }).collect();

    // TODO: Refactor body handling
    if let Some(content_length) = headers.get("Content-Length") {
        let length = content_length.parse::<usize>().expect("Expected Content-Length to be of type int");
        let mut body = vec![0u8; length];
        buf_reader.read_exact(&mut body).expect("Body should be the length of Content-Length");

        println!("Body: {:#?}", body);
    }   


    // TODO: Refactor routing to be more generic
    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html")
    };


    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = format!(
        "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
    );

    stream.write_all(response.as_bytes()).unwrap();

}