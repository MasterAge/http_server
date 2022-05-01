use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs::{DirEntry, File, read_dir};
use std::io::Read;
use chrono;
use crate::html::file_list_to_html;

use crate::http_status;
use crate::http_status::HttpStatus;

#[derive(Debug)]
pub enum Method {
    GET, POST, PUT, HEAD
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
    fn error(status: HttpStatus, message: String) -> HttpResponse {
        HttpResponse {status, body: message.into_bytes(), content_type: PLAIN_TEXT}
    }

    fn success(content_type: ContentType, body: Vec<u8>) -> HttpResponse {
        HttpResponse {status: http_status::OK, body, content_type}
    }

    pub fn serialize(&self) -> Vec<u8> {
        let header = vec![
            format!("HTTP/1.0 {} {}", self.status.0.to_string(), self.status.1),
            format!("Server: {}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
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
        "HEAD" => Method::HEAD,
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
    match request.method {
        Method::GET => process_get_request(request),
        Method::HEAD => {
            match process_get_request(request) {
                Ok(response) => Ok(HttpResponse {body: vec![], ..response}),
                Err(e) => return Err(e)
            }
        }
        _ => return Err("Unsupported method...")
    }
}

fn path_to_relative(path: String) -> String{
    vec![".", path.as_str()].join("")
}

fn process_get_request(request: HttpRequest) -> Result<HttpResponse, &'static str> {
    if request.path.ends_with("/") {
        return match list_files(request.path.to_string()) {
            Ok(files) =>
                Ok(HttpResponse::success(HTML,
                                         file_list_to_html(files,request.path).into_bytes())),
            Err(_) => Err("Failed to list files")
        };
    } else {
        // Retrieve file
        let relative_path = path_to_relative(request.path);
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

        Ok(HttpResponse::success(PLAIN_TEXT, buffer))
    }
}

fn list_files(dir: String) -> Result<Vec<String>, &'static str> {
    // List files
    let rel_path = path_to_relative(dir);
    let rel_path_len = rel_path.len();
    let entries = match read_dir(rel_path) {
        Ok(readdir) => readdir,
        Err(_) => return Err("Could not list files in directory")
    };

    Ok(entries.filter(|entry| entry.is_ok())
        .map(|entry| {
            let path = entry.unwrap().path();
            let isdir = path.is_dir();
            let path_out = path.into_os_string().into_string();
            if path_out.is_ok() {
                let mut path_out = path_out.unwrap().split_off(rel_path_len);
                if isdir {
                    // Add trailing / to directories.
                    path_out.push_str("/")
                }
                return path_out
            } else {
                String::new()
            }

        })
        .filter(|path| path.len() > 0)
        .collect())
}