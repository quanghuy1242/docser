use std::sync::OnceLock;

static JS_SCRIPT: OnceLock<String> = OnceLock::new();

pub fn load_js_script() -> &'static str {
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