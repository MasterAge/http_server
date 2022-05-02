mod http;
mod http_status;
mod html;

use std::borrow::Borrow;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::sleep;
use std::{env, time};
use std::time::Duration;

use clap::Parser;
use log::{info, warn};
use simple_logger;

const READ_DELAY_MS: u64 = 30;

fn handle_connection(mut stream: TcpStream) {
    let mut request = String::new();
    if stream.set_read_timeout(Some(Duration::from_millis(READ_DELAY_MS))).is_err() {
        warn!("Failed to set read timeout on stream");
    }

    loop {
        let mut buffer = [0; 1000];
        let bytes_read = match stream.read(&mut buffer) {
            Ok(len) => len,
            Err(_) => if request.len() > 0 { 0 } else { return }

        };

        match String::from_utf8(buffer[0..bytes_read].to_vec()) {
            Ok(text) => request.push_str(text.as_str()),
            Err(_) => println!("Received non-text characters, ignoring...")
        };

        // Assume that the first read of 0 or timeout indicates the end of the message.
        if request.len() > 0 && bytes_read == 0 {
            info!("Received request:\n{}", request);

            let response = http::process_http_request(request.borrow());

            println!("{in_ip} - - [{datetime}] \"{first_line}\" {code} -",
                     in_ip=stream.peer_addr()
                         .map_or("-".to_string(),
                                 |sock_addr| sock_addr.ip().to_string()),
                     datetime=chrono::offset::Local::now().format("%F %X"),
                     first_line=request.split_at(request.find("\r\n").unwrap_or(0)).0,
                     code=response.status.0);

            if stream.write(response.serialize().as_slice()).is_err() {
                println!("Failed to write to stream. Closing connection...")
            }

            return
        }
        sleep(time::Duration::from_millis(READ_DELAY_MS));
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Port to run on.
    #[clap(short, long, default_value = "8000")]
    port: String,

    /// IP to bind to.
    #[clap(short, long, default_value = "127.0.0.1")]
    ip: String,

    /// Directory to serve files from.
    #[clap(short, long, default_value = ".")]
    directory: String,

    /// Enable verbose logging.
    #[clap(short, long)]
    verbose: bool,
}

fn main() {

    let args = Args::parse();
    let ip: String = args.ip;
    let port: String = args.port;
    let directory: String = args.directory;

    let log_level = if args.verbose {log::Level::Info} else { log::Level::Warn };
    if simple_logger::init_with_level(log_level).is_err() {
        println!("Failed to init logging, moving on without it...");
    }

    env::set_current_dir(directory.as_str()).expect(format!("Failed to serve files from {}", directory).as_str());

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
