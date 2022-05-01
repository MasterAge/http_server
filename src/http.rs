use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use chrono;

use crate::http_status;
use crate::http_status::HttpStatus;

const SERVER_NAME: &str = "rust_http_server";

#[derive(Debug)]
pub enum Method {
    GET, POST, PUT
}

#[derive(Debug)]
pub struct ContentType(&'static str);

const OCTET_STREAM: ContentType = ContentType("application/octet-stream");
const HTML: ContentType = ContentType("text/html;charset=utf-8");
const PLAIN_TEXT: ContentType = ContentType("text/plain");

#[derive(Debug)]
pub struct HttpRequest {
    method: Method,
    path: String,
    headers: HashMap<String, String>,
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status: HttpStatus,
    pub content_type: ContentType,
    pub body: Vec<u8>,
}

impl HttpResponse {
    fn error(status: HttpStatus, content_type: ContentType, message: String) -> HttpResponse {
        HttpResponse {status, body: message.into_bytes(), content_type}
    }

    fn success(content_type: ContentType, body: Vec<u8>) -> HttpResponse {
        HttpResponse {status: http_status::OK, body, content_type}
    }

    pub fn serialize(&self) -> Vec<u8> {
        let header = vec![
            format!("HTTP/1.0 {} {}", self.status.0.to_string(), self.status.1),
            format!("Server: {}/{}", SERVER_NAME, env!("CARGO_PKG_VERSION")),
            format!("Date: {}", chrono::offset::Local::now()),
            format!("Content-Type: {}", self.content_type.0),
            format!("Content-Length: {}", self.body.len()),
            "\r\n".to_string(), // To ensure a blank line at bottom of headers before starting body.
        ].join("\r\n");

        let mut message = header.into_bytes();
        message.extend(self.body.iter());
        return message;
    }
}


pub fn process_http_request(request: &str) -> Result<HttpResponse, &'static str> {
    let lines: Vec<&str> = request.split("\n").collect();
    let first_line = lines[0];
    let parts: Vec<&str> = first_line.split(' ').collect();

    // Request must have 3 lines and include method, path and version  on the first list.
    if lines.len() < 3 || parts.len() < 3 {
        return Err("Received invalid request");
    }

    let method = match parts[0] {
        "GET" => Method::GET,
        "PUT" => Method::PUT,
        "POST" => Method::POST,
        _ => return Err("Received unsupported HTTP method. Ignoring...")
    };

    let mut headers = HashMap::new();

    for line in &lines[1..] {
        if line.len() == 0 {
            break;
        }

        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() == 2 {
            headers.insert(parts[0].to_string(), parts[1].trim().to_string());
        }
    }

    let request = HttpRequest { method, path: parts[1].to_string(), headers};
    println!("{:?}", request);

    match request.method {
        Method::GET => process_get_request(request),
        _ => return Err("Only GET is supported...")
    }
}

fn process_get_request(request: HttpRequest) -> Result<HttpResponse, &'static str> {
    if request.path == "/" {
        Ok(HttpResponse::success(PLAIN_TEXT, "File listing...".as_bytes().to_vec()))
    } else {
        // retrieve file
        let relative_path = vec![".", request.path.as_str()].join("");
        let mut file = match File::open(relative_path.as_str()) {
            Ok(f) => f,
            Err(_) => return Err("Could not find file at requested path.")
        };

        let mut buffer: Vec<u8> = Vec::new();
        let bytes_read = match file.read_to_end(&mut buffer) {
            Ok(len) => len,
            Err(_) => return Err("Failed to read file...")
        };

        println!("Read {} bytes from file {}", bytes_read.to_string(), relative_path.as_str());

        Ok(HttpResponse::success(OCTET_STREAM, buffer))
    }
}
