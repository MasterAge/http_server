use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

pub struct HttpError(u32, &'static str);


#[derive(Debug)]
pub enum Method {
    GET, POST, PUT
}

#[derive(Debug)]
pub struct HttpRequest {
    method: Method,
    path: String,
    headers: HashMap<String, String>,
}

#[derive(Debug)]
pub struct HttpResponse {
    pub body: Vec<u8>,
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
        Ok(HttpResponse{body: [0; 100].to_vec()})
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

        Ok(HttpResponse{body: buffer})
    }
}
