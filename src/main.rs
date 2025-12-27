mod api;
mod code_server;
mod db;
mod models;
mod out;
mod packet_parser;
mod gameserver_check;

use axum::{
    extract::Extension,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post, delete},
    Router,
};
use std::sync::Arc;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize JSON database
    let store = db::init_db().await?;

    let app_state = Arc::new(AppState { store });

    // Build our application with routes
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/code-server.js", get(code_server::language_server_handler))
        .route("/api/isps", get(api::list_isps))
        .route("/api/isps", post(api::create_isp))
        .route("/api/isps/:id", delete(api::delete_isp))
        .route("/api/websites", get(api::list_websites))
        .route("/api/websites", post(api::create_website))
        .route("/api/websites/:id", delete(api::delete_website))
        .route("/api/gameservers", get(api::list_game_servers))
        .route("/api/gameservers", post(api::create_game_server))
        .route("/api/gameservers/test", post(api::test_game_server_config))
        .route("/api/gameservers/:id", delete(api::delete_game_server))
        .route("/api/gameservers/:id/test", post(api::test_game_server))
        .route("/metrics", get(metrics_handler))
        .layer(Extension(app_state));

    // Run it
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3100").await?;
    out::info("main", &format!("Net Sentinel running on http://localhost:3100"));
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
struct AppState {
    store: db::JsonStore,
}

async fn index_handler() -> impl IntoResponse {
    let html = include_str!("../public/index.html").replace("{{VERSION}}", VERSION);
    Html(html)
}


async fn check_internet_connectivity(ip: &str) -> (bool, u64) {
    use tokio::time::{timeout, Duration, Instant};
    let start = Instant::now();
    
    // Create HTTP client with short timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build();
    
    let client = match client {
        Ok(c) => c,
        Err(_) => return (false, start.elapsed().as_millis() as u64),
    };
    
    // Try HTTP request to the IP (try both HTTP and HTTPS)
    let urls = [
        format!("http://{}", ip),
        format!("https://{}", ip),
    ];
    
    for url in &urls {
        if let Ok(result) = timeout(Duration::from_secs(2), client.get(url).send()).await {
            if result.is_ok() {
                // Even if we get an error response (like 404), if we got a response,
                // the IP is reachable, so internet is up
                let elapsed_ms = start.elapsed().as_millis() as u64;
                return (true, elapsed_ms);
            }
        }
    }
    
    let elapsed_ms = start.elapsed().as_millis() as u64;
    (false, elapsed_ms)
}

async fn check_website_external(url: &str) -> (bool, u64) {
    use tokio::time::{timeout, Duration, Instant};
    let start = Instant::now();
    
    // Ensure URL has scheme
    let url = if !url.starts_with("http://") && !url.starts_with("https://") {
        format!("https://{}", url)
    } else {
        url.to_string()
    };
    
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build();
    
    let client = match client {
        Ok(c) => c,
        Err(_) => {
            let elapsed_ms = start.elapsed().as_millis() as u64;
            return (false, elapsed_ms);
        }
    };
    
    let result = if let Ok(result) = timeout(Duration::from_secs(2), client.get(&url).send()).await {
        if let Ok(response) = result {
            // Only consider the website up if we get a successful HTTP status code (200-299)
            response.status().is_success()
        } else {
            false
        }
    } else {
        false
    };
    
    let elapsed_ms = start.elapsed().as_millis() as u64;
    (result, elapsed_ms)
}

