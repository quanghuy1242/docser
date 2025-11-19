# **Algorithmic Paradigms for High-Fidelity HTML Content Extraction and DOM Pattern Recognition**

## **1\. The Evolution of Document Object Model Extraction**

The programmatic isolation of "main content" from web documents—a process technically referred to as boilerplate removal or article extraction—constitutes one of the foundational challenges in information retrieval and computational linguistics. As the World Wide Web has evolved from a collection of static, semantic hypertext documents into a complex ecosystem of Single Page Applications (SPAs), client-side rendering, and ad-tech-laden layouts, the task of identifying the "informational payload" of a page has increased in complexity by orders of magnitude.  
For a web crawler operating in the contemporary landscape, simple template matching or reliance on legacy regular expressions is no longer sufficient. Modern extraction requires a hybrid methodology that synthesizes structural heuristics, statistical text analysis, and framework-specific pattern recognition. The objective of this report is to provide an exhaustive technical analysis of these methodologies, focusing on the specific requirement of extracting *raw HTML*—preserving the semantic richness of hyperlinks, emphasis, and structure—while surgically excising navigational scaffolding, footers, and dynamic injections.  
The necessity for such precision arises from the dual nature of modern web content. On one hand, the "document" (the article, the documentation entry, the blog post) remains the primary unit of consumption. On the other hand, the "application" (the headers, sidebars, infinite scroll mechanisms, and tracking pixels) surrounds this document with a dense layer of DOM nodes that offer high noise and low signal. Differentiating between these two layers requires an understanding of both the linguistic properties of content and the architectural patterns of modern frontend frameworks.  
This analysis explores the theoretical underpinnings of extraction algorithms like Mozilla’s Readability and Python’s Newspaper3k, details the specific DOM signatures of major documentation frameworks (Docusaurus, Sphinx, MkDocs) and news publications (The New York Times, BBC, Reuters), and prescribes a robust algorithmic approach for sanitization and HTML preservation.

## **2\. Theoretical Foundations of Extraction Heuristics**

To construct a reliable extractor, one must first understand the algorithmic principles that govern how machines perceive "importance" within a DOM tree. Unlike human vision, which relies on spatial layout and visual weight (rendering), extraction algorithms usually operate on the raw DOM tree, inferring importance from the statistical properties of text nodes and tag distributions.

### **2.1 The Linguistic Dichotomy: Navigational vs. Informational Text**

At the core of most heuristic extraction engines lies a linguistic observation described as a "law" of web content: there is a measurable statistical difference between text written for navigation and text written for information.  
**Navigational Text** is characterized by brevity and imperative mood. It is functionally designed to direct users elsewhere. Elements such as "Click here," "Home," "Contact Us," or "More Stories" are typically short, devoid of complex punctuation, and, crucially, heavily encapsulated in anchor tags (\<a\>).  
**Informational Text**, conversely, aims to convey complex meaning. It utilizes longer sentences, varied punctuation (commas, periods, semicolons), and a rich vocabulary. It is characterized by "stopword" density—common function words like "the," "and," "is," and "of" that are necessary for grammatical structure but often absent in terse navigational labels.  
This distinction allows algorithms to classify DOM nodes not by what they *are* (since a \<div\> can be anything), but by the statistical profile of the text they contain.

### **2.2 Link Density Analysis**

One of the most robust mathematical heuristics utilized in extraction is **Link Density**. This metric quantifies the ratio of linked text to total text within a container. The underlying assumption is that the main body of an article may contain links, but they will be sparse relative to the total word count. In contrast, a sidebar or footer menu will have a link density approaching 1.0.  
The formula for calculating the Link Density (LD) of a given node N can be expressed as:  
Where Links(N) represents the set of all descendant anchor tags of node N.  
An algorithm will typically traverse the DOM, calculating this ratio for every block-level element. A threshold is applied—often around 0.25 to 0.5 depending on the strictness of the parser. If a \<div\> contains 1000 characters of text, but 800 of those characters are inside \<a\> tags (LD \= 0.8), the node is classified as navigational clutter, such as a "Related Articles" list or a tag cloud, and is effectively pruned from the candidate list. This heuristic is particularly powerful because it is language-agnostic and resistant to changes in class names or IDs.

