use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use openwok_core::money::Money;
use openwok_core::types::{MenuItem, MenuItemId, Restaurant, RestaurantId, ZoneId};
use rusqlite::params;
use serde::Deserialize;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct CreateRestaurant {
    pub name: String,
    pub zone_id: ZoneId,
    pub menu: Vec<CreateMenuItem>,
}

#[derive(Deserialize)]
pub struct CreateMenuItem {
    pub name: String,
    pub price: Money,
}

fn row_to_restaurant(
    conn: &rusqlite::Connection,
    id: &str,
    name: String,
    zone_id: &str,
    active: bool,
) -> Restaurant {
    let mut stmt = conn
        .prepare("SELECT id, name, price FROM menu_items WHERE restaurant_id = ?1")
        .unwrap();
    let menu: Vec<MenuItem> = stmt
        .query_map(params![id], |row| {
            let mid: String = row.get(0)?;
            let mname: String = row.get(1)?;
            let mprice: String = row.get(2)?;
            Ok(MenuItem {
                id: MenuItemId::from_uuid(uuid::Uuid::parse_str(&mid).unwrap()),
                name: mname,
                price: Money::from(mprice.as_str()),
                restaurant_id: RestaurantId::from_uuid(uuid::Uuid::parse_str(id).unwrap()),
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    Restaurant {
        id: RestaurantId::from_uuid(uuid::Uuid::parse_str(id).unwrap()),
        name,
        zone_id: ZoneId::from_uuid(uuid::Uuid::parse_str(zone_id).unwrap()),
        menu,
        active,
    }
}

pub async fn list(State(state): State<AppState>) -> Json<Vec<Restaurant>> {
    let conn = state.db.lock().await;
    let mut stmt = conn
        .prepare("SELECT id, name, zone_id, active FROM restaurants WHERE active = 1")
        .unwrap();
    let restaurants: Vec<Restaurant> = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let zone_id: String = row.get(2)?;
            let active: bool = row.get(3)?;
            Ok((id, name, zone_id, active))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .map(|(id, name, zone_id, active)| row_to_restaurant(&conn, &id, name, &zone_id, active))
        .collect();
    Json(restaurants)
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<RestaurantId>,
) -> Result<Json<Restaurant>, StatusCode> {
    let conn = state.db.lock().await;
    let id_str = id.to_string();
    let result = conn.query_row(
        "SELECT id, name, zone_id, active FROM restaurants WHERE id = ?1",
        params![id_str],
        |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let zone_id: String = row.get(2)?;
            let active: bool = row.get(3)?;
            Ok((id, name, zone_id, active))
        },
    );

    match result {
        Ok((id, name, zone_id, active)) => {
            Ok(Json(row_to_restaurant(&conn, &id, name, &zone_id, active)))
        }
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateRestaurant>,
) -> (StatusCode, Json<Restaurant>) {
    let conn = state.db.lock().await;
    let id = RestaurantId::new();
    let id_str = id.to_string();
    let zone_str = body.zone_id.to_string();
    let rest_name = body.name;

    // Ensure zone exists (create if not)
    conn.execute(
        "INSERT OR IGNORE INTO zones (id, name) VALUES (?1, ?2)",
        params![zone_str, format!("Zone {}", &zone_str[..8])],
    )
    .unwrap();

    conn.execute(
        "INSERT INTO restaurants (id, name, zone_id, active) VALUES (?1, ?2, ?3, 1)",
        params![id_str, rest_name, zone_str],
    )
    .unwrap();

    let menu: Vec<MenuItem> = body
        .menu
        .into_iter()
        .map(|m| {
            let mid = MenuItemId::new();
            let mid_str = mid.to_string();
            let price_str = m.price.amount().to_string();
            let item_name = m.name;
            conn.execute(
                "INSERT INTO menu_items (id, restaurant_id, name, price) VALUES (?1, ?2, ?3, ?4)",
                params![mid_str, id_str, item_name, price_str],
            )
            .unwrap();
            MenuItem {
                id: mid,
                name: item_name,
                price: m.price,
                restaurant_id: id,
            }
        })
        .collect();

    let restaurant = Restaurant {
        id,
        name: rest_name,
        zone_id: body.zone_id,
        menu,
        active: true,
    };

    (StatusCode::CREATED, Json(restaurant))
}
