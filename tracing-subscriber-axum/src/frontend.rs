//! Frontend asset serving using embedded files

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::Response;
use include_dir::{include_dir, Dir};
use std::sync::Arc;

// Embed the frontend dist directory at compile time
static FRONTEND_DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/frontend/dist");

/// State for frontend serving (stores base path)
#[derive(Clone)]
pub struct FrontendState {
    pub base_path: Arc<String>,
}

/// Serve the index.html file at the root path
pub async fn serve_index(State(state): State<FrontendState>) -> Response {
    // Try to serve embedded index.html, fallback to placeholder
    if let Some(file) = FRONTEND_DIST.get_file("index.html") {
        let mut contents = String::from_utf8_lossy(file.contents()).to_string();

        // Inject base tag with absolute path to make assets work correctly
        // This ensures assets load from the correct base path
        if let Some(head_pos) = contents.find("<head>") {
            let insert_pos = head_pos + "<head>".len();
            let base_tag = format!("\n    <base href=\"{}/\">", state.base_path);
            contents.insert_str(insert_pos, &base_tag);
        }

        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .header(header::CACHE_CONTROL, "public, max-age=3600")
            .body(Body::from(contents))
            .unwrap()
    } else {
        serve_placeholder().await
    }
}

/// Serve static assets with proper MIME types
pub async fn serve_static(Path(path): Path<String>) -> Response {
    // Path already has the wildcard part extracted (e.g., "index-Dm3cA5i_.js")
    // We need to prepend "assets/" to match the embedded directory structure from Vite
    let asset_path = format!("assets/{}", path);

    // Try to serve from embedded assets
    if let Some(file) = FRONTEND_DIST.get_file(&asset_path) {
        let contents = file.contents();
        let mime_type = mime_guess::from_path(&asset_path)
            .first_or_octet_stream()
            .to_string();

        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime_type)
            .header(header::CACHE_CONTROL, "public, max-age=31536000") // 1 year for assets
            .body(Body::from(contents))
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from(format!("Asset not found: {}", asset_path)))
            .unwrap()
    }
}

/// Fallback handler for when frontend assets are not built yet
pub async fn serve_placeholder() -> Response {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Tracing Dashboard - Not Built</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }
        .container {
            text-align: center;
            padding: 2rem;
            background: rgba(0, 0, 0, 0.2);
            border-radius: 1rem;
            backdrop-filter: blur(10px);
        }
        h1 {
            margin: 0 0 1rem 0;
            font-size: 2.5rem;
        }
        p {
            margin: 0.5rem 0;
            font-size: 1.1rem;
        }
        code {
            background: rgba(255, 255, 255, 0.1);
            padding: 0.25rem 0.5rem;
            border-radius: 0.25rem;
            font-family: monospace;
        }
        .api-list {
            margin-top: 2rem;
            text-align: left;
            background: rgba(0, 0, 0, 0.2);
            padding: 1rem;
            border-radius: 0.5rem;
        }
        .api-list h2 {
            margin-top: 0;
        }
        .api-list ul {
            list-style: none;
            padding: 0;
        }
        .api-list li {
            margin: 0.5rem 0;
            font-family: monospace;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Tracing Dashboard</h1>
        <p>The frontend has not been built yet.</p>
        <p>To build the frontend, run:</p>
        <p><code>cd web && npm install && npm run build</code></p>

        <div class="api-list">
            <h2>Available API Endpoints:</h2>
            <ul>
                <li>GET /ws - WebSocket for real-time logs</li>
                <li>GET /api/logs - Get historical logs</li>
                <li>POST /api/levels - Update log levels</li>
                <li>GET /api/targets - Get log targets</li>
            </ul>
        </div>
    </div>
</body>
</html>
    "#;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(html))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mime_type_guessing() {
        use mime_guess::from_path;

        let js_mime = from_path("app.js").first_or_octet_stream();
        assert_eq!(js_mime.as_ref(), "text/javascript");

        let css_mime = from_path("style.css").first_or_octet_stream();
        assert_eq!(css_mime.as_ref(), "text/css");

        let html_mime = from_path("index.html").first_or_octet_stream();
        assert_eq!(html_mime.as_ref(), "text/html");

        let png_mime = from_path("image.png").first_or_octet_stream();
        assert_eq!(png_mime.as_ref(), "image/png");
    }

    #[tokio::test]
    async fn test_placeholder() {
        let response = serve_placeholder().await;
        assert_eq!(response.status(), StatusCode::OK);
    }
}
