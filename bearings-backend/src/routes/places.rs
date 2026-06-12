//! GET /api/places — places REST endpoints. Data access lives in
//! `repositories::place_repo`.

use crate::db::SupabaseClient;
use crate::error::AppError;
use crate::repositories::place_repo::{PlaceFilter, PlaceRepository, SupabasePlaceRepository};
use axum::extract::{Path, Query, State};
use axum::Json;
use bearings_shared::models::Place;
use serde::Deserialize;

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
    let repo = SupabasePlaceRepository::new(db);
    let places = repo
        .find(PlaceFilter {
            country: params.country,
            place_type: params.place_type,
            city: params.city,
            bear_popular: params.bear_popular.unwrap_or(false),
            men_only: params.men_only.unwrap_or(false),
            limit: params.limit.unwrap_or(100).min(500),
        })
        .await?;
    Ok(Json(places))
}

/// GET /api/places/:id
pub async fn get_one(
    State(db): State<SupabaseClient>,
    Path(id): Path<i64>,
) -> Result<Json<Place>, AppError> {
    let repo = SupabasePlaceRepository::new(db);
    repo.find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Place {id} not found")))
        .map(Json)
}

/// GET /api/places/nearby?lat=x&lng=y&radius_km=50
/// Requires the `places_nearby` SQL function in Supabase (deploy/sql/places_nearby.sql).
pub async fn nearby(
    State(db): State<SupabaseClient>,
    Query(params): Query<NearbyQuery>,
) -> Result<Json<Vec<Place>>, AppError> {
    let repo = SupabasePlaceRepository::new(db);
    let places = repo
        .find_nearby(params.lat, params.lng, params.radius_km.unwrap_or(50.0))
        .await?;
    Ok(Json(places))
}

#[derive(Deserialize)]
pub struct NearbyQuery {
    pub lat: f64,
    pub lng: f64,
    pub radius_km: Option<f64>,
}