### **2.3 The Mozilla Readability Algorithm**

The Readability library, originally developed by Arc90 and now maintained by Mozilla for Firefox's "Reader View," represents the industry standard for heuristic extraction. Its logic is built upon a scoring system that propagates weight up the DOM tree, identifying the specific ancestor node that acts as the container for the most valuable content.

#### **2.3.1 Scoring Metrics**

Readability assigns a "content score" to paragraph elements (\<p\>) based on their textual characteristics. Points are awarded for:

* **Sentence Structure:** The presence of commas and periods, which indicate complex thought.  
* **Minimum Length:** Paragraphs with fewer than 25 characters are often ignored unless they contain significant punctuation.  
* **Class/ID Weighting:** The algorithm inspects the class and ID attributes for semantic clues. Matches against a "positive" regex (e.g., article, body, content, entry) boost the score, while matches against a "negative" regex (e.g., comment, meta, footer, hidden) reduce it.

#### **2.3.2 Ancestry Propagation**

A critical innovation in Readability is the concept of score bubbling. A highly scoring paragraph does not just retain its score; it contributes points to its parent and grandparent nodes.  
This mechanism allows the algorithm to identify the *wrapper* element. If a document has a structure where \<div id="main"\> contains fifty high-scoring \<p\> tags, the score of \#main will accumulate rapidly, mathematically distinguishing it from a \<div id="sidebar"\> which might contain only one or two low-scoring paragraphs. The node with the highest aggregate score becomes the "Candidate Root".

#### **2.3.3 Content Cleanup**

Once the Candidate Root is identified, Readability performs a cleaning pass. It does not simply return the inner HTML of the root. Instead, it iterates through the children of the root, applying the Link Density check and removing elements that look like specific UI components (e.g., sharing buttons, ad slots) based on class name heuristics. This ensures that even if the sidebar is outside the Candidate Root, inline ads or "Read More" widgets *inside* the root are still excised.

### **2.4 The Newspaper3k Algorithm and Stopword Analysis**

While Readability focuses heavily on structural and punctuation cues, the Newspaper3k library (a Python-based extractor) emphasizes **Stopword Density** as a primary signal for content detection.

#### **2.4.1 The Role of Stopwords**

In Natural Language Processing (NLP), stopwords are high-frequency words that carry little unique semantic meaning but are essential for syntax (e.g., "the", "is", "at", "which"). Newspaper3k operates on the premise that "navigational" text often lacks these function words. A menu item labeled "Sports" or "Contact" has zero stopwords. A sentence in an article, "The quick brown fox jumps over the lazy dog," contains multiple.  
Newspaper3k utilizes a localized approach. It detects the language of the document and loads a specific corpus of stopwords for that language.

* **Latin Languages:** It tokenizes text by splitting on whitespace and checks against lists like stopwords-en.txt.  
* **Non-Latin Languages:** For languages like Chinese or Arabic, where whitespace is not a reliable delimiter, it employs specialized tokenizers (e.g., jieba for Chinese, NLTK tokenizers for Arabic) to segment text before counting.

#### **2.4.2 Text Density vs. Link Density**

Newspaper3k combines stopword analysis with text density. It calculates a score for every node based on the number of stopwords it contains. A node is considered a "high-value" candidate only if it possesses a high density of text *and* that text is rich in stopwords. This effectively filters out "dense" but low-value areas like copyright footers, which might have many words ("Copyright 2025 All Rights Reserved...") but lack the syntactic variation of natural language prose.

### **2.5 Machine Learning and Visual Heuristics**

