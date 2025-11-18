# Docser

A web scraping MCP server that converts web pages to markdown using headless browser automation. There should be tons of other tools, but I wanted to do this myself.

## Quick Install

1. **Install docser:**
   ```bash
   cargo install --git https://github.com/quanghuy1242/docser
   ```

2. **Install browser binaries:**
   ```bash
   npx playwright install webkit
   ```

   **Note:** This downloads WebKit browser (~50MB) required for scraping.

3. **Configure MCP:**
   ```json
   {
     "mcpServers": {
       "docser": {
         "command": "docser"
       }
     }
   }
   ```

### Usage

After installation and MCP configuration:

1. **Restart your AI assistant** (Claude, etc.) to load the new MCP server
2. **Use the tool** in conversations:

   ```
   Please scrape this webpage: https://example.com/blog/post
   ```

   The AI will automatically use the `crawl_url` tool and return the content as markdown.

## Manual Installation (For Developers)

### Installation Steps

1. **Build the project:**
   ```bash
   cargo build --release
   ```

2. **Configure MCP:**
   ```json
   {
     "mcpServers": {
       "docser": {
         "command": "cargo",
         "args": ["run", "--bin", "docser", "--release"],
         "cwd": "/path/to/docser"
       }
     }
   }
   ```

2. **Test:**
   ```bash
   cargo run --release -- --test

   # MCP inspector testing
   npx @modelcontextprotocol/inspector cargo run --bin docser
   ```

## Troubleshooting

- **Command not found**: Ensure `~/.cargo/bin` is in your PATH (rustup adds this automatically)
- **Browser not found**: Run `npx playwright install webkit` to download browser binaries
- **First run is slow**: After browser installation, initial page loads may be slower
- **Timeout issues**: Check network connectivity and page load times
- **Missing content**: Verify the page uses standard HTML structure
