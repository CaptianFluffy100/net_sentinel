use crate::{gameserver_check, models::*, AppState};
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::sync::Arc;
use anyhow::Result;

pub async fn list_isps(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    match list_isps_internal(&state.store).await {
        Ok(isps) => (StatusCode::OK, Json(isps)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn list_isps_internal(store: &crate::db::JsonStore) -> Result<Vec<Isp>> {
    let db = store.read().await?;
    let mut isps = db.isps;
    isps.sort_by_key(|isp| isp.id);
    Ok(isps)
}

pub async fn create_isp(
    Extension(state): Extension<Arc<AppState>>,
    Json(create_isp): Json<CreateIsp>,
) -> impl IntoResponse {
    // Basic validation
    if create_isp.name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Name cannot be empty"})),
        )
            .into_response();
    }

    if create_isp.ip.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "IP cannot be empty"})),
        )
            .into_response();
    }

    let name = create_isp.name.clone();
    let ip = create_isp.ip.clone();

    let result = state.store.write(|db| {
        // Check for duplicate IP
        if db.isps.iter().any(|isp| isp.ip == ip) {
            return Err(anyhow::anyhow!("IP address already exists"));
        }

        let id = db.get_next_id();
        let isp = Isp {
            id,
            name: name.clone(),
            ip: ip.clone(),
        };
        let isp_clone = isp.clone();
        db.isps.push(isp);
        Ok(isp_clone)
    }).await;

    match result {
        Ok(isp) => {
            (StatusCode::CREATED, Json(isp)).into_response()
        }
        Err(e) => {
            let error_msg = e.to_string();
            let status = if error_msg.contains("already exists") {
                StatusCode::CONFLICT
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (
                status,
                Json(serde_json::json!({"error": error_msg})),
            )
                .into_response()
        }
    }
}