While heuristic approaches like Readability and Newspaper3k dominate due to their speed and lack of training requirements, Machine Learning (ML) approaches offer an alternative for complex cases. Tools like **Boilerpipe** treat the document as a linear sequence of text blocks rather than a hierarchical tree.  
Boilerpipe employs a classifier (often based on Decision Trees or Support Vector Machines) to label each text block as "Content" or "Boilerplate." The features used for this classification include:

* **Average Word Length:** Content often has longer words than navigation.  
* **Absolute Position:** Content is rarely at the very top or very bottom of the HTML source.  
* **Text Density Trends:** A block of text surrounded by other dense blocks is likely content (the "Cluster" heuristic).

More recent advancements utilize Deep Learning, specifically Convolutional Neural Networks (CNNs) or Long Short-Term Memory (LSTM) networks, to analyze the visual rendering of the page. These models "look" at the page via a headless browser, identifying content based on visual cues (centered alignment, font size, background color contrast) rather than just DOM structure. However, for the specific task of extracting *raw HTML* while preserving structure, heuristic DOM traversal remains superior to text-sequence classification, as ML models often flatten the output into plain text, losing the valuable hyperlinks and formatting the user requires.

## **3\. Anatomy of Boilerplate and Exclusion Strategies**

To successfully extract main content, defining what to keep is only half the battle; the algorithm must also rigorously define what to exclude. "Boilerplate" is not merely empty space; it is a sophisticated array of functional elements designed to maximize engagement, ad revenue, and compliance.

### **3.1 Taxonomies of Web Clutter**

The modern web page is cluttered with elements that are technically text but practically noise. A robust extraction algorithm must maintain a taxonomy of these elements to target them with CSS selectors or DOM pruning logic.

* **Navigation (Primary & Secondary):** Global headers, breadcrumbs, and "hamburger" menus. These are usually marked with \<nav\> or role="navigation".  
* **Solicitation Modals:** "Subscribe to our newsletter," "Allow Notifications," or "Install our App" overlays. These are often inserted dynamically via JavaScript and possess high z-indexes.  
* **Compliance Banners:** GDPR/CCPA cookie consent banners. These are ubiquitous in 2025 and often contain significant text ("We use cookies to improve your experience...") that can confuse density-based extractors.  
* **Recirculation Modules:** "You might also like," "Trending now," or "Sponsored Content" grids. These usually appear *after* the main article text but *before* the footer, often masquerading as part of the article structure.  
* **Advertising Frames:** Iframe-based programmatic ad slots (Google AdSense, Header Bidding wrappers).

### **3.2 Ad-Tech Injection Patterns and EasyList**

Advertising is a primary driver of DOM complexity. Ads are rarely static; they are injected into containers that may have randomized IDs to evade blockers. However, the *structure* of these containers often follows predictable patterns dictated by ad networks.  
The **EasyList** project provides a comprehensive crowdsourced database of CSS selectors used to block these elements. An effective crawler should incorporate a subset of EasyList's "Element Hiding Rules" to pre-clean the DOM before extraction.  
Common selector patterns from EasyList include:

* **Generic Ad Wrappers:** div\[id^="google\_ads"\], .adsbygoogle, div\[class\*="sponsored"\].  
* **Network-Specific Class Names:** .taboola-container, .outbrain-widget, .revcontent.  
* **Dimension-Based Exclusion:** Some heuristics exclude \<div\> or \<iframe\> elements that match standard Interactive Advertising Bureau (IAB) sizes (e.g., 300x250, 728x90) if the dimensions can be inferred from attributes.

Integrating these negative selectors prevents the extractor from accidentally capturing a "Sponsored Story" as the main article, a common failure mode in simple text-density algorithms.

### **3.3 CSS Obfuscation and "Class Mangling"**

