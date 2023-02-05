use postgres::{Client, Error, NoTls};
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::env;

#[macro_use]
extern crate serde_derive;

// Define the model in a struct
#[derive(Serialize, Deserialize, Debug)]
struct User {
    pub username: String,
    pub password: String,
    pub email: String,
}

// Environment variables defined in the docker compose to connect ot the DB
const DB_URL: &'static str = env!("DATABASE_URL");

fn main() {
    match set_database() {
        Ok(_) => (),
        Err(_) => ()
    }

    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_client(stream),
            Err(e) => println!("Error: {}", e)
        }
    }
}

// Database setup: change this accordingly to the model
fn set_database() -> Result<(), Error> {
    let mut client = Client::connect(DB_URL, NoTls,).unwrap();

    client.batch_execute("
        CREATE TABLE IF NOT EXISTS app_user (
            id              SERIAL PRIMARY KEY,
            username        VARCHAR UNIQUE NOT NULL,
            password        VARCHAR NOT NULL,
            email           VARCHAR UNIQUE NOT NULL
        )",
    )?;

    Ok(())
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    match stream.read(&mut buffer) {
        Ok(size) => {
            let request = String::from_utf8_lossy(&buffer[..size]);
            
            let (status_line, content) = if request.starts_with("POST /users HTTP/1.1") {
                handle_post_request(&request)
            } else if request.starts_with("GET /users HTTP/1.1") {
                ("HTTP/1.1 200 OK\r\n\r\n".to_owned(), handle_get_all_request())
            } else if request.starts_with("GET /hello HTTP/1.1") {
                ("HTTP/1.1 200 OK\r\n\r\n".to_owned(), "Hello world".to_owned())
            } else if request.starts_with("DELETE /users") {
                handle_delete_request(&request)
            } else if request.starts_with("PUT /users") {
                handle_update_request(&request)
            } else {
                println!("Request: {}", request);
                ("HTTP/1.1 404 NOT FOUND\r\n\r\n".to_owned(), "404 Not Found".to_owned())
            };

            let response = format!("{}{}", status_line, content);
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

fn handle_update_request(request: &str) -> (String, String)  {    
    
    match update_one(request) { 
        Ok(_) => (), 
        Err(_) => ()
    }
    ("HTTP/1.1 200 OK\r\n\r\n".to_owned(), format!("Update user"),)
}

fn update_one(request: &str) -> Result<(), Error>  {   
    let request_body = request.split("\r\n\r\n").last().unwrap_or("");
    let user: User = serde_json::from_str(request_body).unwrap();
    
    let mut client = Client::connect(DB_URL, NoTls,).unwrap();

    let mut request_split = request.split(" ");
    let mut request_split2 = request_split.nth(1).unwrap().split("?");
    let mut request_split3 = request_split2.nth(1).unwrap().split("=");
    let mut request_split4 = request_split3.nth(1).unwrap().split(" ");
    let id = request_split4.nth(0).unwrap();
    let id = id.parse::<i32>().unwrap();

    client.execute("UPDATE app_user SET username=$2, password=$3, email=$4 WHERE id=$1", &[&id,&user.username, &user.password, &user.email]).unwrap();
    Ok(())
}

fn handle_delete_request(request: &str) -> (String, String)  {    
    match delete_one(request) { 
        Ok(_) => (), 
        Err(_) => ()
    }

    ("HTTP/1.1 200 OK\r\n\r\n".to_owned(), format!("Deleted user"),)
}

fn delete_one(request: &str) -> Result<(), Error>  {    
    let mut client = Client::connect(DB_URL, NoTls,).unwrap();

    let mut request_split = request.split(" ");
    let mut request_split2 = request_split.nth(1).unwrap().split("?");
    let mut request_split3 = request_split2.nth(1).unwrap().split("=");
    let mut request_split4 = request_split3.nth(1).unwrap().split(" ");
    let id = request_split4.nth(0).unwrap();
    let id = id.parse::<i32>().unwrap();

    client.execute("DELETE FROM app_user WHERE id=$1", &[&id]).unwrap();
    Ok(())
}

fn handle_get_all_request() -> String {
    let mut client = Client::connect(DB_URL,NoTls,).unwrap();

    let mut users: Vec<User> = Vec::new();

    for row in client.query("SELECT username, password, email FROM app_user", &[]).unwrap() {
        let username: &str = row.get(0);
        let password: &str = row.get(1);
        let email: &str = row.get(2);

        let user = User {
            username: username.to_string(),
            password: password.to_string(),
            email: email.to_string(),
        };

        users.push(user);
    }

    let users_json = serde_json::to_string(&users).unwrap();
    users_json
}


fn handle_post_request(request: &str) -> (String, String)  {    
    let request_body = request.split("\r\n\r\n").last().unwrap_or("");

    match create_one(request_body) { 
        Ok(_) => (), 
        Err(_) => ()
    }

    ("HTTP/1.1 200 OK\r\n\r\n".to_owned(), format!("Received data: {}", request_body),)
}

fn create_one(request_body: &str) -> Result<(), Error> {
    let user: User = serde_json::from_str(request_body).unwrap();
    
    let mut client = Client::connect(DB_URL, NoTls,).unwrap();

    client.execute(
        "INSERT INTO app_user (username, password, email) VALUES ($1, $2, $3)",
        &[&user.username, &user.password, &user.email],
    )?;

    Ok(())
}