pub async fn delete_isp(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.store.write(|db| {
        let initial_len = db.isps.len();
        db.isps.retain(|isp| isp.id != id);
        if db.isps.len() < initial_len {
            Ok(())
        } else {
            Err(anyhow::anyhow!("ISP not found"))
        }
    }).await {
        Ok(_) => {
            (StatusCode::NO_CONTENT, Json(serde_json::json!({"success": true}))).into_response()
        }
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (
                status,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub async fn list_websites(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    match list_websites_internal(&state.store).await {
        Ok(websites) => (StatusCode::OK, Json(websites)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn list_websites_internal(store: &crate::db::JsonStore) -> Result<Vec<Website>> {
    let db = store.read().await?;
    let mut websites = db.websites;
    websites.sort_by_key(|website| website.id);
    Ok(websites)
}

pub async fn create_website(
    Extension(state): Extension<Arc<AppState>>,
    Json(create_website): Json<CreateWebsite>,
) -> impl IntoResponse {
    // Basic validation
    if create_website.url.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "URL cannot be empty"})),
        )
            .into_response();
    }

    let url = create_website.url.clone();
    let direct_connect = create_website.direct_connect;
    let direct_connect_url = create_website.direct_connect_url.clone();

    let result = state.store.write(|db| {
        // Check for duplicate URL
        if db.websites.iter().any(|website| website.url == url) {
            return Err(anyhow::anyhow!("URL already exists"));
        }

        let id = db.get_next_id();
        let website = Website {
            id,
            url: url.clone(),
            direct_connect,
            direct_connect_url: direct_connect_url.clone(),
        };
        let website_clone = website.clone();
        db.websites.push(website);
        Ok(website_clone)
    }).await;

    match result {
        Ok(website) => {
            (StatusCode::CREATED, Json(website)).into_response()
        }
        Err(e) => {
            let error_msg = e.to_string();
            let status = if error_msg.contains("already exists") {
                StatusCode::CONFLICT
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (
                status,
                Json(serde_json::json!({"error": error_msg})),
            )
                .into_response()
        }
    }
}

pub async fn delete_website(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.store.write(|db| {
        let initial_len = db.websites.len();
        db.websites.retain(|website| website.id != id);
        if db.websites.len() < initial_len {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Website not found"))
        }
    }).await {
        Ok(_) => {
            (StatusCode::NO_CONTENT, Json(serde_json::json!({"success": true}))).into_response()
        }
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (
                status,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub async fn list_game_servers(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    match list_game_servers_internal(&state.store).await {
        Ok(game_servers) => (StatusCode::OK, Json(game_servers)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn list_game_servers_internal(store: &crate::db::JsonStore) -> Result<Vec<GameServer>> {
    let db = store.read().await?;
    let mut game_servers = db.game_servers;
    game_servers.sort_by_key(|server| server.id);
    Ok(game_servers)
}

pub async fn create_game_server(
    Extension(state): Extension<Arc<AppState>>,
    Json(create_game_server): Json<CreateGameServer>,
) -> impl IntoResponse {
    if create_game_server.name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Name cannot be empty"})),
        )
            .into_response();
    }

    if create_game_server.address.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Address cannot be empty"})),
        )
            .into_response();
    }

    if create_game_server.pseudo_code.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Pseudo code cannot be empty"})),
        )
            .into_response();
    }

    let name = create_game_server.name.clone();
    let address = create_game_server.address.clone();
    let port = create_game_server.port;
    let protocol = create_game_server.protocol.clone();
    let timeout_ms = create_game_server.timeout_ms;
    let pseudo_code = create_game_server.pseudo_code.clone();

    let result = state.store.write(|db| {
        if db.game_servers.iter().any(|server| {
            server.address == address && server.port == port && server.protocol == protocol
        }) {
            return Err(anyhow::anyhow!("Game server with the same address/protocol already exists"));
        }

        let id = db.get_next_id();
        let game_server = GameServer {
            id,
            name: name.clone(),
            address: address.clone(),
            port,
            protocol: protocol.clone(),
            timeout_ms,
            pseudo_code: pseudo_code.clone(),
        };
        let game_server_clone = game_server.clone();
        db.game_servers.push(game_server);
        Ok(game_server_clone)
    }).await;

    match result {
        Ok(game_server) => {
            (StatusCode::CREATED, Json(game_server)).into_response()
        }
        Err(e) => {
            let error_msg = e.to_string();
            let status = if error_msg.contains("already exists") {
                StatusCode::CONFLICT
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (
                status,
                Json(serde_json::json!({"error": error_msg})),
            )
                .into_response()
        }
    }
}

pub async fn delete_game_server(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.store.write(|db| {
        let initial_len = db.game_servers.len();
        db.game_servers.retain(|server| server.id != id);
        if db.game_servers.len() < initial_len {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Game server not found"))
        }
    }).await {
        Ok(_) => {
            (StatusCode::NO_CONTENT, Json(serde_json::json!({"success": true}))).into_response()
        }
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (
                status,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub async fn test_game_server(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let server = match state.store.read().await {
        Ok(db) => db.game_servers.into_iter().find(|server| server.id == id),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    let server = match server {
        Some(server) => server,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Game server not found"})),
            )
                .into_response();
        }
    };

    let result = gameserver_check::check_game_server(&server).await;
    (StatusCode::OK, Json(result)).into_response()
}

pub async fn test_game_server_config(
    Json(create_game_server): Json<CreateGameServer>,
) -> impl IntoResponse {
    if create_game_server.address.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Address cannot be empty"})),
        )
            .into_response();
    }

    if create_game_server.pseudo_code.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Pseudo code is required"})),
        )
            .into_response();
    }

    let server = GameServer {
        id: 0,
        name: if create_game_server.name.trim().is_empty() {
            "Preview Server".to_string()
        } else {
            create_game_server.name.clone()
        },
        address: create_game_server.address.clone(),
        port: create_game_server.port,
        protocol: create_game_server.protocol.clone(),
        timeout_ms: create_game_server.timeout_ms,
        pseudo_code: create_game_server.pseudo_code.clone(),
    };

    println!("===================== Game Server Test =====================");
    println!(
        "Testing {} ({:?}): {}:{} timeout {}ms",
        server.name, server.protocol, server.address, server.port, server.timeout_ms
    );
    println!("Pseudo-code payload:\n{}", server.pseudo_code);

    let result = gameserver_check::check_game_server(&server).await;

    println!("Result success: {}", result.success);
    if let Some(raw) = &result.raw_response {
        println!("Raw response: {}", raw);
    }
    println!("Parsed values: {}", result.parsed_values);
    if let Some(error) = &result.error {
        println!(
            "Error: {} (line {:?})",
            error.message,
            error.line.as_ref().map(|line| line.to_string())
        );
    }
    println!("===========================================================");

    (StatusCode::OK, Json(result)).into_response()
}
