# http_server
A recreation of the `http.server` python module, in rust.

## Features
* Server list of files and directories in starting directory
* Download files
* Traverse directories
* Upload files (files cannot be overridden)
* Configurable port and IP

## Build
```bash
cargo build --release
```
The binary can be found in `target/release/http_server`.

## Usage
Start the server, defaulting to 127.0.0.1:8000:
```bash
http_server
```

Start the server, with custom IP and port:
```bash
http_server -i 192.168.1.2 -p 8123
```

Note: The following examples assume the default IP and port are used.

List files
```bash
curl http://127.0.0.1:8000
curl http://127.0.0.1:8000/directory
```

Download file
```bash
curl http://127.0.0.1:8000/file.txt
curl http://127.0.0.1:8000/directory/file.txt
```

Upload file
```bash
curl --data-binary <data> http://127.0.0.1:8000/<filename>
curl --data-binary @<file> http://127.0.0.1:8000/<filename>
```
