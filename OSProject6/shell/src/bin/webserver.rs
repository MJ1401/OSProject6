use std::{net::{TcpListener, TcpStream}, thread, io::{BufRead, BufReader, Read, Write}, fs::read, path::Path, env, sync::{Arc, Mutex}};

fn handle_client(mut stream: TcpStream, streaming_flag: bool, total: &Arc<Mutex<i32>>, valid: &Arc<Mutex<i32>>) {
    let mut s = "".to_owned();
    let mut buffer = [0; 500];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                let read_buffer = std::str::from_utf8(&buffer[..n]).unwrap();
                s.push_str(read_buffer);
                if s.contains("\r\n\r\n") || s.contains("\n\n") { // From https://doc.rust-lang.org/std/string/struct.String.html#method.contains
                    break;
                }
            }
            Err(e) => {
                println!("{}", e);
                return;
            }
        }
    }
    println!("Client IP address: {}", stream.peer_addr().unwrap());
    println!("Read {} bytes from {}", s.len(), stream.peer_addr().unwrap());
    println!("{}", s);

    let mut response = String::new();

    // Gets the file path request 
    let file_request = get_requested_file(&s);
    let cur_dir = std::env::current_dir().unwrap();
    let file_path = cur_dir.join(Path::new(file_request.trim_start_matches('/')));

    // let file_content = std::fs::read_to_string(file_path).unwrap();
    // response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: {}\r\n\r\n{}", file_content.len(), file_content).to_owned();

    // Pieces from https://doc.rust-lang.org/std/path/struct.PathBuf.html#method.canonicalize
    if !file_path.exists() {
        let mut total = total.lock().unwrap(); 
        *total += 1;
        response = format!("HTTP/1.1 404 Not Found\r\n\r\n<html><body><h1>404 Not Found</h1></body></html>");
    } else if file_path.is_dir() {
        let mut total = total.lock().unwrap();
        *total += 1;
        response = format!("HTTP/1.1 404 Not Found\r\n\r\n<html><body><h1>404 Not Found: The requested file is a directory</h1></body></html>"); 
    } else if file_path.is_file() {
        if streaming_flag {
            // This is where I would do part 2 for streaming TODO
        } else {
            let mut total = total.lock().unwrap();
            *total += 1;
            let mut valid = valid.lock().unwrap();
            *valid += 1;
            let file_content = std::fs::read_to_string(file_path).unwrap();
            response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: {}\r\n\r\n{}",
                file_content.len(), file_content).to_owned();
        }
    } else {
        let mut total = total.lock().unwrap();
        *total += 1;
        match file_path.canonicalize(){
            Ok(file) => {
                let mut valid = valid.lock().unwrap();
                *valid += 1;
                if file.starts_with(cur_dir){
                    let content = format!("<html><body><h1>Canonicalized message received</h1><p>Requested file: {}</p></body></html>", file_path.display());
                    response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: {}\r\n\r\n{}", content.len(), content);

                } else{
                    response = format!("HTTP/1.1 403 Forbidden\r\n\r\n<html><body><h1>403 Forbidden</h1></body></html>");
                }
            },
            Err(_) => {
                response = format!("HTTP/1.1 404 Not Found\r\n\r\n<html><body><h1>404 Not Found</h1></body></html>");
            }
        }
    }
    let num_total = total.lock().unwrap();
    let num_valid = valid.lock().unwrap();
    println!("Total requests: {}, Valid requests: {}", num_total, num_valid);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8887")?;
    let args: Vec<String> = env::args().collect();
    let streaming_flag = args.contains(&String::from("-s")); // Checks if there is a streaming flag
    let total_requests = Arc::new(Mutex::new(0)); // Tracks total_requests
    let valid_requests = Arc::new(Mutex::new(0)); // Tracks valid_requests
    for stream in listener.incoming() { // From Class
        let stream = stream?;
        let total_requests = Arc::clone(&total_requests);
        let valid_requests = Arc::clone(&valid_requests);
        thread::spawn (move || {
            handle_client(stream, streaming_flag, &total_requests, &valid_requests);
        });
    }
    Ok(())
}

fn get_requested_file(s: &str) -> &str {
    // Finds the file requested
    let start = s.find("GET").unwrap_or(0) + 4;
    let end = s.find("HTTP/1.1").unwrap_or(s.len()) - 1;
    s[start..end].trim()
}