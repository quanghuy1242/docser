use playwright_rs::{Playwright, protocol::page::{GotoOptions, WaitUntil}};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::constants::load_js_script;
use crate::models::{Link, SearchResult};

#[derive(Clone)]
pub struct BrowserManager {
    instance: Arc<Mutex<Option<Arc<Playwright>>>>,
}

impl BrowserManager {
    pub async fn new() -> Self {
        let playwright = Playwright::launch().await.ok().map(Arc::new);
        Self {
            instance: Arc::new(Mutex::new(playwright)),
        }
    }

    // Helper to get or launch playwright
    async fn get_playwright(&self) -> Result<Arc<Playwright>, Box<dyn std::error::Error + Send + Sync>> {
        let mut pw_lock = self.instance.lock().await;
        if let Some(ref pw) = *pw_lock {
            Ok(pw.clone())
        } else {
            let pw = Arc::new(Playwright::launch().await?);
            *pw_lock = Some(pw.clone());
            Ok(pw)
        }
    }

    pub async fn scrape_page(&self, url: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let playwright = self.get_playwright().await?;

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
            .goto(
                url,
                Some(
                    GotoOptions::new()
                        .wait_until(WaitUntil::DomContentLoaded)
                        .timeout(std::time::Duration::from_secs(30)),
                ),
            )
            .await?
            .expect("URL should return a response");
        if !response.ok() {
            return Err(format!("HTTP error: {}", response.status()).into());
        }

        // Smart waiting for SPA content: wait for Angular app to be ready
        // Check for Angular-specific indicators or content elements
        let ready_indicators = vec![
            "document.querySelector('app-post')",     // Angular component
            "document.querySelector('[ng-version]')", // Angular app
            "document.querySelector('main, article, .post-content, .article-content')", // Content areas
        ];

        let max_wait_ms = 10000; // 10 seconds for heavy SPAs
        let check_interval_ms = 250; // check every 250ms
        let mut page_ready = false;

        for attempt in 0..(max_wait_ms / check_interval_ms) {
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
                        eprintln!(
                            "DEBUG: Page ready indicator '{}' found on attempt {}",
                            indicator,
                            attempt + 1
                        );
                        break;
                    }
                }
            }

            if ready {
                page_ready = true;
                // Final stabilization delay
                tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(check_interval_ms)).await;
        }

        if !page_ready {
            eprintln!("WARNING: Page did not become ready within timeout");
        }

        // Get the HTML content, expanding shadow roots and handling slots, excluding style and script tags
        let html: String = page.evaluate_value(load_js_script()).await?;

        // Convert to markdown
        let markdown = html2md::parse_html(&html);

        eprintln!("DEBUG: Markdown length: {}", markdown.len());
        Ok(markdown)
    }

    pub async fn search_android_dev(&self, query: &str, max_page: u32) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://developer.android.com/s/results?q={}",
            urlencoding::encode(query)
        );
        let playwright = self.get_playwright().await?;

        let browser = playwright.webkit().launch().await?;
        let page = browser.new_page().await?;

        let mut links = Vec::new();

        // Retry up to 3 times
        for attempt in 1..=3 {
            let response = page
                .goto(
                    &url,
                    Some(
                        GotoOptions::new()
                            .wait_until(WaitUntil::DomContentLoaded)
                            .timeout(std::time::Duration::from_secs(30)),
                    ),
                )
                .await?;
            if let Some(resp) = response {
                if !resp.ok() {
                    if attempt == 3 {
                        return Err(format!("HTTP error: {}", resp.status()).into());
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                }
            }

            // Wait for search results
            let ready_indicators = vec!["document.querySelector('.gs-title')"];

            let max_wait_ms = 10000;
            let check_interval_ms = 250;

            let mut ready = false;
            for _ in 0..(max_wait_ms / check_interval_ms) {
                for indicator in &ready_indicators {
                    let result: String = page
                        .evaluate_value(&format!("!!({})", indicator))
                        .await
                        .unwrap_or_else(|_| "false".to_string());

                    if result == "true" {
                        ready = true;
                        break;
                    }
                }
                if ready {
                    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(check_interval_ms)).await;
            }

            if !ready {
                eprintln!(
                    "WARNING: Search results did not load on attempt {} of 3",
                    attempt
                );
                if attempt == 3 {
                    return Err("Search results did not load after 3 attempts".into());
                }
                // Exponential backoff: 1s, 2s, 4s
                let backoff_secs = 2u64.pow(attempt - 1);
                eprintln!(
                    "INFO: Retrying after {} seconds (exponential backoff)",
                    backoff_secs
                );
                tokio::time::sleep(std::time::Duration::from_secs(backoff_secs)).await;
            }

            // Extract links with more specific selector
            let extracted_links_str: String = page
                .evaluate_value(r#"JSON.stringify(Array.from(document.querySelectorAll('.gsc-webResult.gsc-result .gs-webResult .gs-title a')).map(a => ({href: a.href, text: a.textContent.trim()})))"#)
                .await
                .unwrap_or_else(|_| "[]".to_string());

            let all_links: Vec<Link> =
                serde_json::from_str(&extracted_links_str).unwrap_or_else(|_| Vec::new());

            // Filter and dedup
            let mut seen = std::collections::HashSet::new();
            links = all_links
                .into_iter()
                .filter(|l| {
                    l.href.starts_with("https://developer.android.com/")
                        && !l.text.is_empty()
                        && seen.insert(l.href.clone())
                })
                .collect();

            // Debug: Print first few extracted links to verify
            if !links.is_empty() {
                eprintln!("DEBUG: Found {} links in total", links.len());
                for (i, link) in links.iter().take(3).enumerate() {
                    eprintln!("DEBUG[{}]: {}", i + 1, link.text);
                }
            } else {
                eprintln!("DEBUG: No links found with primary selector");
            }

            if links.is_empty() {
                eprintln!("WARNING: Primary selector found no links, trying fallback selector");
                // Fallback
                let fallback_links_str: String = page
                    .evaluate_value(r#"JSON.stringify(Array.from(document.querySelectorAll('.devsite-article a')).filter(a => a.href.startsWith('https://developer.android.com/') && a.textContent.trim()).reduce((acc, a) => { if (!acc.some(item => item.href === a.href)) acc.push({href: a.href, text: a.textContent.trim()}); return acc; }, []))"#)
                    .await
                    .unwrap_or_else(|_| "[]".to_string());
                links = serde_json::from_str(&fallback_links_str).unwrap_or_else(|_| Vec::new());

                if !links.is_empty() {
                    eprintln!("INFO: Fallback selector found {} links", links.len());
                } else {
                    eprintln!("ERROR: Both primary and fallback selectors found no links");
                }
            }

            // If max_page > 1, click next for additional pages
            for page_num in 2..=max_page {
                // Get current page number to verify navigation worked
                let current_page: String = page
                    .evaluate_value(
                        "document.querySelector('.gsc-cursor-current-page')?.textContent",
                    )
                    .await
                    .unwrap_or_else(|_| "-1".to_string());

                eprintln!(
                    "DEBUG: Currently on page {}, trying to navigate to page {}",
                    current_page, page_num
                );

                // Click the target page number
                let locator = page
                    .locator(&format!(".gsc-cursor-page:nth-child({})", page_num))
                    .await;
                if locator.click(Default::default()).await.is_ok() {
                    // Wait for results to update with specific wait conditions
                    let max_pagination_wait_ms = 10000;
                    let pagination_check_interval_ms = 250;

                    let mut page_loaded = false;
                    let mut loading_detected = true;

                    // First wait for loading to start (might already be loading)
                    for _ in 0..(2000 / pagination_check_interval_ms) {
                        let result: String = page
                            .evaluate_value("!!document.querySelector('.gsc-control-wrapper-cse.gsc-loading-fade')")
                            .await
                            .unwrap_or_else(|_| "false".to_string());

                        if result == "true" {
                            loading_detected = true;
                            break;
                        }
                        tokio::time::sleep(tokio::time::Duration::from_millis(
                            pagination_check_interval_ms,
                        ))
                        .await;
                    }

                    // If we detected loading, wait for it to complete
                    if loading_detected {
                        for _ in 0..(max_pagination_wait_ms / pagination_check_interval_ms) {
                            let result: String = page
                                .evaluate_value("!!document.querySelector('.gsc-control-wrapper-cse.gsc-loading-fade')")
                                .await
                                .unwrap_or_else(|_| "false".to_string());

                            if result == "false" {
                                // Loading has completed, verify we actually reached the target page
                                let new_page: String = page
                                    .evaluate_value(&format!("document.querySelector('.gsc-cursor-page:nth-child({})')?.textContent", page_num))
                                    .await
                                    .unwrap_or_else(|_| "??".to_string());

                                if new_page == page_num.to_string() {
                                    // Successfully navigated to the target page
                                    page_loaded = true;
                                    eprintln!("DEBUG: Successfully navigated to page {}", page_num);
                                    // Additional stabilization delay
                                    tokio::time::sleep(tokio::time::Duration::from_millis(500))
                                        .await;
                                    break;
                                } else {
                                    eprintln!(
                                        "WARNING: Expected page {} but ended up on page {}",
                                        page_num, new_page
                                    );
                                }
                            }
                            tokio::time::sleep(tokio::time::Duration::from_millis(
                                pagination_check_interval_ms,
                            ))
                            .await;
                        }
                    }

                    if !page_loaded {
                        eprintln!("WARNING: Pagination page did not load properly within timeout");
                        break;
                    }

                    // Extract more links with the same specific selector
                    let more_links_str: String = page
                        .evaluate_value(r#"JSON.stringify(Array.from(document.querySelectorAll('.gsc-webResult.gsc-result .gs-webResult .gs-title a')).map(a => ({href: a.href, text: a.textContent.trim()})))"#)
                        .await
                        .unwrap_or_else(|_| "[]".to_string());

                    let more_links: Vec<Link> =
                        serde_json::from_str(&more_links_str).unwrap_or_else(|_| Vec::new());

                    // Filter and dedup against global seen
                    let filtered_more = more_links
                        .into_iter()
                        .filter(|l| {
                            l.href.starts_with("https://developer.android.com/")
                                && !l.text.is_empty()
                                && seen.insert(l.href.clone())
                        })
                        .collect::<Vec<_>>();

                    links.extend(filtered_more);
                }
            }

            // No next_page

            // If we got links, success
            if !links.is_empty() {
                eprintln!(
                    "INFO: Successfully extracted {} links on attempt {}",
                    links.len(),
                    attempt
                );
                break;
            }

            if attempt == 3 {
                return Err("No links extracted after 3 attempts".into());
            }
            // Exponential backoff: 1s, 2s, 4s
            let backoff_secs = 2u64.pow(attempt - 1);
            eprintln!(
                "WARNING: No links extracted on attempt {} of 3, retrying after {} seconds",
                attempt, backoff_secs
            );
            tokio::time::sleep(std::time::Duration::from_secs(backoff_secs)).await;
        }

        let result = SearchResult { links };
        // TODO: Implement SQLite caching with TTL and eviction strategy
        if result.links.is_empty() {
            return Err("No links extracted".into());
        }
        Ok(serde_json::to_string(&result).unwrap())
    }
}