async fn check_website_direct(url: &str, direct_connect_url: Option<&str>) -> (bool, u64) {
    use tokio::time::{timeout, Duration, Instant};
    let start = Instant::now();
    
    // If direct_connect_url is provided, use it directly
    if let Some(direct_url) = direct_connect_url {
        if !direct_url.trim().is_empty() {
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(2))
                .danger_accept_invalid_certs(true)
                .build();
            
            if let Ok(client) = client {
                if let Ok(result) = timeout(Duration::from_secs(2), client.get(direct_url).send()).await {
                    if let Ok(response) = result {
                        // Only consider the website up if we get a successful HTTP status code (200-299)
                        if response.status().is_success() {
                            let elapsed_ms = start.elapsed().as_millis() as u64;
                            return (true, elapsed_ms);
                        }
                    }
                }
            }
            let elapsed_ms = start.elapsed().as_millis() as u64;
            return (false, elapsed_ms);
        }
    }
    
    // Fallback: Parse URL to get hostname and resolve DNS
    let url_str = if !url.starts_with("http://") && !url.starts_with("https://") {
        format!("https://{}", url)
    } else {
        url.to_string()
    };
    
    let parsed_url = match reqwest::Url::parse(&url_str) {
        Ok(u) => u,
        Err(_) => {
            let elapsed_ms = start.elapsed().as_millis() as u64;
            return (false, elapsed_ms);
        }
    };
    
    let hostname = match parsed_url.host_str() {
        Some(h) => h,
        None => {
            let elapsed_ms = start.elapsed().as_millis() as u64;
            return (false, elapsed_ms);
        }
    };
    
    // Resolve DNS to get IP address
    let ip = match tokio::net::lookup_host(format!("{}:80", hostname)).await {
        Ok(mut addrs) => {
            match addrs.next() {
                Some(addr) => addr.ip(),
                None => {
                    let elapsed_ms = start.elapsed().as_millis() as u64;
                    return (false, elapsed_ms);
                }
            }
        }
        Err(_) => {
            let elapsed_ms = start.elapsed().as_millis() as u64;
            return (false, elapsed_ms);
        }
    };
    
    // Try both HTTP and HTTPS
    let schemes = ["http", "https"];
    let port = parsed_url.port().unwrap_or_else(|| {
        if url_str.starts_with("https://") { 443 } else { 80 }
    });
    
    for scheme in &schemes {
        let direct_url = format!("{}://{}:{}/", scheme, ip, port);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .danger_accept_invalid_certs(true) // For direct IP connections
            .build();
        
        if let Ok(client) = client {
            let request = client.get(&direct_url).header("Host", hostname);
            if let Ok(result) = timeout(Duration::from_secs(2), request.send()).await {
                if let Ok(response) = result {
                    // Only consider the website up if we get a successful HTTP status code (200-299)
                    if response.status().is_success() {
                        let elapsed_ms = start.elapsed().as_millis() as u64;
                        return (true, elapsed_ms);
                    }
                }
            }
        }
    }
    
    let elapsed_ms = start.elapsed().as_millis() as u64;
    (false, elapsed_ms)
}

