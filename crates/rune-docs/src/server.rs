use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use warp::Filter;
use crate::DocsEngine;

/// Start the documentation web server
pub async fn start_server(port: u16, docs: Arc<DocsEngine>) -> Result<()> {
    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    
    println!("🚀 Starting Rune documentation server at http://localhost:{}", port);
    println!("📚 Access documentation at: http://localhost:{}", port);
    println!("🔍 Search available at: http://localhost:{}/search", port);
    println!("💡 Examples at: http://localhost:{}/examples", port);
    println!("🎓 Tutorials at: http://localhost:{}/tutorials", port);
    println!();
    println!("Press Ctrl+C to stop the server");
    
    // Clone Arc for each route
    let docs_for_routes = docs.clone();
    let docs_for_content = docs.clone();
    let docs_for_examples = docs.clone();
    let docs_for_search = docs.clone();
    
    // Root route - redirect to getting started
    let root = warp::path::end()
        .map(|| {
            warp::reply::with_header(
                warp::reply::with_status("", warp::http::StatusCode::FOUND),
                "Location", "/getting-started"
            )
        });
    
    // Static CSS
    let css = warp::path("static")
        .and(warp::path("style.css"))
        .map(|| {
            warp::reply::with_header(
                include_str!("../static/style.css"),
                "content-type", "text/css"
            )
        });
    
    // Documentation content routes
    let content = warp::path::param::<String>()
        .and(warp::path::end())
        .map(move |page: String| {
            let docs = docs_for_content.clone();
            if let Some(content) = docs.get_content(&page) {
                let html = render_page(&page, content);
                warp::reply::with_header(html, "content-type", "text/html")
            } else {
                warp::reply::with_header(
                    render_404(&page),
                    "content-type", "text/html"
                )
            }
        });
    
    // Examples route
    let examples = warp::path("examples")
        .and(warp::path::param::<String>().or(warp::path::end().map(|| "".to_string())).unify())
        .map(move |category: String| {
            let docs = docs_for_examples.clone();
            let category = if category.is_empty() { None } else { Some(category.as_str()) };
            let examples = docs.get_examples(category);
            let html = render_examples_page(category, &examples);
            warp::reply::with_header(html, "content-type", "text/html")
        });
    
    // Search route
    let search = warp::path("search")
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .map(move |params: std::collections::HashMap<String, String>| {
            let docs = docs_for_search.clone();
            if let Some(query) = params.get("q") {
                let results = docs.search(query);
                let html = render_search_results(query, &results);
                warp::reply::with_header(html, "content-type", "text/html")
            } else {
                let html = render_search_page();
                warp::reply::with_header(html, "content-type", "text/html")
            }
        });
    
    // API endpoints for JSON data
    let api_examples = warp::path("api")
        .and(warp::path("examples"))
        .and(warp::path::param::<String>().or(warp::path::end().map(|| "".to_string())).unify())
        .map(move |category: String| {
            let docs = docs_for_routes.clone();
            let category = if category.is_empty() { None } else { Some(category.as_str()) };
            let examples = docs.get_examples(category);
            warp::reply::json(&examples)
        });
    
    let routes = root
        .or(css)
        .or(examples)
        .or(search)
        .or(api_examples)
        .or(content);
    
    warp::serve(routes)
        .run(addr)
        .await;
    
    Ok(())
}

/// Render the main page template
fn render_page(title: &str, content: &str) -> String {
    let html_content = markdown_to_html(content);
    format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - Rune VCS Documentation</title>
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    <nav class="navbar">
        <div class="nav-brand">
            <h1>📖 Rune VCS Documentation</h1>
        </div>
        <div class="nav-links">
            <a href="/getting-started">Getting Started</a>
            <a href="/commands">Commands</a>
            <a href="/examples">Examples</a>
            <a href="/migration">Migration</a>
            <a href="/best-practices">Best Practices</a>
            <a href="/troubleshooting">Troubleshooting</a>
            <a href="/search">Search</a>
        </div>
    </nav>
    
    <div class="container">
        <main class="content">
            {}
        </main>
        
        <aside class="sidebar">
            <div class="quick-links">
                <h3>🚀 Quick Start</h3>
                <ul>
                    <li><a href="/getting-started">Getting Started</a></li>
                    <li><a href="/examples/basic">Basic Examples</a></li>
                    <li><a href="/migration">Migrate from Git</a></li>
                </ul>
                
                <h3>📚 Documentation</h3>
                <ul>
                    <li><a href="/commands">Command Reference</a></li>
                    <li><a href="/best-practices">Best Practices</a></li>
                    <li><a href="/troubleshooting">Troubleshooting</a></li>
                </ul>
                
                <h3>💡 Learn</h3>
                <ul>
                    <li><a href="/examples">All Examples</a></li>
                    <li><a href="/examples/workflow">Workflows</a></li>
                    <li><a href="/examples/branching">Branching</a></li>
                </ul>
            </div>
        </aside>
    </div>
    
    <footer class="footer">
        <p>&copy; 2025 Rune VCS - Next-generation version control</p>
        <p>Generated by built-in documentation system</p>
    </footer>
</body>
</html>
"#, title, html_content)
}

