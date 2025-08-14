/// Template rendering utilities for the documentation system

use handlebars::Handlebars;
use serde_json::json;
use std::collections::HashMap;

/// Initialize Handlebars template engine with built-in templates
pub fn init_templates() -> Handlebars<'static> {
    let mut handlebars = Handlebars::new();
    
    // Register built-in templates
    handlebars.register_template_string("base", BASE_TEMPLATE)
        .expect("Failed to register base template");
    
    handlebars.register_template_string("page", PAGE_TEMPLATE)
        .expect("Failed to register page template");
    
    handlebars.register_template_string("examples", EXAMPLES_TEMPLATE)
        .expect("Failed to register examples template");
    
    handlebars.register_template_string("search", SEARCH_TEMPLATE)
        .expect("Failed to register search template");
    
    // Register helpers
    handlebars.register_helper("eq", Box::new(eq_helper));
    handlebars.register_helper("markdown", Box::new(markdown_helper));
    
    handlebars
}

/// Base HTML template
const BASE_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{title}} - Rune VCS Documentation</title>
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    {{> navbar}}
    
    <div class="container">
        <main class="content">
            {{{content}}}
        </main>
        
        {{> sidebar}}
    </div>
    
    {{> footer}}
</body>
</html>
"#;

/// Page content template
const PAGE_TEMPLATE: &str = r#"
<div class="page-content">
    {{{markdown content}}}
</div>
"#;

/// Examples page template
const EXAMPLES_TEMPLATE: &str = r#"
<h1>{{title}}</h1>

{{#if category}}
    {{#if examples}}
        {{#each examples}}
        <div class="example">
            <h3>▶ {{title}}</h3>
            <p>{{description}}</p>
            <div class="commands">
                {{#each commands}}
                <div class="command">
                    <span class="prompt">{{@index}}.</span>
                    <code>{{this}}</code>
                </div>
                {{/each}}
            </div>
            {{#if expected_output}}
            <div class="expected-output">
                <h4>Expected Output:</h4>
                <pre>{{expected_output}}</pre>
            </div>
            {{/if}}
        </div>
        {{/each}}
    {{else}}
        <p>No examples found for category "{{category}}".</p>
    {{/if}}
{{else}}
    <div class="categories">
        {{#each categories}}
        <div class="category-card">
            <h3><a href="/examples/{{id}}">{{title}}</a></h3>
            <p>{{description}}</p>
        </div>
        {{/each}}
    </div>
{{/if}}
"#;

/// Search template
const SEARCH_TEMPLATE: &str = r#"
<h1>🔍 {{#if query}}Search Results{{else}}Search Documentation{{/if}}</h1>

<form method="get" action="/search" class="search-form">
    <input type="text" name="q" value="{{query}}" placeholder="Search documentation, examples, and tutorials..." class="search-input">
    <button type="submit" class="search-button">Search</button>
</form>

{{#if query}}
    <div class="search-results">
        <p>Found {{results.length}} result(s) for "{{query}}"</p>
        {{#each results}}
        <div class="search-result">
            <h3><a href="{{url}}">{{type_icon}} {{title}}</a></h3>
            <p>{{snippet}}</p>
        </div>
        {{/each}}
    </div>
{{else}}
    <div class="search-tips">
        <h3>Search Tips</h3>
        <ul>
            <li>Search for command names: "commit", "branch", "merge"</li>
            <li>Look for topics: "ignore patterns", "collaboration", "troubleshooting"</li>
            <li>Find workflows: "feature development", "daily routine"</li>
            <li>Get help with errors: "merge conflict", "cannot push"</li>
        </ul>
    </div>

    <div class="popular-searches">
        <h3>Popular Searches</h3>
        <div class="search-tags">
            <a href="/search?q=getting+started" class="search-tag">Getting Started</a>
            <a href="/search?q=ignore+patterns" class="search-tag">Ignore Patterns</a>
            <a href="/search?q=branching" class="search-tag">Branching</a>
            <a href="/search?q=merge+conflict" class="search-tag">Merge Conflicts</a>
            <a href="/search?q=collaboration" class="search-tag">Collaboration</a>
            <a href="/search?q=migration" class="search-tag">Migration from Git</a>
        </div>
    </div>
{{/if}}
"#;

/// Handlebars helper for equality comparison
fn eq_helper(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let param1 = h.param(0).map(|v| v.value());
    let param2 = h.param(1).map(|v| v.value());
    
    let result = match (param1, param2) {
        (Some(v1), Some(v2)) => v1 == v2,
        _ => false,
    };
    
    out.write(&result.to_string())?;
    Ok(())
}

/// Handlebars helper for markdown rendering
fn markdown_helper(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    if let Some(content) = h.param(0).map(|v| v.value().as_str()).flatten() {
        let html = markdown_to_html(content);
        out.write(&html)?;
    }
    Ok(())
}

/// Convert markdown to HTML
fn markdown_to_html(markdown: &str) -> String {
    use pulldown_cmark::{Parser, Options, html};
    
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    
    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

/// Render a template with data
pub fn render_template(
    handlebars: &Handlebars,
    template_name: &str,
    data: &serde_json::Value,
) -> Result<String, handlebars::RenderError> {
    handlebars.render(template_name, data)
}

/// Create template data for a page
pub fn create_page_data(title: &str, content: &str) -> serde_json::Value {
    json!({
        "title": title,
        "content": content
    })
}

/// Create template data for examples
pub fn create_examples_data(
    category: Option<&str>,
    examples: &[&crate::Example],
) -> serde_json::Value {
    if let Some(cat) = category {
        json!({
            "title": format!("Examples: {}", cat),
            "category": cat,
            "examples": examples
        })
    } else {
        let categories = vec![
            json!({
                "id": "basic",
                "title": "Basic Operations",
                "description": "Initialize, add, commit, status"
            }),
            json!({
                "id": "branching",
                "title": "Branching & Merging",
                "description": "Create branches, merge, workflows"
            }),
            json!({
                "id": "remote",
                "title": "Remote Operations",
                "description": "Clone, push, pull, collaboration"
            }),
            json!({
                "id": "ignore",
                "title": "Ignore System",
                "description": "Smart patterns, templates, debugging"
            }),
            json!({
                "id": "files",
                "title": "File Operations",
                "description": "Staging, diff, reset, restore"
            }),
            json!({
                "id": "workflow",
                "title": "Workflows",
                "description": "Feature development, daily routines"
            }),
            json!({
                "id": "migration",
                "title": "Migration",
                "description": "Converting from Git, equivalents"
            }),
            json!({
                "id": "troubleshooting",
                "title": "Troubleshooting",
                "description": "Common issues and solutions"
            }),
        ];
        
        json!({
            "title": "All Examples",
            "categories": categories
        })
    }
}

/// Create template data for search
pub fn create_search_data(
    query: Option<&str>,
    results: Option<&[crate::SearchResult]>,
) -> serde_json::Value {
    match (query, results) {
        (Some(q), Some(r)) => {
            let results_with_icons: Vec<_> = r.iter().map(|result| {
                let type_icon = match result.result_type {
                    crate::SearchResultType::Documentation => "📖",
                    crate::SearchResultType::Example => "💡",
                    crate::SearchResultType::Tutorial => "🎓",
                };
                
                json!({
                    "title": result.title,
                    "snippet": result.snippet,
                    "url": result.url,
                    "type_icon": type_icon
                })
            }).collect();
            
            json!({
                "query": q,
                "results": results_with_icons
            })
        }
        _ => json!({})
    }
}
