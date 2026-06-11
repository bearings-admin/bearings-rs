//! Data access for the `places` table.

use async_trait::async_trait;
use bearings_shared::models::Place;
use crate::db::SupabaseClient;
use crate::error::AppError;
use super::clause;

#[derive(Debug, Default, Clone)]
pub struct PlaceFilter {
    pub country:      Option<String>,
    /// bar | leather-bar | sauna-bathhouse | campground | ...
    pub place_type:   Option<String>,
    pub city:         Option<String>,
    pub bear_popular: bool,
    pub men_only:     bool,
    pub limit:        u32,
}

#[async_trait]
pub trait PlaceRepository: Send + Sync {
    async fn find(&self, filter: PlaceFilter) -> Result<Vec<Place>, AppError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<Place>, AppError>;
    /// Places within `radius_km` of a point, via the `places_nearby` Supabase RPC.
    async fn find_nearby(&self, lat: f64, lng: f64, radius_km: f64) -> Result<Vec<Place>, AppError>;
}

pub struct SupabasePlaceRepository {
    db: SupabaseClient,
}

impl SupabasePlaceRepository {
    pub fn new(db: SupabaseClient) -> Self {
        Self { db }
    }
}

#[async_trait]
impl PlaceRepository for SupabasePlaceRepository {
    async fn find(&self, filter: PlaceFilter) -> Result<Vec<Place>, AppError> {
        let mut url = format!(
            "{}/rest/v1/places?select=*&active=eq.true&order=country.asc,city.asc&limit={}",
            self.db.url, filter.limit
        );
        if let Some(c) = filter.country    { url.push_str(&clause("country",    "eq", &c)); }
        if let Some(t) = filter.place_type { url.push_str(&clause("place_type", "eq", &t)); }
        if let Some(c) = filter.city       { url.push_str(&clause("city",       "eq", &c)); }
        if filter.bear_popular             { url.push_str(&clause("bear_popular", "eq", "true")); }
        if filter.men_only                 { url.push_str(&clause("men_only",     "eq", "true")); }
        self.db.get_json::<Vec<Place>>(&url).await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Place>, AppError> {
        let url = format!("{}/rest/v1/places?select=*&id=eq.{}&limit=1", self.db.url, id);
        let mut places: Vec<Place> = self.db.get_json(&url).await?;
        Ok(places.pop())
    }

    async fn find_nearby(&self, lat: f64, lng: f64, radius_km: f64) -> Result<Vec<Place>, AppError> {
        let body = serde_json::json!({
            "input_lat": lat,
            "input_lng": lng,
            "radius_km": radius_km
        });
        self.db.post_rpc("places_nearby", &body).await
    }
}