/// Render examples page
fn render_examples_page(category: Option<&str>, examples: &[&crate::Example]) -> String {
    let title = match category {
        Some(cat) => format!("Examples: {}", cat),
        None => "All Examples".to_string(),
    };
    
    let content = if let Some(cat) = category {
        if examples.is_empty() {
            format!("<h1>Examples: {}</h1><p>No examples found for this category.</p>", cat)
        } else {
            let examples_html = examples.iter().map(|ex| {
                let commands_html = ex.commands.iter()
                    .enumerate()
                    .map(|(i, cmd)| format!("<div class=\"command\"><span class=\"prompt\">{}.</span> <code>{}</code></div>", i + 1, html_escape(cmd)))
                    .collect::<Vec<_>>()
                    .join("");
                
                let output_html = if let Some(output) = &ex.expected_output {
                    format!("<div class=\"expected-output\"><h4>Expected Output:</h4><pre>{}</pre></div>", html_escape(output))
                } else {
                    String::new()
                };
                
                format!(r#"
                <div class="example">
                    <h3>▶ {}</h3>
                    <p>{}</p>
                    <div class="commands">
                        {}
                    </div>
                    {}
                </div>
                "#, html_escape(&ex.title), html_escape(&ex.description), commands_html, output_html)
            }).collect::<Vec<_>>().join("");
            
            format!("<h1>Examples: {}</h1>{}", cat, examples_html)
        }
    } else {
        let categories = vec![
            ("basic", "Basic Operations", "Initialize, add, commit, status"),
            ("branching", "Branching & Merging", "Create branches, merge, workflows"),
            ("remote", "Remote Operations", "Clone, push, pull, collaboration"),
            ("ignore", "Ignore System", "Smart patterns, templates, debugging"),
            ("files", "File Operations", "Staging, diff, reset, restore"),
            ("workflow", "Workflows", "Feature development, daily routines"),
            ("migration", "Migration", "Converting from Git, equivalents"),
            ("troubleshooting", "Troubleshooting", "Common issues and solutions"),
        ];
        
        let categories_html = categories.iter().map(|(cat, title, desc)| {
            format!(r#"
            <div class="category-card">
                <h3><a href="/examples/{}">{}</a></h3>
                <p>{}</p>
            </div>
            "#, cat, title, desc)
        }).collect::<Vec<_>>().join("");
        
        format!("<h1>All Examples</h1><div class=\"categories\">{}</div>", categories_html)
    };
    
    render_page(&title, &content)
}

/// Render search page
fn render_search_page() -> String {
    let content = r#"
<h1>🔍 Search Documentation</h1>

<form method="get" action="/search" class="search-form">
    <input type="text" name="q" placeholder="Search documentation, examples, and tutorials..." class="search-input">
    <button type="submit" class="search-button">Search</button>
</form>

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
"#;
    
    render_page("Search", content)
}

/// Render search results
fn render_search_results(query: &str, results: &[crate::SearchResult]) -> String {
    let results_html = if results.is_empty() {
        format!("<p>No results found for \"{}\"</p>", html_escape(query))
    } else {
        results.iter().map(|result| {
            let type_icon = match result.result_type {
                crate::SearchResultType::Documentation => "📖",
                crate::SearchResultType::Example => "💡",
                crate::SearchResultType::Tutorial => "🎓",
            };
            
            format!(r#"
            <div class="search-result">
                <h3><a href="{}">{} {}</a></h3>
                <p>{}</p>
            </div>
            "#, result.url, type_icon, html_escape(&result.title), html_escape(&result.snippet))
        }).collect::<Vec<_>>().join("")
    };
    
    let content = format!(r#"
<h1>🔍 Search Results</h1>

<form method="get" action="/search" class="search-form">
    <input type="text" name="q" value="{}" placeholder="Search documentation..." class="search-input">
    <button type="submit" class="search-button">Search</button>
</form>

<div class="search-results">
    <p>Found {} result(s) for "{}"</p>
    {}
</div>
"#, html_escape(query), results.len(), html_escape(query), results_html);
    
    render_page("Search Results", &content)
}

/// Render 404 page
fn render_404(page: &str) -> String {
    let content = format!(r#"
<h1>📄 Page Not Found</h1>
<p>The page "{}" was not found.</p>

<h3>Available Pages:</h3>
<ul>
    <li><a href="/getting-started">Getting Started</a></li>
    <li><a href="/commands">Command Reference</a></li>
    <li><a href="/migration">Migration from Git</a></li>
    <li><a href="/best-practices">Best Practices</a></li>
    <li><a href="/troubleshooting">Troubleshooting</a></li>
    <li><a href="/examples">Examples</a></li>
    <li><a href="/search">Search</a></li>
</ul>
"#, html_escape(page));
    
    render_page("Page Not Found", &content)
}

/// Convert markdown to HTML (simple implementation)
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

/// Escape HTML special characters
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
