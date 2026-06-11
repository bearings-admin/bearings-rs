//! Minimal **read-only** MCP server over the Streamable-HTTP transport, mounted at
//! `POST /mcp`. It speaks JSON-RPC 2.0 and implements the methods a client needs to
//! discover and call tools: `initialize`, `ping`, `tools/list`, `tools/call`.
//!
//! Each tool is a thin, injection-safe gateway over one PostgREST query (a table plus
//! an allow-list of filters), so the Bearings directory — events, places, title
//! holders, creators, campaigns, digital spaces — is queryable by any MCP client
//! (Claude Desktop/Code, custom agents). **No tool writes:** read-only by construction.
//!
//! Hand-rolled rather than pulled from an SDK on purpose: the surface is tiny, it adds
//! no dependency, and a reviewer can see exactly how MCP maps onto our data layer.

use axum::{extract::State, http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::{json, Value};
use crate::db::SupabaseClient;
use crate::repositories::clause;

/// MCP protocol revision we default to (we echo the client's if it sends one).
const PROTOCOL_VERSION: &str = "2025-06-18";

/// One allowed filter: (argument name, db column, PostgREST op, description).
type Filter = (&'static str, &'static str, &'static str, &'static str);

/// A read-only tool = metadata + a PostgREST query spec.
struct Tool {
    name: &'static str,
    description: &'static str,
    table: &'static str,
    /// Always-applied query (e.g. `active=eq.true`); empty for views.
    base: &'static str,
    select: &'static str,
    /// PostgREST order clause without the `order=` prefix; empty to skip.
    order: &'static str,
    filters: &'static [Filter],
}

const REGISTRY: &[Tool] = &[
    Tool {
        name: "search_events",
        description: "Search bear community events (festivals, runs, parties, weeks). Filter by country, type or name.",
        table: "events", base: "active=eq.true",
        select: "id,name,city,country,start_date,end_date,type,link",
        order: "start_date.asc",
        filters: &[
            ("country", "country", "ilike", "Country name, partial match (e.g. \"Spain\")."),
            ("type",    "type",    "eq",    "Exact event type."),
            ("q",       "name",    "ilike", "Free text matched against the event name."),
        ],
    },
    Tool {
        name: "list_places",
        description: "Bear-friendly places: saunas, bars, campgrounds, leather bars. Filter by country, city or place_type.",
        table: "places", base: "active=eq.true",
        select: "id,name,city,country,place_type,website,address",
        order: "name.asc",
        filters: &[
            ("country",    "country",    "ilike", "Country, partial match."),
            ("city",       "city",       "ilike", "City, partial match."),
            ("place_type", "place_type", "eq",    "e.g. sauna-bathhouse, leather-bar, party-venue."),
        ],
    },
    Tool {
        name: "current_title_holders",
        description: "Currently-reigning bear title holders across competitions.",
        table: "current_title_holders", base: "",
        select: "*", order: "",
        filters: &[],
    },
    Tool {
        name: "list_clubs",
        description: "Bear clubs and brotherhoods worldwide. Filter by country or city.",
        table: "clubs", base: "active=eq.true",
        select: "id,name,city,country,club_type,website", order: "name.asc",
        filters: &[
            ("country", "country", "ilike", "Country, partial match."),
            ("city",    "city",    "ilike", "City, partial match."),
        ],
    },
    Tool {
        name: "list_creators",
        description: "Bear creators: musicians, DJs, authors, illustrators, filmmakers, performers. Filter by creator_type or country.",
        table: "creators", base: "active=eq.true",
        select: "id,name,creator_type,city,country,website", order: "name.asc",
        filters: &[
            ("creator_type", "creator_type", "eq",    "e.g. musician, dj, author, illustrator, filmmaker."),
            ("country",      "country",      "ilike", "Country, partial match."),
        ],
    },
    Tool {
        name: "list_campaigns",
        description: "Bear community fundraising campaigns (privacy-protected ones excluded). Filter by cause.",
        table: "campaigns", base: "active=eq.true&privacy_mode=eq.false",
        select: "id,name,org,cause,goal,raised,currency,donate_url,link", order: "urgent.desc",
        filters: &[("cause", "cause", "eq", "e.g. HIV/AIDS, Refugees & Safety, Elders & Seniors.")],
    },
    Tool {
        name: "list_digital_spaces",
        description: "Online bear spaces: dating apps, Discords, podcasts, media. Filter by space_type or country.",
        table: "digital_spaces", base: "active=eq.true",
        select: "id,name,space_type,description,url,country", order: "member_count.desc.nullslast",
        filters: &[
            ("space_type", "space_type", "eq",    "e.g. dating-app, discord-server, podcast, bear-media."),
            ("country",    "country",    "ilike", "Country, partial match."),
        ],
    },
];

fn input_schema(t: &Tool) -> Value {
    let mut props = serde_json::Map::new();
    for (param, _col, _op, desc) in t.filters {
        props.insert((*param).to_string(), json!({"type": "string", "description": desc}));
    }
    props.insert("limit".into(), json!({"type": "integer", "description": "Max rows (default 50, max 200)."}));
    json!({"type": "object", "properties": props, "additionalProperties": false})
}

fn tool_descriptors() -> Value {
    let list: Vec<Value> = REGISTRY.iter().map(|t| json!({
        "name": t.name,
        "description": t.description,
        "inputSchema": input_schema(t),
    })).collect();
    json!({ "tools": list })
}

/// Execute a tool: build a safe PostgREST query from the spec + arguments, fetch JSON.
async fn call_tool(db: &SupabaseClient, name: &str, args: &Value) -> Result<Value, String> {
    let tool = REGISTRY.iter().find(|t| t.name == name)
        .ok_or_else(|| format!("unknown tool: {name}"))?;

    let mut parts: Vec<String> = Vec::new();
    if !tool.base.is_empty() { parts.push(tool.base.to_string()); }
    for (param, col, op, _desc) in tool.filters {
        if let Some(v) = args.get(param).and_then(Value::as_str) {
            if v.is_empty() { continue; }
            let value = if *op == "ilike" { format!("*{v}*") } else { v.to_string() };
            // clause() percent-encodes the value (injection-safe); drop its leading '&'.
            parts.push(clause(col, op, &value).trim_start_matches('&').to_string());
        }
    }
    parts.push(format!("select={}", tool.select));
    if !tool.order.is_empty() { parts.push(format!("order={}", tool.order)); }
    let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(50).clamp(1, 200);
    parts.push(format!("limit={limit}"));

    let url = format!("{}/rest/v1/{}?{}", db.url, tool.table, parts.join("&"));
    let rows: Vec<Value> = db.get_json(&url).await.map_err(|e| e.to_string())?;
    Ok(json!({
        "content": [ { "type": "text", "text": serde_json::to_string_pretty(&rows).unwrap_or_default() } ],
        "structuredContent": { "rows": rows }
    }))
}

fn ok(id: Value, result: Value) -> Value { json!({"jsonrpc": "2.0", "id": id, "result": result}) }
fn rpc_err(id: Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

/// `POST /mcp` — JSON-RPC 2.0 dispatcher for the MCP methods we support.
pub async fn mcp_handler(State(db): State<SupabaseClient>, Json(req): Json<Value>) -> Response {
    let method = req.get("method").and_then(Value::as_str).unwrap_or_default();

    // Notifications (no `id`) are acknowledged with 202 and no body.
    let Some(id) = req.get("id").cloned() else {
        return StatusCode::ACCEPTED.into_response();
    };

    let resp = match method {
        "initialize" => {
            let pv = req.get("params").and_then(|p| p.get("protocolVersion"))
                .and_then(Value::as_str).unwrap_or(PROTOCOL_VERSION);
            ok(id, json!({
                "protocolVersion": pv,
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "bearings", "version": env!("CARGO_PKG_VERSION") },
                "instructions": "Read-only access to the Bearings gay-bear community directory. Call tools/list to discover queries."
            }))
        }
        "ping" => ok(id, json!({})),
        "tools/list" => ok(id, tool_descriptors()),
        "tools/call" => {
            let params = req.get("params").cloned().unwrap_or_else(|| json!({}));
            let name = params.get("name").and_then(Value::as_str).unwrap_or_default();
            let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
            // Tool failures are returned as an MCP result with isError=true (not a
            // protocol error), so the client surfaces them to the model.
            let result = match call_tool(&db, name, &args).await {
                Ok(r) => r,
                Err(e) => json!({"content": [{"type": "text", "text": format!("Error: {e}")}], "isError": true}),
            };
            ok(id, result)
        }
        _ => rpc_err(id, -32601, "method not found"),
    };
    Json(resp).into_response()
}

/// `GET /mcp` — we don't offer a server-initiated SSE stream; say so per spec.
pub async fn mcp_get() -> StatusCode { StatusCode::METHOD_NOT_ALLOWED }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_exposes_tools_with_schemas() {
        let arr = tool_descriptors()["tools"].as_array().unwrap().clone();
        assert!(arr.len() >= 5);
        for t in &arr {
            assert!(t["name"].is_string());
            assert!(t["inputSchema"]["properties"]["limit"].is_object());
        }
    }

    #[test]
    fn tool_names_are_unique() {
        let mut names: Vec<&str> = REGISTRY.iter().map(|t| t.name).collect();
        let n = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), n, "duplicate tool name in REGISTRY");
    }
}
