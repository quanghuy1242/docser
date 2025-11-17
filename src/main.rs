use rmcp::{
    ServerHandler, ServiceExt,
    model::{ServerCapabilities, ServerInfo, CallToolResult, Content},
    transport::stdio,
    tool, tool_handler, tool_router,
    ErrorData as McpError,
    handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    schemars,
};
use serde::Deserialize;
use chromiumoxide::{Browser, BrowserConfig};
use html2md;
use futures_util::StreamExt;

#[derive(Clone)]
struct SimpleServer {
    tool_router: ToolRouter<Self>,
}

impl SimpleServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    async fn wait_for_function(&self, page: &chromiumoxide::Page, js_function: &str, timeout_ms: u64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let start_time = std::time::Instant::now();
        let polling_interval = std::time::Duration::from_millis(200); // Reduced polling frequency
        
        loop {
            if start_time.elapsed().as_millis() > timeout_ms as u128 {
                return Err("Timeout waiting for function".into());
            }
            
            // Evaluate the function directly (not wrapped in Boolean)
            match page.evaluate(js_function).await {
                Ok(result) => {
                    if let Ok(value) = result.into_value::<bool>() {
                        if value {
                            return Ok(());
                        }
                    }
                }
                Err(_) => {} // Ignore evaluation errors during polling
            }
            
            // Wait before next poll
            tokio::time::sleep(polling_interval).await;
        }
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct CrawlUrlRequest {
    url: String,
}

#[tool_router]
impl SimpleServer {
    #[tool(description = "Crawls a URL and converts the content to markdown")]
    async fn crawl_url(&self, Parameters(request): Parameters<CrawlUrlRequest>) -> Result<CallToolResult, McpError> {
        match self.crawl_and_convert(&request.url).await {
            Ok(markdown) => Ok(CallToolResult::success(vec![Content::text(markdown)])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!("Error crawling URL: {}", e))]))
        }
    }
}

impl SimpleServer {
    async fn crawl_and_convert(&self, url: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Launch browser
        let (mut browser, mut handler) = Browser::launch(BrowserConfig::builder()
            .build()?)
            .await?;

        // Spawn handler task
        tokio::spawn(async move {
            while let Some(_) = handler.next().await {}
        });

        // Create page and navigate
        let page = browser.new_page(url).await?;
        
        // Enable stealth mode to avoid detection
        page.enable_stealth_mode().await?;
        
                // Wait for page to load initially
        page.wait_for_navigation().await?;
        
        // Try to trigger content loading by scrolling
        page.evaluate(r#"
            // Quick scroll to trigger lazy loading
            window.scrollTo(0, 500);
            setTimeout(() => window.scrollTo(0, 0), 200);
        "#).await?;
        
        // Wait for Angular/Material Design content to load using polling
        let angular_loaded = self.wait_for_function(&page, r#"
            (function() {
                const hasAngular = !!document.querySelector('[ng-version]');
                if (hasAngular) {
                    const main = document.querySelector('main');
                    const hasContent = main && main.innerText.length > 50;
                    return hasContent;
                }
                return false;
            })()
        "#, 3000).await; // Reduced to 3 seconds // Increased timeout to 20 seconds
        
        if angular_loaded.is_err() {
            eprintln!("Warning: Angular content loading timed out, proceeding with available content");
        }
        
        // Additional wait for Angular to render
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
        
        // Get the HTML content
        let html = page.content().await?;
        
        // Close browser
        browser.close().await?;
        
        // Preprocess HTML to remove style and script tags
        let processed_html = if let Some(body_start) = html.find("<body") {
            eprintln!("DEBUG: Found body tag at position {}", body_start);
            if let Some(body_end) = html[body_start..].find("</body>") {
                eprintln!("DEBUG: Found body end tag at relative position {}", body_end);
                let body_content = &html[body_start..body_start + body_end + 7];
                eprintln!("DEBUG: Body content length: {}", body_content.len());
                
                // Remove style and script tags
                let without_styles = body_content.split("<style").flat_map(|part| {
                    if let Some(end) = part.find("</style>") {
                        vec![&part[end + 8..]]
                    } else {
                        vec![part]
                    }
                }).collect::<String>();
                
                let without_scripts = without_styles.split("<script").flat_map(|part| {
                    if let Some(end) = part.find("</script>") {
                        vec![&part[end + 9..]]
                    } else {
                        vec![part]
                    }
                }).collect::<String>();
                
                eprintln!("DEBUG: After removing styles/scripts: {}", without_scripts.len());
                without_scripts
            } else {
                eprintln!("DEBUG: No body end tag found");
                html
            }
        } else {
            eprintln!("DEBUG: No body tag found, using full HTML");
            html
        };
        
        eprintln!("DEBUG: Processed HTML length: {}", processed_html.len());
        eprintln!("DEBUG: Still contains @font-face: {}", processed_html.contains("@font-face"));
        
        // Convert HTML to markdown
        let markdown = html2md::parse_html(&processed_html);
        eprintln!("DEBUG: Markdown length: {}", markdown.len());
        Ok(markdown)
    }
}

#[tool_handler]
impl ServerHandler for SimpleServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for test mode
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--test" {
        return test_scraping().await;
    }

