
use scraper::{Html, Selector};
use lazy_static::lazy_static;
use readability_rust::Readability;

struct Framework {
    main_container: &'static str,
    text_content_selector: &'static str,
    exclusions: &'static [&'static str],
}

lazy_static! {
    static ref FRAMEWORKS: Vec<Framework> = vec![
        // Docusaurus v2/v3
        Framework {
            main_container: "main",
            text_content_selector: "article.markdown",
            exclusions: &[".pagination-nav", ".theme-doc-toc-desktop", ".theme-doc-sidebar-container", ".hash-link"],
        },
        // Sphinx (RTD)
        Framework {
            main_container: ".wy-nav-content",
            text_content_selector: "[itemprop='articleBody']",
            exclusions: &[".wy-nav-side", ".rst-footer-buttons", "a.headerlink"],
        },
        // Sphinx (Alabaster)
        Framework {
            main_container: "div.body",
            text_content_selector: "div.body",
            exclusions: &[".sphinxsidebar", ".link-header"],
        },
        // MkDocs (Material)
        Framework {
            main_container: ".md-main",
            text_content_selector: ".md-content__inner",
            exclusions: &[".md-sidebar", ".md-footer", ".md-header", ".md-clipboard"],
        },
        // GitBook (Legacy)
        Framework {
            main_container: ".page-inner",
            text_content_selector: ".page-inner section",
            exclusions: &[".book-summary", ".book-header"],
        },
        // GitBook (Cloud)
        Framework {
            main_container: "main",
            text_content_selector: "main",
            exclusions: &["nav", "div[class*='sidebar']"],
        },
        // Hugo (General)
        Framework {
            main_container: "main",
            text_content_selector: ".content, .post-content",
            exclusions: &["header", "footer", ".menu"],
        },
        // Nextra
        Framework {
            main_container: "main",
            text_content_selector: "main",
            exclusions: &["nav", "footer", ".nextra-sidebar-container"],
        },
        // NY Times
        Framework {
            main_container: "#site-content",
            text_content_selector: "section[data-testid='story-content']",
            exclusions: &["#site-content-skip", "[data-testid='related-links']", "[data-testid='newsletter-signup']"],
        },
        // BBC News
        Framework {
            main_container: "[role='main']",
            text_content_selector: "[data-component='text-block']",
            exclusions: &["[role='complementary']", ".bbc-1151pbn"],
        },
        // CNN
        Framework {
            main_container: ".article__content",
            text_content_selector: ".Paragraph__component",
            exclusions: &[".el-spoke-story", ".zn-body__read-more", ".ad-container"],
        },
        // Reuters
        Framework {
            main_container: "main",
            text_content_selector: "[class*='article-body__content']",
            exclusions: &["[data-testid='sidebar']", "nav", ".read-next-container"],
        },
    ];

    static ref EXCLUSION_SELECTORS: Vec<&'static str> = vec![
        "header", "footer", "nav", "aside", "[role='navigation']",
        "[role='banner']", "[role='contentinfo']", "[role='alert']",
        ".ad", ".advertisement", "[class*='google_ads']", "[id*='div-gpt-ad']",
        ".share-buttons", ".social-media", ".twitter-tweet", "div[class*='share']",
        ".modal", ".popup", ".overlay", "[class*='cookie']", "[class*='consent']",
        ".author-bio", ".timestamp", ".meta-data",
        ".no-print", ".print-only"
    ];
}

pub fn extract_content(html: &str) -> String {
    let document = Html::parse_document(html);

    // Tier 1: Framework Detection
    for framework in FRAMEWORKS.iter() {
        if let Some(content) = apply_framework_extraction(&document, framework) {
            return content;
        }
    }

    // Tier 2: Semantic Discovery
    if let Some(content) = apply_semantic_extraction(&document) {
        return content;
    }

    // Tier 3: Heuristic Fallback (using readability-rust crate, as it's already a dependency)
    if let Ok(mut parser) = Readability::new(html, None) {
        if let Some(article) = parser.parse() {
            if let Some(content) = article.content {
                return content;
            }
        }
    }

    // Fallback to returning the original HTML if no specific content can be extracted
    html.to_string()
}

fn apply_framework_extraction(document: &Html, framework: &Framework) -> Option<String> {
    let main_container_selector = Selector::parse(framework.main_container).ok()?;
    
    if document.select(&main_container_selector).next().is_some() {
        let content_selector = Selector::parse(framework.text_content_selector).ok()?;
        let mut content_html = String::new();

        for element in document.select(&content_selector) {
            content_html.push_str(&element.html());
        }

        if !content_html.is_empty() {
            let fragment = Html::parse_fragment(&content_html);
            let mut cleaned_html = String::new();

            for node in fragment.root_element().children() {
                if let Some(element_ref) = scraper::ElementRef::wrap(node) {
                    let mut a = true;
                    for selector_str in framework.exclusions.iter().chain(EXCLUSION_SELECTORS.iter()) {
                        if let Ok(selector) = Selector::parse(selector_str) {
                            if selector.matches(&element_ref) {
                                a = false;
                                break;
                            }
                        }
                    }
                    if a {
                        cleaned_html.push_str(&element_ref.html());
                    }
                } else if let Some(text) = node.value().as_text() {
                    cleaned_html.push_str(text.text.as_ref());
                }
            }
            return Some(cleaned_html);
        }
    }

    None
}

fn apply_semantic_extraction(document: &Html) -> Option<String> {
    let semantic_selectors = ["[itemprop='articleBody']", "[role='main']"];
    for selector_str in semantic_selectors.iter() {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                let fragment = Html::parse_fragment(&element.html());
                let mut cleaned_html = String::new();

                for node in fragment.root_element().children() {
                    if let Some(element_ref) = scraper::ElementRef::wrap(node) {
                        let mut a = true;
                        for selector_str in EXCLUSION_SELECTORS.iter() {
                            if let Ok(selector) = Selector::parse(selector_str) {
                                if selector.matches(&element_ref) {
                                    a = false;
                                    break;
                                }
                            }
                        }
                        if a {
                            cleaned_html.push_str(&element_ref.html());
                        }
                    } else if let Some(text) = node.value().as_text() {
                        cleaned_html.push_str(text.text.as_ref());
                    }
                }
                return Some(cleaned_html);
            }
        }
    }
    None
}

