use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
use campi::ThreadPool;

use std::time::Duration;

use libcamera::{
    camera::CameraConfigurationStatus,
    camera_manager::CameraManager,
    framebuffer::AsFrameBuffer,
    framebuffer_allocator::{FrameBuffer, FrameBufferAllocator},
    framebuffer_map::MemoryMappedFrameBuffer,
    pixel_format::PixelFormat,
    properties,
    stream::StreamRole,
};

// drm-fourcc does not have MJPEG type yet, construct it from raw fourcc identifier
const PIXEL_FORMAT_MJPEG: PixelFormat = PixelFormat::new(u32::from_le_bytes([b'M', b'J', b'P', b'G']), 0);

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
    } else if request_line == "GET /src/d5.jpg HTTP/1.1" {
        // static image from file
        println!("request test: {}", request_line);
        //----------------------------------------------
        let filename = "./src/d5.jpg";

        let mgr = CameraManager::new().unwrap();
    
        let cameras = mgr.cameras();
    
        let cam = cameras.get(0).expect("No cameras found");
    
        println!(
            "Using camera: {}",
            *cam.properties().get::<properties::Model>().unwrap()
        );
    
        let mut cam = cam.acquire().expect("Unable to acquire camera");
    
        // This will generate default configuration for each specified role
        let mut cfgs = cam.generate_configuration(&[StreamRole::ViewFinder]).unwrap();
    
        // Use MJPEG format so we can write resulting frame directly into jpeg file
        cfgs.get_mut(0).unwrap().set_pixel_format(PIXEL_FORMAT_MJPEG);
    
        println!("Generated config: {:#?}", cfgs);
    
        match cfgs.validate() {
            CameraConfigurationStatus::Valid => println!("Camera configuration valid!"),
            CameraConfigurationStatus::Adjusted => println!("Camera configuration was adjusted: {:#?}", cfgs),
            CameraConfigurationStatus::Invalid => panic!("Error validating camera configuration"),
        }
    
        // Ensure that pixel format was unchanged
        assert_eq!(
            cfgs.get(0).unwrap().get_pixel_format(),
            PIXEL_FORMAT_MJPEG,
            "MJPEG is not supported by the camera"
        );
    
        cam.configure(&mut cfgs).expect("Unable to configure camera");
    
        let mut alloc = FrameBufferAllocator::new(&cam);
    
        // Allocate frame buffers for the stream
        let cfg = cfgs.get(0).unwrap();
        let stream = cfg.stream().unwrap();
        let buffers = alloc.alloc(&stream).unwrap();
        println!("Allocated {} buffers", buffers.len());
    
        // Convert FrameBuffer to MemoryMappedFrameBuffer, which allows reading &[u8]
        let buffers = buffers
            .into_iter()
            .map(|buf| MemoryMappedFrameBuffer::new(buf).unwrap())
            .collect::<Vec<_>>();
    
        // Create capture requests and attach buffers
        let mut reqs = buffers
            .into_iter()
            .map(|buf| {
                let mut req = cam.create_request(None).unwrap();
                req.add_buffer(&stream, buf).unwrap();
                req
            })
            .collect::<Vec<_>>();
    
        // Completed capture requests are returned as a callback
        let (tx, rx) = std::sync::mpsc::channel();
        cam.on_request_completed(move |req| {
            tx.send(req).unwrap();
        });
    
        cam.start(None).unwrap();
    
        // Multiple requests can be queued at a time, but for this example we just want a single frame.
        cam.queue_request(reqs.pop().unwrap()).unwrap();
    
        println!("Waiting for camera request execution");
        let req = rx.recv_timeout(Duration::from_secs(2)).expect("Camera request failed");
    
        println!("Camera request {:?} completed!", req);
        println!("Metadata: {:#?}", req.metadata());
    
        // Get framebuffer for our stream
        let framebuffer: &MemoryMappedFrameBuffer<FrameBuffer> = req.buffer(&stream).unwrap();
        println!("FrameBuffer metadata: {:#?}", framebuffer.metadata());
    
        // MJPEG format has only one data plane containing encoded jpeg data with all the headers
        let planes = framebuffer.data();
        let jpeg_data = planes.get(0).unwrap();
        // Actual JPEG-encoded data will be smalled than framebuffer size, its length can be obtained from metadata.
        let jpeg_len = framebuffer.metadata().unwrap().planes().get(0).unwrap().bytes_used as usize;
    
        std::fs::write(&filename, &jpeg_data[..jpeg_len]).unwrap();
        println!("Written {} bytes to {}", jpeg_len, &filename);
    
        //----------------------------------------------
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

