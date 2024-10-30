use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
use campi::ThreadPool;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:9009").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    println!("start handling connection");
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    // let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
    //     println!("request index: {}", request_line);
    //     let contents = fs::read_to_string(filename).unwrap();
    //     ("HTTP/1.1 200 OK", "./src/index.html")
    // } else if request_line == "GET /test HTTP/1.1" {
    //     println!("request test: {}", request_line);
    //     let contents = fs::read_to_string(filename).unwrap();
    //     ("HTTP/1.1 200 OK", "./src/test.html")
    // } else if request_line == "GET /src/d5.png HTTP/1.1" {
    //     println!("request test: {}", request_line);
    //     let contents = fs::read(filename).unwrap();
    //     ("HTTP/1.1 200 OK", "./src/d5.png")
    // } else {
    //     println!("request unknown: {}", request_line);
    //     let contents = fs::read_to_string(filename).unwrap();
    //     ("HTTP/1.1 404 NOT FOUND", "./src/error.html")
    // };

    if request_line == "GET / HTTP/1.1" {
        // index page
        println!("request index: {}", request_line);
        let contents = fs::read_to_string("./src/index.html").unwrap();
        let status_line = "HTTP/1.1 200 OK";
        let length = contents.len();
        let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
        stream.write_all(response.as_bytes()).unwrap();
    } else if request_line == "GET /test HTTP/1.1" {
        // test page
        println!("request test: {}", request_line);
        let contents = fs::read_to_string("./src/test.html").unwrap();
        let status_line = "HTTP/1.1 200 OK";
        let length = contents.len();
        let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
        stream.write_all(response.as_bytes()).unwrap();
    } else if request_line == "GET /src/d5.png HTTP/1.1" {
        // static image from file
        println!("request test: {}", request_line);
        let contents = fs::read("./src/d5.png").unwrap();
        let status_line = "HTTP/1.1 200 OK";
        let length = contents.len();
        let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n");
        stream.write(response.as_bytes()).unwrap();
        stream.write_all(&contents).unwrap();

    } else {
        // error page
        println!("request unknown: {}", request_line);
        let contents = fs::read_to_string("./src/error.html").unwrap();
        let status_line = "HTTP/1.1 404 NOT FOUND";
        let length = contents.len();
        let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
        stream.write_all(response.as_bytes()).unwrap();
    };


}

