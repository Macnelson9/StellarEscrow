use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    response::Response,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{EventQuery, PagedResponse, ReplayRequest, WebSocketMessage};
use crate::websocket::WebSocketManager;
use crate::{database::Database, models::Event};

/// Default page size — kept small for mobile clients.
const DEFAULT_LIMIT: i64 = 20;
const MAX_LIMIT: i64 = 100;

/// GET / — API discovery / navigation index.
pub async fn api_index() -> Json<serde_json::Value> {
    Json(json!({
        "name": "StellarEscrow Indexer API",
        "version": "1.0.0",
        "endpoints": {
            "health":          "GET  /health",
            "events":          "GET  /events?limit=20&offset=0&event_type=&trade_id=&from_ledger=&to_ledger=",
            "event_by_id":     "GET  /events/:id",
            "events_by_trade": "GET  /events/trade/:trade_id",
            "events_by_type":  "GET  /events/type/:event_type",
            "replay":          "POST /events/replay  {from_ledger, to_ledger?}",
            "websocket":       "GET  /ws",
            "help":            "GET  /help"
        }
    }))
}

pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    }))
}

pub async fn get_events(
    Query(params): Query<EventQuery>,
    State(state): State<AppState>,
) -> Result<Json<PagedResponse<Event>>, AppError> {
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let offset = params.offset.unwrap_or(0).max(0);
    let query = EventQuery { limit: Some(limit), offset: Some(offset), ..params };

    let (events, total) = tokio::try_join!(
        state.database.get_events(&query),
        state.database.count_events(&query),
    )?;

    Ok(Json(PagedResponse {
        has_more: offset + limit < total,
        items: events,
        total,
        limit,
        offset,
    }))
}

pub async fn get_event_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Event>, AppError> {
    let event = state.database.get_event_by_id(id).await?;
    Ok(Json(event))
}

pub async fn get_events_by_trade_id(
    Path(trade_id): Path<u64>,
    Query(params): Query<EventQuery>,
    State(state): State<AppState>,
) -> Result<Json<PagedResponse<Event>>, AppError> {
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let offset = params.offset.unwrap_or(0).max(0);
    let query = EventQuery { trade_id: Some(trade_id), limit: Some(limit), offset: Some(offset), ..params };

    let (events, total) = tokio::try_join!(
        state.database.get_events(&query),
        state.database.count_events(&query),
    )?;

    Ok(Json(PagedResponse { has_more: offset + limit < total, items: events, total, limit, offset }))
}

pub async fn get_events_by_type(
    Path(event_type): Path<String>,
    Query(params): Query<EventQuery>,
    State(state): State<AppState>,
) -> Result<Json<PagedResponse<Event>>, AppError> {
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let offset = params.offset.unwrap_or(0).max(0);
    let query = EventQuery { event_type: Some(event_type), limit: Some(limit), offset: Some(offset), ..params };

    let (events, total) = tokio::try_join!(
        state.database.get_events(&query),
        state.database.count_events(&query),
    )?;

    Ok(Json(PagedResponse { has_more: offset + limit < total, items: events, total, limit, offset }))
}

pub async fn replay_events(
    State(state): State<AppState>,
    Json(request): Json<ReplayRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let to_ledger = request.to_ledger.unwrap_or(i64::MAX);
    let events = state.database.get_events_in_range(request.from_ledger, to_ledger, "contract_id").await?;

    for event in &events {
        let ws_message = WebSocketMessage {
            event_type: event.event_type.clone(),
            data: event.data.clone(),
            timestamp: event.timestamp,
        };
        state.ws_manager.broadcast(ws_message).await;
    }

    Ok(Json(json!({ "replayed": events.len(), "from_ledger": request.from_ledger, "to_ledger": request.to_ledger })))
}

pub async fn ws_handler(
    State(state): State<AppState>,
    ws: axum::extract::ws::WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| state.ws_manager.handle_connection(socket))
}

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<Database>,
    pub ws_manager: Arc<WebSocketManager>,
}
