use html2md;
use playwright_rs::{Playwright, protocol::page::{GotoOptions, WaitUntil}};
use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
};
use serde::Deserialize;

use std::sync::OnceLock;

static JS_SCRIPT: OnceLock<String> = OnceLock::new();

fn load_js_script() -> &'static str {
    JS_SCRIPT.get_or_init(|| {
        r#"
(function() {
    /**
     * Recursively extracts HTML from a root node, correctly processing open shadow DOMs,
     * filling <slot> elements, and ignoring <style> and <script> tags.
     *
     * @param {Node} root - The root node to start extracting HTML from.
     * @returns {string} The serialized HTML as a string.
     */
    function getComposedHtml(root) {
        let html = '';

        /**
         * The recursive function that traverses the DOM.
         * @param {Node} node - The current node to process.
         */
        function traverseAndBuildHtml(node) {
            switch (node.nodeType) {
                // Element node (e.g., <div>, <p>, <my-component>)
                case Node.ELEMENT_NODE:
                    const tagName = node.tagName.toLowerCase();

                    // --- NEW: IGNORE SCRIPT AND STYLE TAGS ---
                    // If the node is a style or script tag, stop processing it and its children.
                    if (tagName === 'style' || tagName === 'script') {
                        return; // Exit this branch of the traversal
                    }

                    // --- KEY LOGIC FOR <SLOT> ELEMENTS ---
                    if (tagName === 'slot') {
                        const assignedNodes = node.assignedNodes();
                        if (assignedNodes.length > 0) {
                            for (const assignedNode of assignedNodes) {
                                traverseAndBuildHtml(assignedNode);
                            }
                        } else {
                            for (const fallbackChild of node.childNodes) {
                                traverseAndBuildHtml(fallbackChild);
                            }
                        }
                        return; // Stop processing this slot element
                    }

                    // For all other elements:
                    // Reconstruct the opening tag, including its attributes.
                    const attributes = Array.from(node.attributes).map(attr => ` ${attr.name}="${attr.value}"`).join('');
                    html += `<${tagName}${attributes}>`;

                    // If the element hosts a shadow root, traverse into the shadow DOM.
                    // Otherwise, traverse its regular children (light DOM).
                    const children = node.shadowRoot ? node.shadowRoot.childNodes : node.childNodes;
                    for (const child of children) {
                        traverseAndBuildHtml(child);
                    }

                    // Add the closing tag.
                    html += `</${tagName}>`;
                    break;

                // Text node
                case Node.TEXT_NODE:
                    html += node.textContent;
                    break;

                // Comment node
                case Node.COMMENT_NODE:
                    html += `<!--${node.textContent}-->`;
                    break;
                
                // For other node types (like DocumentFragment), just process their children.
                default:
                   if (node.childNodes) {
                       for (const child of node.childNodes) {
                            traverseAndBuildHtml(child);
                        }
                   }
                   break;
            }
        }

        // Start the traversal from the children of the provided root node.
        for (const child of root.childNodes) {
            traverseAndBuildHtml(child);
        }

        return html;
    }

    // Get the full HTML by wrapping the composed content
    const htmlAttributes = Array.from(document.documentElement.attributes).map(attr => ` ${attr.name}="${attr.value}"`).join('');
    return `<html${htmlAttributes}>` + getComposedHtml(document.documentElement) + '</html>';
})()
"#.to_string()
    })
}


#[derive(Clone)]
struct SimpleServer {
    tool_router: ToolRouter<Self>,
    playwright: std::sync::Arc<tokio::sync::Mutex<Option<std::sync::Arc<Playwright>>>>,
}

impl SimpleServer {
    async fn new() -> Self {
        let playwright = Playwright::launch().await.ok().map(std::sync::Arc::new);
        Self {
            tool_router: Self::tool_router(),
            playwright: std::sync::Arc::new(tokio::sync::Mutex::new(playwright)),
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
    async fn crawl_url(
        &self,
        Parameters(request): Parameters<CrawlUrlRequest>,
    ) -> Result<CallToolResult, McpError> {
        match self.crawl_and_convert(&request.url).await {
            Ok(markdown) => Ok(CallToolResult::success(vec![Content::text(markdown)])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Error crawling URL: {}",
                e
            ))])),
        }
    }
}

