#![allow(unused)]
#![allow(non_camel_case_types)]
// use std::fs;
// use std::net::{TcpListener, TcpStream};
// use std::io::prelude::*;
//
// fn main() {
//
//     let listener: TcpListener = TcpListener::bind("127.0.1.1:8180").unwrap();
//
//     for stream  in listener.incoming(){
//         let stream:TcpStream = stream.unwrap();
//
//
//         handle_connection(stream);
//     }
//
// }
//
// fn handle_connection (mut stream: TcpStream){
//     let mut buffer: [u8; 1024] = [0; 1024];
//     let contents: String = fs::read_to_string("index.html").unwrap();
//     let response : String = format!("HTTP/1.1 200 OK \r\n Content-Length: {} \r\n\r\n {}",contents.len(),contents);
//     stream.read(&mut buffer).unwrap();
//     println!("request {}", String::from_utf8_lossy(&buffer[..]));
//     stream.write(response.as_bytes()).unwrap();
// r   stream.flush().unwrap();
// }
//
//

use std::net::{TcpListener, TcpStream};
use mongodb::error::Error;
use mongodb::{Client, Database, Collection};
use bson::{Bson, Document};
use std::{thread, u32};
use mongodb::bson::doc;
use tokio::task;
use std::io::{Read, Write};
use std::sync::Arc;
use serde::{Deserialize,Serialize};