async fn metrics_handler(Extension(state): Extension<Arc<AppState>>) -> Response {
    let start = std::time::Instant::now();
    let isps = match api::list_isps_internal(&state.store).await {
        Ok(isps) => isps,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "# HELP net_sentinel_error Error fetching ISPs\n# TYPE net_sentinel_error counter\nnet_sentinel_error 1\n",
            )
                .into_response();
        }
    };

    let websites = match api::list_websites_internal(&state.store).await {
        Ok(websites) => websites,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "# HELP net_sentinel_error Error fetching websites\n# TYPE net_sentinel_error counter\nnet_sentinel_error 1\n",
            )
                .into_response();
        }
    };

    let game_servers = match api::list_game_servers_internal(&state.store).await {
        Ok(servers) => servers,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "# HELP net_sentinel_error Error fetching game servers\n# TYPE net_sentinel_error counter\nnet_sentinel_error 1\n",
            )
                .into_response();
        }
    };

    // Run all checks concurrently: ISPs, websites, and game servers all at the same time
    let ((internet_up, isp_timing_results), website_results, game_server_results) = tokio::join!(
        // Check internet connectivity - check all ISPs concurrently (max 100 at a time)
        async {
            if !isps.is_empty() {
                use futures::stream::{self, StreamExt};
                use std::collections::HashMap;
                
                // Create a stream of futures with concurrency limit of 100
                let ip_addresses: Vec<String> = isps.iter().map(|isp| isp.ip.clone()).collect();
                let results = stream::iter(ip_addresses.iter().cloned())
                    .map(|ip| async move {
                        let (success, timing_ms) = check_internet_connectivity(&ip).await;
                        (ip, success, timing_ms)
                    })
                    .buffer_unordered(100);
                
                // Check results as they come in - return true on first success
                let mut stream = results;
                let mut internet_up_result = false;
                let mut timing_map: HashMap<String, u64> = HashMap::new();
                while let Some((ip, success, timing_ms)) = stream.next().await {
                    timing_map.insert(ip.clone(), timing_ms);
                    if success && !internet_up_result {
                        // Found a reachable ISP, internet is up
                        internet_up_result = true;
                    }
                }
                (internet_up_result, timing_map)
            } else {
                (false, std::collections::HashMap::new())
            }
        },
        // Check all websites concurrently (max 100 at a time)
        async {
            if !websites.is_empty() {
                use std::collections::HashMap;
                use futures::stream::{self, StreamExt};
                
                // Build a list of all check operations (external and direct) to perform with cloned data
                let mut check_operations = Vec::new();
                for website in &websites {
                    let url = website.url.clone();
                    let url_for_check = website.url.clone();
                    check_operations.push(("external".to_string(), url.clone(), url_for_check.clone(), None));
                    
                    if website.direct_connect {
                        let url_for_check2 = website.url.clone();
                        let direct_url = website.direct_connect_url.clone();
                        check_operations.push(("direct".to_string(), url.clone(), url_for_check2, direct_url));
                    }
                }
                
                // Execute all checks concurrently
                let results_stream = stream::iter(check_operations)
                    .map(|(check_type, url, url_for_check, direct_url)| async move {
                        let (result, timing_ms) = match check_type.as_str() {
                            "external" => {
                                check_website_external(&url_for_check).await
                            }
                            "direct" => {
                                check_website_direct(&url_for_check, direct_url.as_deref()).await
                            }
                            _ => (false, 0),
                        };
                        ((url, check_type), (result, timing_ms))
                    })
                    .buffer_unordered(100);
                
                let mut results = HashMap::new();
                let mut stream = results_stream;
                while let Some((key, result_timing)) = stream.next().await {
                    results.insert(key, result_timing);
                }
                
                results
            } else {
                std::collections::HashMap::new()
            }
        },
        // Check game servers concurrently
        async {
            if !game_servers.is_empty() {
                use std::collections::HashMap;
                use futures::stream::{self, StreamExt};
                
                let servers_clone: Vec<_> = game_servers.iter().cloned().collect();
                let results_stream = stream::iter(servers_clone)
                    .map(|server| async move {
                        let result = crate::gameserver_check::check_game_server(&server).await;
                        (server.id, server.name.clone(), server.address.clone(), server.port, result)
                    })
                    .buffer_unordered(100);
                
                let mut results = HashMap::new();
                let mut stream = results_stream;
                while let Some((id, name, address, port, result)) = stream.next().await {
                    results.insert(id, (name, address, port, result));
                }
                results
            } else {
                std::collections::HashMap::new()
            }
        }
    );

    let response = build_metrics_response(&isps, internet_up, &isp_timing_results, &websites, &website_results, &game_servers, &game_server_results);
    
    // Log timing information for fastest and slowest checks
    log_timing_info(&isps, &isp_timing_results, &websites, &website_results, &game_servers, &game_server_results);
    
    let elapsed = start.elapsed();
    out::info("metrics", &format!("Processed /metrics endpoint in {:.2}ms", elapsed.as_secs_f64() * 1000.0));
    response
}

