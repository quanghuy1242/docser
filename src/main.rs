mod constants;
mod models;
mod browser;
mod server;
pub mod extractor;

use server::SimpleServer;
use rmcp::{ServiceExt, transport::stdio};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server = SimpleServer::new().await;

    let service = server.serve(stdio()).await?;

    service.waiting().await?;
    Ok(())
}
