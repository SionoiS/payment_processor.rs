use std::env;
use std::net::SocketAddr;
use std::sync::Mutex;

use actix_web::{web, App, HttpServer};

use firestore_grpc_cloudrun::compute_metadata;
use firestore_grpc_cloudrun::firestore_client::FirestoreClient;

use tonic::transport::channel::Channel;

mod handlers;
mod ip_white_list_middleware;
mod models;
mod signature_middleware;

fn get_port() -> SocketAddr {
    let port = env::var("PORT").expect("Trying to read enviroment variable PORT Error: ");

    ["0.0.0.0:", &port]
        .concat()
        .parse::<SocketAddr>()
        .expect("Trying to parse SocketAddr Error: ")
}

struct MyData {
    project_id: String,
    client: FirestoreClient<Channel>,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let project_id = compute_metadata::get_project_id().await.unwrap();
    let client = firestore_grpc_cloudrun::get_client().await.unwrap();

    let data = web::Data::new(Mutex::new(MyData { project_id, client }));

    //https://docs.rs/crate/actix-web
    HttpServer::new(move || {
        App::new()
            .data(data.clone())
            .wrap(signature_middleware::VerifySignature)
            .wrap(ip_white_list_middleware::IpWhiteList)
            .service(handlers::notifications)
    })
    .bind(get_port())?
    .run()
    .await
}