    // tracing_subscriber::fmt()
    //     .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
    //     .with_writer(std::io::stderr)
    //     .init();

    let server = SimpleServer::new();
    let service = server.serve(stdio()).await?;

    service.waiting().await?;
    Ok(())
}

async fn test_scraping() -> Result<(), Box<dyn std::error::Error>> {
    println!("DEBUG: Starting test_scraping function");
    let server = SimpleServer::new();
    let url = "https://m3.material.io/components/search/guidelines";

    println!("Testing Material Design content scraping...");
    
    // Get HTML first for debugging
    let (mut browser, mut handler) = Browser::launch(BrowserConfig::builder().build()?).await?;
    tokio::spawn(async move {
        while let Some(_) = handler.next().await {}
    });
    let page = browser.new_page(url).await?;
    page.enable_stealth_mode().await?;
    
    page.wait_for_navigation().await?;
    page.evaluate(r#"window.scrollTo(0, 500); setTimeout(() => window.scrollTo(0, 0), 200);"#).await?;
    
    let angular_loaded = server.wait_for_function(&page, r#"
        (function() {
            const hasAngular = !!document.querySelector('[ng-version]');
            if (hasAngular) {
                const main = document.querySelector('main');
                const hasContent = main && main.innerText.length > 50;
                return hasContent;
            }
            return false;
        })()
    "#, 3000).await;
    
    if angular_loaded.is_err() {
        eprintln!("Warning: Angular content loading timed out, proceeding with available content");
    }
    
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
    let html = page.content().await?;
    browser.close().await?;
    
    println!("Raw HTML length: {}", html.len());
    println!("Contains ng-version: {}", html.contains("ng-version"));
    println!("Contains main tag: {}", html.contains("<main"));
    println!("Contains Usage: {}", html.contains("Usage"));
    
    // Preprocess HTML to remove style and script tags
    let processed_html = if let Some(body_start) = html.find("<body") {
        println!("DEBUG: Found body tag at position {}", body_start);
        if let Some(body_end) = html[body_start..].find("</body>") {
            println!("DEBUG: Found body end tag at relative position {}", body_end);
            let body_content = &html[body_start..body_start + body_end + 7];
            println!("DEBUG: Body content length: {}", body_content.len());
            
            // Remove style and script tags
            let without_styles = body_content.split("<style").flat_map(|part| {
                if let Some(end) = part.find("</style>") {
                    vec![&part[end + 8..]]
                } else {
                    vec![part]
                }
            }).collect::<String>();
            
            let without_scripts = without_styles.split("<script").flat_map(|part| {
                if let Some(end) = part.find("</script>") {
                    vec![&part[end + 9..]]
                } else {
                    vec![part]
                }
            }).collect::<String>();
            
            println!("DEBUG: After removing styles/scripts: {}", without_scripts.len());
            without_scripts
        } else {
            println!("DEBUG: No body end tag found");
            html
        }
    } else {
        println!("DEBUG: No body tag found, using full HTML");
        html
    };
    
    // Now convert to markdown
    let markdown = html2md::parse_html(&processed_html);
    
    println!("Processed HTML length: {}", processed_html.len());
    println!("Still contains @font-face: {}", processed_html.contains("@font-face"));
    println!("Markdown length: {}", markdown.len());
    
    // Show the first part of the markdown
    let preview_len = markdown.len().min(2000);
    println!("\nMarkdown preview (first {} chars):\n{}", preview_len, &markdown[..preview_len]);
    
    // Check for key content
    if markdown.contains("Search") {
        println!("✅ Found search content!");
    }
    if markdown.contains("Material Design") {
        println!("✅ Found Material Design reference!");
    }
    if markdown.contains("Usage") || markdown.contains("Anatomy") {
        println!("✅ Found guideline sections!");
    }

    Ok(())
}