fn log_timing_info(
    isps: &[crate::models::Isp],
    isp_timing_results: &std::collections::HashMap<String, u64>,
    websites: &[crate::models::Website],
    website_results: &std::collections::HashMap<(String, String), (bool, u64)>,
    game_servers: &[crate::models::GameServer],
    game_server_results: &std::collections::HashMap<i64, (String, String, u16, crate::models::GameServerTestResult)>,
) {
    use crate::out;
    
    // Collect all timing data with identifiers
    let mut all_timings: Vec<(String, u64)> = Vec::new();
    
    // ISP timings
    for isp in isps {
        if let Some(&timing_ms) = isp_timing_results.get(&isp.ip) {
            all_timings.push((format!("ISP: {} ({})", isp.name, isp.ip), timing_ms));
        }
    }
    
    // Website timings
    for website in websites {
        if let Some(&(_, timing_ms)) = website_results.get(&(website.url.clone(), "external".to_string())) {
            all_timings.push((format!("Website External: {}", website.url), timing_ms));
        }
        if website.direct_connect {
            if let Some(&(_, timing_ms)) = website_results.get(&(website.url.clone(), "direct".to_string())) {
                all_timings.push((format!("Website Direct: {}", website.url), timing_ms));
            }
        }
    }
    
    // Game server timings
    for server in game_servers {
        if let Some((name, address, port, result)) = game_server_results.get(&server.id) {
            all_timings.push((format!("Game Server: {} ({}:{})", name, address, port), result.response_time_ms));
        }
    }
    
    if all_timings.is_empty() {
        return;
    }
    
    // Find fastest and slowest
    if let Some(fastest) = all_timings.iter().min_by_key(|(_, ms)| *ms) {
        out::info("timing", &format!("Fastest check: {} - {}ms", fastest.0, fastest.1));
    }
    
    if let Some(slowest) = all_timings.iter().max_by_key(|(_, ms)| *ms) {
        out::info("timing", &format!("Slowest check: {} - {}ms", slowest.0, slowest.1));
    }
    
    // Log all timings sorted by time
    let mut sorted_timings = all_timings;
    sorted_timings.sort_by_key(|(_, ms)| *ms);
    out::info("timing", "All check times (sorted):");
    for (name, timing_ms) in sorted_timings {
        out::info("timing", &format!("  {} - {}ms", name, timing_ms));
    }
}

fn parse_return_output(output: &str) -> Vec<(String, String)> {
    // Parse a RETURN output string like "server=10.0.2.27, protocol=773, player_max=500"
    // into a vector of (key, value) pairs
    let mut pairs = Vec::new();
    
    for part in output.split(',') {
        let part = part.trim();
        if let Some(equal_pos) = part.find('=') {
            let key = part[..equal_pos].trim().to_string();
            let value = part[equal_pos + 1..].trim().to_string();
            
            // Remove quotes if present (both single and double)
            let value = value
                .trim_start_matches('\'')
                .trim_end_matches('\'')
                .trim_start_matches('"')
                .trim_end_matches('"')
                .to_string();
            
            if !key.is_empty() {
                pairs.push((key, value));
            }
        }
    }
    
    pairs
}

