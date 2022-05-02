use std::collections::HashMap;
use std::fs::{File, read_dir};
use std::io::Read;

use chrono;
use log::{info};

use crate::html::file_list_to_html;
use crate::http_status;
use crate::http_status::HttpStatus;

#[derive(Debug)]
pub struct ContentType(&'static str);

#[allow(dead_code)]
const OCTET_STREAM: ContentType = ContentType("application/octet-stream");
const HTML: ContentType = ContentType("text/html;charset=utf-8");
const PLAIN_TEXT: ContentType = ContentType("text/plain");

#[derive(Debug)]
#[allow(dead_code)]
pub struct HttpRequest {
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
    fn error(status: HttpStatus, message: &str) -> HttpResponse {
        let mut body = String::from(message);
        if !body.ends_with('\n') {
            body.push('\n');
        }

        HttpResponse {status, body: body.into_bytes(), content_type: PLAIN_TEXT}
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


pub fn process_http_request(request: &str) -> HttpResponse {
    let lines: Vec<&str> = request.split("\n").collect();
    let first_line = lines[0];
    let parts: Vec<&str> = first_line.split(' ').collect();

    // Request must have 3 lines and include method, path and version  on the first list.
    if lines.len() < 3 || parts.len() < 3 {
        return HttpResponse::error(http_status::BAD_REQUEST, "Received invalid request");
    }

    let method = parts[0];
    let path = parts[1];
    // Version is in parts[2]

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

    let request = HttpRequest { path: path.to_string(), headers };
    match method {
        "GET" => process_get_request(request),
        "HEAD" => {
            let response = process_get_request(request);

            // Responses to HEAD requests don't return bodies.
            if response.status.0 == http_status::OK.0 {
                return HttpResponse { body: vec![], ..response }
            }

            return response
        }
        _ => return HttpResponse::error(http_status::NOT_IMPLEMENTED,"Unsupported method...")
    }
}

fn path_to_relative(path: String) -> String{
    vec![".", path.as_str()].join("")
}

fn process_get_request(request: HttpRequest) -> HttpResponse {
    if request.path.ends_with("/") {
        return match list_files(request.path.to_string()) {
            Ok(files) =>
                HttpResponse::success(HTML, file_list_to_html(files,request.path).into_bytes()),
            Err(e) => HttpResponse::error(http_status::INTERNAL_ERROR, e)
        };
    } else {
        // Retrieve file
        let relative_path = path_to_relative(request.path);
        let mut file = match File::open(relative_path.as_str()) {
            Ok(f) => f,
            Err(_) => return HttpResponse::error(http_status::NOT_FOUND, "Could not find file at requested path.")
        };

        let mut buffer: Vec<u8> = Vec::new();
        let bytes_read = match file.read_to_end(&mut buffer) {
            Ok(len) => len,
            Err(_) => return HttpResponse::error(http_status::INTERNAL_ERROR, "Failed to read file...")
        };

        info!("Read {} bytes from file {}", bytes_read.to_string(), relative_path.as_str());

        HttpResponse::success(PLAIN_TEXT, buffer)
    }
}

pub fn list_files(dir: String) -> Result<Vec<String>, &'static str> {
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
                path_out
            } else {
                String::new()
            }
        })
        .filter(|path| path.len() > 0)
        .collect())
}