A growing trend in modern web development, particularly in React-based sites (NYT, Facebook, Twitter), is **Class Mangling**. Driven by CSS-in-JS libraries like Styled Components or Emotion, class names are generated as non-semantic hashes (e.g., .css-1q2w3e, .sc-gZhRl) rather than semantic labels (.article-body).  
This renders simple class-based heuristics (looking for .content) ineffective.

* **Dynamic Hash Generation:** The class .css-1q2w3e might change to .css-9f8a7b on the next deployment.  
* **Strategy:** To counter this, extractors must rely on **structural selectors** (e.g., main \> article \> div:nth-child(2)) or **stable data attributes** (data-testid, data-component) which are preserved for automated testing purposes.  
* **The :has() Selector:** The introduction of the CSS :has() pseudo-class allows for powerful structural targeting. For example, finding a generic container that *contains* an ad script: div:has(script\[src\*="ads.google.com"\]) allows the crawler to prune the parent container without knowing its obfuscated class name.

## **4\. Framework-Specific Extraction Patterns**

While general heuristics are necessary for the "wild web," a significant portion of high-value content—specifically documentation—is hosted on a small number of predictable frameworks. Identifying the underlying framework (Static Site Generator or SPA) allows for the application of "Gold Standard" selectors that offer near-perfect precision.

### **4.1 Docusaurus (v2 and v3)**

Docusaurus, developed by Meta, is the dominant framework for open-source documentation. It renders markdown files into React components. Understanding its DOM structure requires distinguishing between its extensive sidebar/TOC apparatus and the actual markdown payload.

#### **4.1.1 Architecture and Versioning**

Docusaurus sites are Single Page Applications (SPAs). This means the initial HTML payload might be a shell, with content hydrated via JavaScript. However, Docusaurus pre-renders static HTML for SEO, meaning a raw HTTP request (without JS execution) usually yields content.

* **v2 vs. v3:** Docusaurus v3 introduced major changes to MDX compilation (MDX v3), which altered how markdown is nested. However, the *wrapping* classes remain largely consistent due to the "Infima" styling system Docusaurus uses.

#### **4.1.2 Structural Signatures**

The Docusaurus DOM is characterized by the main tag wrapper and the specific breakdown of the doc page into a sidebar, a main container, and a table of contents (TOC).

* **Main Wrapper:** The high-level container is almost always \<main\> or div\[class\*="docMainContainer"\].  
* **The Markdown Container:** This is the critical target. Docusaurus wraps the compiled Markdown content in a specific div or article.  
  * **Selector:** article is the standard container.  
  * **Refined Selector:** div.theme-doc-markdown or simply .markdown. Targeting .markdown is superior to targeting article because the article tag often includes the "Edit this page" footer and pagination links, which the user likely wants to exclude. The .markdown class specifically targets the rendered body text.

**Table 1: Docusaurus Extraction Selectors**

| Component | Primary Selector | Fallback Selector | Notes |
| :---- | :---- | :---- | :---- |
| **Main Content** | article.markdown | main article | .markdown isolates text from footer buttons. |
| **Page Title** | header h1 | article h1:first-of-type | Title is usually the first H1 in the article. |
| **Sidebar** | .theme-doc-sidebar-container | aside\[class\*="sidebar"\] | Located left of main. |
| **TOC** | .theme-doc-toc-desktop | .table-of-contents | Located right of main. |
| **Pagination** | .pagination-nav | nav.pagination-nav | "Next/Previous" links at bottom. |

**Extraction Strategy:** For Docusaurus, prioritize article.markdown. If scraping raw HTML, ensure to strip .theme-doc-toc-mobile which sometimes appears *inside* the main flow for mobile layouts. The pagination-nav is distinct and easily removable via class targeting.

### **4.2 Sphinx (Python Ecosystem)**

Sphinx is the standard documentation generator for the Python ecosystem (Django, NumPy, Linux Kernel). Unlike Docusaurus (React), Sphinx generates classic static HTML using docutils. Its structure is heavily dependent on the "Theme" applied, but the underlying HTML generated by the reStructuredText parser provides consistent hooks.