use futures::stream::{StreamExt, TryStreamExt};
#[derive(Serialize,Deserialize,Debug)]
#[serde(rename_all = "lowercase")]
struct User {
    name : String,
    age : u8,
    phones :Vec<String>
}  
async fn handle_request(mut stream: TcpStream, client: Arc<Client>) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..]);

    let response = match parse_request(&request) {
        Ok((method, path, body)) => match method.as_str() {
            "GET" => handle_get(&client, &path).await,
            "POST" => handle_post(&client, &path, &body).await,
            "PUT" => handle_put(&client, &path, &body).await,
            "DELETE" => handle_delete(&client, &path).await,
            _ => HttpResponse::new(StatusCode::METHOD_NOT_ALLOWED, "Method not allowed"),
        },
        Err(_) => HttpResponse::new(StatusCode::BAD_REQUEST, "Bad request"),
    };

    stream.write_all(response.to_string().as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn parse_request(request: &str) -> Result<(String, String, String), ()> {
    let mut lines = request.lines();
    if let Some(request_line) = lines.next() {
        let mut parts = request_line.split_whitespace();
        if let (Some(method), Some(path), _) = (parts.next(), parts.next(), parts.next()) {
            let body = lines.skip(1).collect::<Vec<&str>>().join("\n");
            return Ok((method.to_owned(), path.to_owned(), body));
        }
    }
    Err(())
}

async fn handle_get(client: &Client, path: &str) -> HttpResponse {
    if path == "/users" {
        match get_users(client).await {
            Ok(users) => HttpResponse::new(StatusCode::OK, "OK"),
            Err(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR, "Failed to retrieve users"),
        }   // Perform GET operation to retrieve all users
        
    } else {
        HttpResponse::new(StatusCode::NOT_FOUND, "Resource not found")
    }
}

async fn handle_post(client: &Client, path: &str, body: &str) -> HttpResponse {
    if path == "/users" {
        // Perform POST operation to create a new user
        create_user(client, body).await;
        HttpResponse::new(StatusCode::CREATED, "User created successfully")
    } else {
        HttpResponse::new(StatusCode::NOT_FOUND, "Resource not found")
    }
}

async fn handle_put(client: &Client, path: &str, body: &str) -> HttpResponse {
    if path.starts_with("/users/") {
        // Extract user ID from path
        let user_id = path.trim_start_matches("/users/");

        // Perform PUT operation to update the user
        update_user(client, user_id, body).await;
        HttpResponse::new(StatusCode::OK, "User updated successfully")
    } else {
        HttpResponse::new(StatusCode::NOT_FOUND, "Resource not found")
    }
}

async fn handle_delete(client: &Client, path: &str) -> HttpResponse {
    if path.starts_with("/users/") {
        // Extract user ID from path
        let user_id = path.trim_start_matches("/users/");

        // Perform DELETE operation to delete the user
        delete_user(client, user_id).await;
        HttpResponse::new(StatusCode::OK, "User deleted successfully")
    } else {
        HttpResponse::new(StatusCode::NOT_FOUND, "Resource not found")
    }
}

async fn get_users(client: &Client) -> Result<(),Error> {
    // Connect to MongoDB and retrieve users
    let db: Database = client.database("mydb");
    let collection: Collection<Document> = db.collection("users");
   let filter = doc! { "age": { "$gt": 12} };
    let mut cursor = collection.find(filter, None).await.unwrap();
    let mut users = String::new();
    while cursor.advance().await? {
            let data = cursor.deserialize_current()?;
        println!("{:?}",data);
    }
    //
Ok(())
}
async fn create_user(client: &Client, body: &str) {
    // Connect to MongoDB and insert a new user
    let db: Database = client.database("mydb");
    println!("{}",body);
    let collection : Collection<Document> = db.collection("users");



match serde_json::from_str::<User>(body) {
        Ok(user) => {
            // Successfully parsed JSON data into a User struct
            println!("Parsed user: {:?}", user);
// let document = bson::to_document(&user).expect("Cannot convert to bson ");
//             // Proceed with database operations, e.g., inserting the user into MongoDB
//                      collection.insert_one(document, None).await.unwrap();
//             // Insert user data into the database
            // collection.insert_one(...);
        }
        Err(err) => {
            // Error parsing JSON
            eprintln!("Error parsing JSON: {}", err);
todo!();
        }
    }

}

async fn update_user(client: &Client, user_id: &str, body: &str) {
    // Connect to MongoDB and update the user
    let db: Database = client.database("mydb");
    let collection: Collection<Document> = db.collection("users");

    let filter = doc! { "_id": user_id };
    let update = doc! { "$set": bson::to_document(body).unwrap() };
    collection.update_one(filter, update, None).await.unwrap();
}

async fn delete_user(client: &Client, user_id: &str) {
    // Connect to MongoDB and delete the user
    let db: Database = client.database("mydb");
    let collection: Collection<Document> = db.collection("users");

    let filter = doc! { "_id": user_id };
    collection.delete_one(filter, None).await.unwrap();
}

struct HttpResponse {
    status_code: StatusCode,
    body: String,
}

impl HttpResponse {
    fn new(status_code: StatusCode, body: &str) -> Self {
        HttpResponse {
            status_code,
            body: body.to_owned(),
        }
    }

    fn to_string(&self) -> String {
        format!(
            "HTTP/1.1 {} {}\r\nContent-Length: {}\r\n\r\n{}",
            self.status_code as u16,
            self.status_code.reason_phrase(),
            self.body.len(),
            self.body
        )
    }
}

#[derive(Debug, Clone, Copy)]
enum StatusCode {
    OK = 200,
    CREATED = 201,
    BAD_REQUEST = 400,
    NOT_FOUND = 404,
    METHOD_NOT_ALLOWED = 405,
    INTERNAL_SERVER_ERROR = 500,
}

impl StatusCode {
    fn reason_phrase(&self) -> &'static str {
        match *self {
            StatusCode::OK => "OK",
            StatusCode::CREATED => "Created",
            StatusCode::BAD_REQUEST => "Bad Request",
            StatusCode::NOT_FOUND => "Not Found",
            StatusCode::METHOD_NOT_ALLOWED => "Method Not Allowed",
            StatusCode::INTERNAL_SERVER_ERROR => "Internal Server Error"
        }
    }
}

#[tokio::main]
async fn main()->Result<(), mongodb::error::Error> {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let client = Arc::new(Client::with_uri_str("mongodb://localhost:27017").await?);

    for stream in listener.incoming() {
        let client_clone = Arc::clone(&client);
        task::spawn(async move {
            let stream = stream.unwrap();
            handle_request(stream, client_clone).await;
        });
    }
    Ok(())
}
