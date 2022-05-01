mod http;
mod http_status;
mod html;

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

        match String::from_utf8(buffer[0..bytes_read].to_vec()) {
            Ok(text) => request.push_str(text.as_str()),
            Err(_) => println!("Received non-text characters, ignoring...")
        };

        if request.ends_with("\r\n\r\n") {
            let response = http::process_http_request(request.borrow());
            // println!("Received request:\n{}", request);
            println!("{in_ip} - - [{datetime}] \"{first_line}\" {code} -",
                     in_ip=stream.peer_addr().map_or("-".to_string(), |sock_addr| sock_addr.ip().to_string()),
                     datetime=chrono::offset::Local::now().format("%F %X"),
                     first_line=request.split_at(request.find("\r\n").unwrap_or(0)).0,
                     code=response.status.0);

            if stream.write(response.serialize().as_slice()).is_err() {
                println!("Failed to write to stream. Closing connection...")
            }

            return
        }
        sleep(time::Duration::from_millis(30));
    }
}

fn main() {
    let ip = "127.0.0.1";
    let port = "8000";

    println!("Starting http server on {ip}:{port} (http://{ip}:{port})", ip=ip, port=port);
    let listener = TcpListener::bind(format!("{}:{}", ip, port))
        .expect("Failed to bind to port");

    // Accept connections and process them serially.
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream),
            Err(_) => continue
        }
    }
}
