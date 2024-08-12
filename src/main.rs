use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

fn main() {
    println!("campi server started");
    let listener = TcpListener::bind("0.0.0.0:49000").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("calling handle_connection");
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    println!("start handling connection");
    let buf_reader = BufReader::new(&mut stream);
    println!("got buf_reader");
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {http_request:#?}");

    let status_line = "HTTP/1.1 200 OK";
    let contents = fs::read_to_string("./src/index.html").unwrap();
    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