impl SimpleServer {
    async fn scrape_page(
        &self,
        url: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let playwright = {
            let mut pw_lock = self.playwright.lock().await;
            if let Some(ref pw) = *pw_lock {
                pw.clone()
            } else {
                let pw = std::sync::Arc::new(Playwright::launch().await?);
                *pw_lock = Some(pw.clone());
                pw
            }
        };

        let _args = vec![
            "--no-sandbox".to_string(),
            "--disable-setuid-sandbox".to_string(),
            "--disable-dev-shm-usage".to_string(),
            "--disable-web-security".to_string(),
            "--disable-background-timer-throttling".to_string(),
            "--disable-renderer-backgrounding".to_string(),
            "--disable-backgrounding-occluded-windows".to_string(),
        ];

        let browser = playwright.webkit().launch().await?;

        let page = browser.new_page().await?;

        let response = page
            .goto(url, Some(GotoOptions::new()
                .wait_until(WaitUntil::DomContentLoaded)
                .timeout(std::time::Duration::from_secs(30))))
            .await?
            .expect("URL should return a response");
        if !response.ok() {
            return Err(format!("HTTP error: {}", response.status()).into());
        }

        // Smart waiting for SPA content: wait for Angular app to be ready
        // Check for Angular-specific indicators or content elements
        let ready_indicators = vec![
            "document.querySelector('app-post')",  // Angular component
            "document.querySelector('[ng-version]')",  // Angular app
            "document.querySelector('main, article, .post-content, .article-content')",  // Content areas
        ];

        let max_wait_ms = 10000; // 10 seconds for heavy SPAs
        let check_interval_ms = 250; // check every 250ms

        for _ in 0..(max_wait_ms / check_interval_ms) {
            let mut ready = false;

            for indicator in &ready_indicators {
                let result: String = page
                    .evaluate_value(&format!("!!({})", indicator))
                    .await
                    .unwrap_or_else(|_| "false".to_string());

                if result == "true" {
                    // Additional check: ensure the element has meaningful content
                    let content_check: String = page
                        .evaluate_value(&format!("({}).textContent.trim().length > 100", indicator))
                        .await
                        .unwrap_or_else(|_| "false".to_string());

                    if content_check == "true" {
                        ready = true;
                        break;
                    }
                }
            }

            if ready {
                // Final stabilization delay
                tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(check_interval_ms)).await;
        }

        // Get the HTML content, expanding shadow roots and handling slots, excluding style and script tags
        let html: String = page.evaluate_value(load_js_script()).await?;

        // Convert to markdown
        let markdown = html2md::parse_html(&html);

        eprintln!("DEBUG: Markdown length: {}", markdown.len());
        Ok(markdown)
    }

    async fn crawl_and_convert(
        &self,
        url: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.scrape_page(url).await
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
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Check for test mode
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--test" {
        return test_scraping().await;
    }

    // tracing_subscriber::fmt()
    //     .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
    //     .with_writer(std::io::stderr)
    //     .init();

    let server = SimpleServer::new().await;
    let service = server.serve(stdio()).await?;

    service.waiting().await?;
    Ok(())
}

async fn test_scraping() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = "https://quanghuy.web.app/post/mua-he-bat-tan/jKxaFBKpr5dSGj4eGHNg";

    println!("Testing content scraping for: {}", url);

    let server = SimpleServer::new().await;
    let markdown = server.scrape_page(url).await?;

    // Save HTML for debugging (we need to extract it from the markdown process)
    // For now, just save the markdown
    std::fs::write(".debug_browser.md", &markdown)?;

    println!("Browser markdown length: {}", markdown.len());
    println!("Contains Vietnamese text: {}", markdown.contains("một thứ rất khó nắm giữ"));
    println!("Contains title: {}", markdown.contains("Mùa hè bất tận"));

    // Show preview
    let preview_len = markdown.len().min(2000);
    println!(
        "\nMarkdown preview (first {} chars):\n{}",
        preview_len,
        &markdown[..preview_len]
    );

    Ok(())
}
