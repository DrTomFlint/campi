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
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
        println!("request index: {}", request_line);
        ("HTTP/1.1 200 OK", "./src/index.html")
    } else {
        println!("request unknown: {}", request_line);
        ("HTTP/1.1 404 NOT FOUND", "./src/error.html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