fn escape_prometheus_label(value: &str) -> String {
    // Escape special characters in Prometheus label values
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn sanitize_metric_name(name: &str) -> String {
    // Prometheus metric names must match [a-zA-Z_:][a-zA-Z0-9_:]*
    // Replace invalid characters with underscores
    let mut sanitized = String::new();
    let mut chars = name.chars().peekable();
    
    // First character must be a letter, underscore, or colon
    if let Some(&first) = chars.peek() {
        if first.is_ascii_alphabetic() || first == '_' || first == ':' {
            sanitized.push(first);
            chars.next();
        } else {
            // If first char is invalid, prefix with underscore
            sanitized.push('_');
        }
    }
    
    // Remaining characters can be alphanumeric, underscore, or colon
    for ch in chars {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == ':' {
            sanitized.push(ch);
        } else {
            sanitized.push('_');
        }
    }
    
    sanitized
}

fn build_metrics_response(
    isps: &[crate::models::Isp],
    internet_up: bool,
    isp_timing_results: &std::collections::HashMap<String, u64>,
    websites: &[crate::models::Website],
    website_results: &std::collections::HashMap<(String, String), (bool, u64)>,
    game_servers: &[crate::models::GameServer],
    game_server_results: &std::collections::HashMap<i64, (String, String, u16, crate::models::GameServerTestResult)>,
) -> Response {
    let mut metrics = format!(
        "# HELP net_sentinel_version Version information\n# TYPE net_sentinel_version gauge\nnet_sentinel_version{{version=\"{}\"}} 1\n",
        VERSION
    );

    metrics.push_str("# HELP net_sentinel_internet_up Internet connectivity status (1 = up, 0 = down)\n# TYPE net_sentinel_internet_up gauge\n");
    metrics.push_str(&format!("net_sentinel_internet_up {}\n", if internet_up { 1 } else { 0 }));

    // Add ISP timing metrics
    metrics.push_str("# HELP net_sentinel_isp_response_time ISP response time in milliseconds\n# TYPE net_sentinel_isp_response_time gauge\n");
    for isp in isps {
        if let Some(&timing_ms) = isp_timing_results.get(&isp.ip) {
            metrics.push_str(&format!(
                "net_sentinel_isp_response_time{{name=\"{}\",ip=\"{}\"}} {}\n",
                escape_prometheus_label(&isp.name),
                escape_prometheus_label(&isp.ip),
                timing_ms
            ));
        }
    }

    // Add website metrics
    metrics.push_str("# HELP net_sentinel_website_external_up External website connectivity status (1 = up, 0 = down)\n# TYPE net_sentinel_website_external_up gauge\n");
    metrics.push_str("# HELP net_sentinel_website_external_response_time External website response time in milliseconds\n# TYPE net_sentinel_website_external_response_time gauge\n");
    metrics.push_str("# HELP net_sentinel_website_direct_up Direct website connectivity status (1 = up, 0 = down)\n# TYPE net_sentinel_website_direct_up gauge\n");
    metrics.push_str("# HELP net_sentinel_website_direct_response_time Direct website response time in milliseconds\n# TYPE net_sentinel_website_direct_response_time gauge\n");
    
    for website in websites {
        // Extract site name from URL (remove protocol, path, etc.)
        let site = website.url
            .replace("https://", "")
            .replace("http://", "")
            .split('/')
            .next()
            .unwrap_or(&website.url)
            .split(':')
            .next()
            .unwrap_or(&website.url)
            .to_string();
        
        // External check result
        if let Some(&(external_result, timing_ms)) = website_results.get(&(website.url.clone(), "external".to_string())) {
            metrics.push_str(&format!(
                "net_sentinel_website_external_up{{site=\"{}\"}} {}\n",
                site,
                if external_result { 1 } else { 0 }
            ));
            metrics.push_str(&format!(
                "net_sentinel_website_external_response_time{{site=\"{}\"}} {}\n",
                site,
                timing_ms
            ));
        }
        
        // Direct check result (only if direct_connect is enabled)
        if website.direct_connect {
            if let Some(&(direct_result, timing_ms)) = website_results.get(&(website.url.clone(), "direct".to_string())) {
                metrics.push_str(&format!(
                    "net_sentinel_website_direct_up{{site=\"{}\"}} {}\n",
                    site,
                    if direct_result { 1 } else { 0 }
                ));
                metrics.push_str(&format!(
                    "net_sentinel_website_direct_response_time{{site=\"{}\"}} {}\n",
                    site,
                    timing_ms
                ));
            }
        }
    }

    // Add game server metrics
    metrics.push_str("# HELP net_sentinel_gameserver_up Game server connectivity status (1 = up, 0 = down)\n# TYPE net_sentinel_gameserver_up gauge\n");
    metrics.push_str("# HELP net_sentinel_gameserver_response_time Game server response time in milliseconds\n# TYPE net_sentinel_gameserver_response_time gauge\n");
    
    // Track which output metrics we've documented to avoid duplicate HELP/TYPE lines
    let mut documented_metrics = std::collections::HashSet::new();
    
    for server in game_servers {
        if let Some((name, address, port, result)) = game_server_results.get(&server.id) {
            let is_up = result.success;
            let response_time = result.response_time_ms;
            
            metrics.push_str(&format!(
                "net_sentinel_gameserver_up{{name=\"{}\",address=\"{}\",port=\"{}\"}} {}\n",
                escape_prometheus_label(name),
                escape_prometheus_label(address),
                port,
                if is_up { 1 } else { 0 }
            ));
            
            metrics.push_str(&format!(
                "net_sentinel_gameserver_response_time{{name=\"{}\",address=\"{}\",port=\"{}\"}} {}\n",
                escape_prometheus_label(name),
                escape_prometheus_label(address),
                port,
                response_time
            ));
            
            // Build common labels string (name, address, port)
            let common_labels = format!(
                "name=\"{}\",address=\"{}\",port=\"{}\"",
                escape_prometheus_label(name),
                escape_prometheus_label(address),
                port
            );
            
            // Add output metrics for success case
            for label in &result.output_labels_success {
                // Parse the RETURN output string (e.g., "protocol=773, player_max=500, version=1.20.1")
                let parsed_labels = parse_return_output(label);
                
                // Create a separate metric for each key-value pair
                for (key, value) in &parsed_labels {
                    // Sanitize key for metric name (Prometheus metric names must match [a-zA-Z_:][a-zA-Z0-9_:]*)
                    let sanitized_key = sanitize_metric_name(key);
                    let metric_name = format!("net_sentinel_gameserver_output_{}", sanitized_key);
                    
                    // Add HELP and TYPE lines once per metric type
                    if documented_metrics.insert(metric_name.clone()) {
                        metrics.push_str(&format!(
                            "# HELP {} Game server output metric for {}\n# TYPE {} gauge\n",
                            metric_name, key, metric_name
                        ));
                    }
                    
                    // Try to parse value as a number, otherwise use 1 and add value as a label
                    let (metric_value, labels_str) = if let Ok(num) = value.parse::<f64>() {
                        // Numeric value - use it directly
                        (num, common_labels.clone())
                    } else {
                        // String value - use 1 as value and add original value as a label
                        let labels_with_value = format!("{},value=\"{}\"", common_labels, escape_prometheus_label(value));
                        (1.0, labels_with_value)
                    };
                    
                    metrics.push_str(&format!(
                        "{}{{{}}} {}\n",
                        metric_name,
                        labels_str,
                        metric_value
                    ));
                }
            }
            
            // Add output metrics for error case (if needed, could be similar)
            for label in &result.output_labels_error {
                let parsed_labels = parse_return_output(label);
                
                for (key, value) in &parsed_labels {
                    let sanitized_key = sanitize_metric_name(key);
                    let metric_name = format!("net_sentinel_gameserver_output_{}", sanitized_key);
                    
                    if documented_metrics.insert(metric_name.clone()) {
                        metrics.push_str(&format!(
                            "# HELP {} Game server output metric for {}\n# TYPE {} gauge\n",
                            metric_name, key, metric_name
                        ));
                    }
                    
                    // For error cases, might want to handle differently, but using same logic for now
                    let (metric_value, labels_str) = if let Ok(num) = value.parse::<f64>() {
                        (num, common_labels.clone())
                    } else {
                        let labels_with_value = format!("{},value=\"{}\"", common_labels, escape_prometheus_label(value));
                        (1.0, labels_with_value)
                    };
                    
                    metrics.push_str(&format!(
                        "{}{{{}}} {}\n",
                        metric_name,
                        labels_str,
                        metric_value
                    ));
                }
            }
        } else {
            // Server not checked (shouldn't happen, but handle gracefully)
            metrics.push_str(&format!(
                "net_sentinel_gameserver_up{{name=\"{}\",address=\"{}\",port=\"{}\"}} 0\n",
                server.name.replace('"', "\\\""),
                server.address.replace('"', "\\\""),
                server.port
            ));
        }
    }

    (StatusCode::OK, metrics).into_response()
}
