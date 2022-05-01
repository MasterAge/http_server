mod http;

use std::borrow::Borrow;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::sleep;
use std::time;

fn handle_connection(mut stream: TcpStream) {
    let mut request = String::new();
    loop {
        let mut buffer = [0; 1000];
        let bytes_read = match stream.read(&mut buffer) {
            Ok(len) => len,
            Err(_) => return
        };
        println!("Read bytes {} ", bytes_read.to_string());

        match String::from_utf8(buffer[0..bytes_read].to_vec()) {
            Ok(text) => request.push_str(text.as_str()),
            Err(_) => println!("Received non-text characters, ignoring...")
        };

        if request.ends_with("\r\n\r\n") || request.ends_with("\n\n") {
            println!("Received request:\n{}", request);
            match http::process_http_request(request.borrow()) {
                Ok(response) => {
                    if stream.write(response.body.as_slice()).is_err() {
                        return
                    }
                },
                Err(e) => println!("{}", e)
            }
            request.clear();
        }
        sleep(time::Duration::from_secs(1));
    }
}


fn main() {
    println!("Starting http server on 127.0.0.1:8000");
    let listener = TcpListener::bind("127.0.0.1:8000").expect("Failed to bind to port");

    // Accept connections and process them serially.
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream),
            Err(_) => continue
        }
    }
}