#### **4.2.1 Theme Variance**

* **Alabaster (Default):** A minimal theme. Content is typically found in div.body or div.document.  
* **ReadTheDocs (RTD):** The most ubiquitous theme. It wraps content in .wy-nav-content and uses Schema.org microdata \`\`.  
* **PyData:** Used by pandas/numpy. Content is in div\#main-content or main.

#### **4.2.2 Semantic Reliability**

Sphinx is notably semantic. It frequently applies role="main" to the content wrapper to support accessibility and internal search functions. Furthermore, the ReadTheDocs theme automatically injects itemprop="articleBody", making it one of the easiest frameworks to scrape reliably using microdata rather than CSS classes.  
**Table 2: Sphinx Extraction Selectors**

| Component | Primary Selector | Fallback Selector | Notes |
| :---- | :---- | :---- | :---- |
| **Main Content** | \`\` | div.body, div.document | Microdata is the most robust method here. |
| **RTD Specific** | .wy-nav-content | section\[data-toggle="wy-nav-shift"\] | Specific to ReadTheDocs theme structure. |
| **Sidebar** | .sphinxsidebar | .wy-nav-side | Contains global navigation tree. |
| **Permalinks** | a.headerlink |  | The "¶" symbols next to headers. Must be removed. |
| **Search Box** | div\[role="search"\] | .search | . |

**Extraction Strategy:** When scraping Sphinx, one must actively remove a.headerlink. These are anchor links (often rendered as ¶) attached to every heading. If preserved in the raw HTML extraction, they clutter the output. The itemprop="articleBody" selector should be the first priority, followed by \[role="main"\].

### **4.3 MkDocs (Material Theme)**

MkDocs is favored in the DevOps and Go communities. The "Material for MkDocs" theme is so prevalent it effectively defines the standard for MkDocs structure.

#### **4.3.1 Structural Consistency**

The Material theme is strictly structured using BEM (Block Element Modifier) naming conventions, making it highly scrape-friendly. The content is consistently located within .md-content.

* **Shadow DOM:** Some advanced configurations of Material MkDocs use Shadow DOM for components like "Copy to Clipboard" buttons or instant loading. Standard CSS selectors in a library like BeautifulSoup will not penetrate Shadow DOM, but since the content itself is usually Light DOM, this is rarely an issue for the text body.

**Table 3: MkDocs Extraction Selectors**

| Component | Primary Selector | Fallback Selector | Notes |
| :---- | :---- | :---- | :---- |
| **Main Content** | .md-content\_\_inner | .md-content | The inner class removes wrapper padding. |
| **Sidebar (Nav)** | .md-sidebar--primary | .md-sidebar | Left-side navigation. |
| **TOC** | .md-sidebar--secondary | .md-nav--secondary | Right-side TOC. |
| **Header** | .md-header | header.md-header | Top bar. |
| **Footer** | .md-footer | footer.md-footer | Next/Prev links and copyright. |

**Extraction Strategy:** Target .md-content\_\_inner. This element contains the pure rendered Markdown HTML. Unlike Docusaurus, Material MkDocs places the h1 title *inside* this container. Be wary of "admonitions" (callout blocks like \!\!\! note), which are rendered as div.admonition. These are valuable content and should be preserved, unlike sidebars.

### **4.4 GitBook**

GitBook exists in two distinct eras: the "Legacy" static site generator (Node.js based) and the modern "Cloud" GitBook (React-based SaaS).

* **Legacy GitBook:** Uses classes like .page-inner and .book-body. It is static and easy to parse.  
* **Cloud GitBook:** Uses dynamic class names (e.g., css-12345) and React hydration. However, it retains accessibility attributes. The content is often wrapped in main.  
  * **Challenge:** Cloud GitBook loads content dynamically. A crawler might see a skeleton unless it executes JS.  
  * **Selector:** If class names are obfuscated, use main or look for the div with the highest text density.  
  * **Sidebar:** Legacy uses .book-summary. Cloud uses nav tags within the flex container.

### **4.5 Hugo and Jekyll**

These generators are "theme-agnostic," meaning the HTML structure depends entirely on the theme chosen by the user (e.g., "Ananke" for Hugo, "Minima" for Jekyll).

* **Hugo:** Most well-coded themes utilize the HTML5 main tag. Common content classes include .content, .post-content, or .article-body. The main article combinator is a strong heuristic here.  
* **Jekyll (Minima):** The default theme puts content in .post-content or .main-content. Navigation is usually in .site-header and footer in .site-footer. Because Jekyll is older, it often relies more on semantic class names than complex DOM nesting.

## **5\. News Media Extraction Strategies**

Extracting content from news publishers differs fundamentally from documentation scraping. Publishers actively employ "anti-scraping" techniques, dynamic loading, and DOM obfuscation to protect ad revenue. Furthermore, news layouts are chaotic, with video players, newsletter signups, and "Read More" widgets interspersed with the text.

### **5.1 The New York Times (React & Data-TestID)**

The New York Times (NYT) utilizes a modern React frontend that heavily obfuscates class names. A scraper looking for .story-body (the old standard) will fail on modern articles which use hashes like css-1xdhyk6.

#### **5.1.1 The Data-TestID Strategy**

To maintain their own internal integration tests, NYT developers leave "breadcrumbs" in the form of data-testid attributes. These attributes are stable across builds, unlike CSS classes.

* **Wrapper:** \[data-testid="story-wrapper"\] or main\#site-content.  
* **Content:** section or div\[data-testid="story-content"\].  
* **Paragraphs:** div p.

**Extraction Strategy:** Avoid classes entirely. Use data-testid. The structure is usually main\#site-content \> div\[data-testid="story-wrapper"\]. Within this wrapper, there are often "companion columns" (for layout). The actual text is inside these columns. A robust selector finds the wrapper and then extracts all \<p\> tags within it, preserving order.

### **5.2 BBC News (Simorgh & JSON-LD)**

BBC News has transitioned to "Simorgh," a React-based platform. Like NYT, it uses dynamic classes (bbc-1151pbn), making CSS fragile.

#### **5.2.1 The JSON-LD Bypass**

BBC News is a prime example where *DOM scraping* is inferior to *Data extraction*. BBC embeds the full content of the article in a JSON-LD block within the \<head\>.

* **Method:** Look for \<script type="application/ld+json"\>. Parse the JSON.  
* **Field:** The articleBody field in the JSON contains the full text.  
* **HTML Fallback:** If raw HTML is strictly required (to preserve links), target the main\[role="main"\] container and the blocks identified by \[data-component="text-block"\].

### **5.3 Reuters and CNN (Obfuscation & Video)**

**Reuters:** Uses CSS Modules (e.g., .article-body\_\_content\_\_17Yit). The suffix is a hash.

* **Selector:** Use "Attribute Starts With" selectors: div\[class^="article-body\_\_content"\]. This ignores the dynamic hash suffix.  
* **Noise:** Reuters often injects a "Read Next" sidebar *visually* into the content column. This must be explicitly excluded via aside or nav tags.

**CNN:** Notorious for "div soup" and aggressive ad injection.

* **Selector:** .article\_\_content or .Paragraph\_\_component.  
* **Challenge:** CNN breaks articles into "Read More" segments that require clicking (or JS execution) to expand. A static scraper will often only get the first 3-4 paragraphs. To capture the full article, a headless browser (Puppeteer/Selenium) is usually required to trigger the "read more" expansion before extraction.

### **5.4 The Golden Path: Schema.org Microdata**

Given the volatility of news DOMs, the most resilient strategy across all publishers is to prioritize **Schema.org** extraction.

* **The Standard:** Publishers want Google News to index them. To ensure this, they inject semantic metadata.  
* **Selector:** \*.  
* **Benefits:** This attribute is often placed on the exact container of the text, stripping away the sidebars and headers that might otherwise be visually adjacent. If a page contains this attribute, it should supersede all heuristic or class-based selectors.  
* **Speakable Content:** The speakable property (used for voice assistants) is another high-value target. It defines the "summary" or most critical sections of the news object using CSS selectors or XPaths defined in the metadata itself.

## **6\. Implementation: The "Preservation" Algorithm**

Based on the analysis above, we can define a comprehensive algorithm for extracting raw HTML content while preserving links and structure. This algorithm operates in tiers, moving from specific high-confidence selectors to general heuristics.

### **6.1 Algorithm Architecture**

The process functions as a funnel:

1. **Tier 1: Framework Detection:** Check for specific signatures (e.g., id="\_\_docusaurus", meta\[name="generator" content="Hugo"\]). If detected, use the lookup table (Section 7).  
2. **Tier 2: Semantic Discovery:** If no framework is known, search for \`\` or \[role="main"\].  
3. **Tier 3: Heuristic Fallback:** If semantic tags are absent, apply a modified Readability algorithm (Link Density \+ Text Density) to identify the content cluster.  
4. **Tier 4: The Pruning Pass:** Once the container is isolated, rigorously remove "Blacklisted" elements (ads, nav, social) from *within* that container.  
5. **Tier 5: Sanitization:** Pass the result through a sanitizer to strip dangerous tags (script, iframe, object) while strictly allowing structural tags (p, h2, a, table).

### **6.2 The Exclusion List (The "Blacklist")**

To effectively strip boilerplate, the crawler must maintain a robust list of CSS selectors to remove. This list is derived from common web patterns and AdBlock rules.  
**Universal Exclusion Selectors:**

* **Structural:** header, footer, nav, aside.  
* **Roles:** \[role="navigation"\], \[role="banner"\], \[role="contentinfo"\], \[role="alert"\] (often cookie banners).  
* **Ad Patterns:** .ad, .advertisement, \[class\*="google\_ads"\], \[id\*="div-gpt-ad"\].  
* **Social/Sharing:** .share-buttons, .social-media, .twitter-tweet, div\[class\*="share"\].  
* **Popups/Modals:** .modal, .popup, .overlay, \[class\*="cookie"\], \[class\*="consent"\].  
* **Metadata:** .author-bio, .timestamp (unless specifically desired), .meta-data.  
* **Print Artifacts:** .no-print, .print-only.

### **6.3 Sanitization and Link Preservation**

The user explicitly requested preserving links. This requires a careful sanitization configuration. Tools like **DOMPurify** (Node.js) or **Bleach** (Python \- deprecated but reference) or **nh3** (modern Rust-based Python binding) are essential.  
**Configuration Strategy:**

* **Allow-list:** html, body, div, span, article, section, p, h1-h6, ul, ol, li, blockquote, pre, code, table, thead, tbody, tr, th, td, a, img, b, i, strong, em, br, hr.  
* **Attribute Allow-list:**  
  * a: href, title, name.  
  * img: src, alt, title.  
  * \*: class, id (Optional: keeping classes can be useful for styling, but stripping them results in cleaner "raw" HTML).  
* **XSS Prevention:** Ensure that href attributes are validated to contain only http, https, or mailto protocols. javascript: links must be stripped.  
* **SEO Handling:** It is best practice to inject rel="nofollow" into all extracted links to prevent the crawler from creating a "link farm" signature if the content is republished.

**DOMPurify Example Configuration:** When using DOMPurify in a JS crawler (e.g., Puppeteer), the config to preserve content but strip scripts would look like:  
`DOMPurify.sanitize(dirtyHTML, {`  
    `ALLOWED_TAGS: ['p', 'a', 'h1', 'h2', 'div', 'img', 'table', 'tr', 'td'],`  
    `ALLOWED_ATTR: ['href', 'src', 'alt', 'class'],`  
    `FORBID_TAGS: ['script', 'style', 'iframe', 'form', 'input', 'button'],`  
    `KEEP_CONTENT: true // Prevents stripping text inside non-allowed tags? No, usually default behavior.`  
`});`

*Note:* DOMPurify by default strips the *content* of script and style tags but preserves the content of unknown tags unless configured otherwise.

## **7\. Comprehensive Selector Reference Table**

The following table consolidates the research into a master reference for specific framework extraction. This serves as the core logic for "Tier 1" of the extraction algorithm.

| Framework / Site | Primary Container | Text Content Selector | Specific Exclusions (Remove these) | Source ID |
| :---- | :---- | :---- | :---- | :---- |
| **Docusaurus v2/v3** | main | article.markdown | .pagination-nav, .theme-doc-toc-desktop, .theme-doc-sidebar-container, .hash-link |  |
| **Sphinx (RTD)** | .wy-nav-content | \`\` | .wy-nav-side, .rst-footer-buttons, a.headerlink |  |
| **Sphinx (Alabaster)** | div.body | div.body | .sphinxsidebar, .link-header |  |
| **MkDocs (Material)** | .md-main | .md-content\_\_inner | .md-sidebar, .md-footer, .md-header, .md-clipboard |  |
| **GitBook (Legacy)** | .page-inner | .page-inner section | .book-summary, .book-header |  |
| **GitBook (Cloud)** | main | main | nav (inside main), div\[class\*="sidebar"\] |  |
| **Hugo (General)** | main | .content, .post-content | header, footer, .menu |  |
| **Nextra** | main | main | nav, footer, .nextra-sidebar-container |  |
| **NY Times** | \#site-content | section | \#site-content-skip, \[data-testid="related-links"\], \[data-testid="newsletter-signup"\] |  |
| **BBC News** | \[role="main"\] | \[data-component="text-block"\] | \[role="complementary"\], .bbc-1151pbn (dynamic ads) |  |
| **CNN** | .article\_\_content | .Paragraph\_\_component | .el-\[span\_63\](start\_span)\[span\_63\](end\_span)editorial-source, .zn-body\_\_read-more, .ad-container |  |
| **Reuters** | main | \[class\*="article-body\_\_cont\[span\_67\](start\_span)\[span\_67\](end\_span)ent"\] | \[data-testid="sidebar"\], nav, .read-next-container |  |

## **8\. Conclusion**

The landscape of web content extraction has shifted from simple HTML parsing to complex DOM forensics. The "article" is no longer a static file but a dynamic view constructed by frameworks like React, Hugo, and Sphinx.  
For a researcher or engineer tasked with building a crawler in 2025, reliance on a single algorithm is a point of failure. The optimal solution is a **Multi-Tiered Strategy**:

1. **Fingerprint the Framework:** Detect Docusaurus, Sphinx, or Next.js and apply the precise selectors defined in Section 7\.  
2. **Trust the Semantic Web:** If the framework is unknown, leverage itemprop="articleBody" and role="main" as the highest-confidence signals.  
3. **Fall Back on Heuristics:** Utilize Link Density and Text Density analysis (à la Readability/Newspaper3k) only when structural and semantic matching fails.  
4. **Aggressive Sanitization:** Regardless of how the content is found, it must be subjected to a strict exclusion list (EasyList-derived) and an HTML sanitizer to ensure the resulting raw HTML is safe, clean, and strictly informational.

By treating the DOM not just as a tree of tags but as a structured artifact of specific software ecosystems, extraction algorithms can achieve near-perfect fidelity, preserving the vital links and structure that transform raw text into connected knowledge.
