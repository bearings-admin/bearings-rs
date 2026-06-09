
use axum::extract::{Path, Query, State};
use axum::Json;
use bearings_shared::models::Place;
use serde::Deserialize;
use crate::{db::SupabaseClient, error::AppError};

#[derive(Deserialize)]
pub struct PlacesQuery {
    pub country: Option<String>,
    /// place_type: bar | leather-bar | sauna-bathhouse | campground
    pub place_type: Option<String>,
    pub city: Option<String>,
    pub bear_popular: Option<bool>,
    pub men_only: Option<bool>,
    pub limit: Option<u32>,
}

/// GET /api/places
pub async fn list(
    State(db): State<SupabaseClient>,
    Query(params): Query<PlacesQuery>,
) -> Result<Json<Vec<Place>>, AppError> {
    let limit = params.limit.unwrap_or(100).min(500);

    let mut url = format!(
        "{}/rest/v1/places?select=*&active=eq.true&order=country.asc,city.asc&limit={}",
        db.url, limit
    );

    if let Some(c) = params.country              { url.push_str(&format!("&country=eq.{}", c)); }
    if let Some(t) = params.place_type           { url.push_str(&format!("&place_type=eq.{}", t)); }
    if let Some(c) = params.city                 { url.push_str(&format!("&city=eq.{}", c)); }
    if let Some(true) = params.bear_popular      { url.push_str("&bear_popular=eq.true"); }
    if let Some(true) = params.men_only          { url.push_str("&men_only=eq.true"); }

    Ok(Json(db.get_json::<Vec<Place>>(&url).await?))
}

/// GET /api/places/:id
pub async fn get_one(
    State(db): State<SupabaseClient>,
    Path(id): Path<i64>,
) -> Result<Json<Place>, AppError> {
    let url = format!("{}/rest/v1/places?select=*&id=eq.{}&limit=1", db.url, id);
    let mut places: Vec<Place> = db.get_json(&url).await?;
    places.pop()
        .ok_or_else(|| AppError::NotFound(format!("Place {} not found", id)))
        .map(Json)
}

/// GET /api/places/nearby?lat=x&lng=y&radius_km=50
/// Returns places within radius_km. Requires the `places_nearby` SQL function
/// to be deployed to Supabase (see deploy/sql/places_nearby.sql).
pub async fn nearby(
    State(db): State<SupabaseClient>,
    Query(params): Query<NearbyQuery>,
) -> Result<Json<Vec<Place>>, AppError> {
    // Uses the places_nearby Supabase RPC function.
    // Deploy it first: run deploy/sql/places_nearby.sql in Supabase SQL editor.
    let body = serde_json::json!({
        "input_lat": params.lat,
        "input_lng": params.lng,
        "radius_km": params.radius_km.unwrap_or(50.0)
    });

    let places: Vec<Place> = db.post_rpc("places_nearby", &body).await?;
    Ok(Json(places))
}

#[derive(Deserialize)]
pub struct NearbyQuery {
    pub lat: f64,
    pub lng: f64,
    pub radius_km: Option<f64>,
}
