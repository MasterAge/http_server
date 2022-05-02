#![allow(dead_code)]

#[derive(Debug)]
pub struct HttpStatus(pub u32, pub &'static str);

pub const OK: HttpStatus = HttpStatus(200, "OK");
pub const NO_CONTENT: HttpStatus = HttpStatus(203, "No Content");

pub const BAD_REQUEST: HttpStatus = HttpStatus(400, "Bad Request");
pub const FORBIDDEN: HttpStatus = HttpStatus(403, "Forbidden");
pub const NOT_FOUND: HttpStatus = HttpStatus(404, "Not Found");

pub const INTERNAL_ERROR: HttpStatus = HttpStatus(500, "Internal Server Error");
pub const NOT_IMPLEMENTED: HttpStatus = HttpStatus(501, "Not Implemented");
