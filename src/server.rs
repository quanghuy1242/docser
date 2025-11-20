use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};
use crate::browser::BrowserManager;
use crate::models::{CrawlUrlRequest, SearchAndroidRequest};

#[derive(Clone)]
pub struct SimpleServer {
    tool_router: ToolRouter<Self>,
    browser: BrowserManager,
}

impl SimpleServer {
    pub async fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            browser: BrowserManager::new().await,
        }
    }
}

#[tool_router]
impl SimpleServer {
    #[tool(description = "Crawls a URL and converts the content to markdown")]
    async fn crawl_url(
        &self,
        Parameters(request): Parameters<CrawlUrlRequest>,
    ) -> Result<CallToolResult, McpError> {
        match self.browser.scrape_page(&request.url).await {
            Ok(markdown) => Ok(CallToolResult::success(vec![Content::text(markdown)])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!("Error: {}", e))])),
        }
    }

    #[tool(description = "Searches Android Developers")]
    async fn search_android(
        &self,
        Parameters(request): Parameters<SearchAndroidRequest>,
    ) -> Result<CallToolResult, McpError> {
        let max_page = request.max_page.unwrap_or(1);
        match self.browser.search_android_dev(&request.query, max_page).await {
            Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!("Error: {}", e))])),
        }
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