mod constants;
mod models;
mod browser;
mod server;

use server::SimpleServer;
use rmcp::{ServiceExt, transport::stdio};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Check for test mode
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--test" {
        return test_scraping().await;
    }

    let server = SimpleServer::new().await;

    let service = server.serve(stdio()).await?;

    service.waiting().await?;
    Ok(())
}

// Move your test function here, or into a separate tests/ folder
async fn test_scraping() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use crate::browser::BrowserManager;

    let url = "https://m3.material.io/components/search/guidelines";
    println!("Testing content scraping for: {}", url);

    // Direct usage of browser manager for testing
    let browser = BrowserManager::new().await;
    let markdown = browser.scrape_page(url).await?;

    // Show preview
    let preview_len = markdown.len().min(2000);
    println!(
        "\nMarkdown preview (first {} chars):\n{}",
        preview_len,
        &markdown[..preview_len]
    );

    // Test search
    println!("\nTesting Android search for 'docked toolbar' with max_page=2...");
    let search_result = browser.search_android_dev("docked toolbar", 2).await?;
    println!("Search result: {}", search_result);

    Ok(())
}
