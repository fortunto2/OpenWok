use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use openwok_core::money::Money;
use openwok_core::types::{MenuItem, MenuItemId, Restaurant, RestaurantId, ZoneId};
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

pub async fn list(State(state): State<AppState>) -> Json<Vec<Restaurant>> {
    let s = state.data.read().await;
    Json(s.restaurants.values().cloned().collect())
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<RestaurantId>,
) -> Result<Json<Restaurant>, StatusCode> {
    let s = state.data.read().await;
    s.restaurants
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateRestaurant>,
) -> (StatusCode, Json<Restaurant>) {
    let id = RestaurantId::new();
    let menu: Vec<MenuItem> = body
        .menu
        .into_iter()
        .map(|m| MenuItem {
            id: MenuItemId::new(),
            name: m.name,
            price: m.price,
            restaurant_id: id,
        })
        .collect();

    let restaurant = Restaurant {
        id,
        name: body.name,
        zone_id: body.zone_id,
        menu,
        active: true,
    };

    let mut s = state.data.write().await;
    s.restaurants.insert(id, restaurant.clone());
    (StatusCode::CREATED, Json(restaurant))
}

pub fn seed_restaurants(data: &mut crate::state::AppData) {
    let zone = ZoneId::new();

    let restaurants = vec![
        (
            "Pad Thai Palace",
            vec![
                ("Pad Thai", "12.99"),
                ("Tom Yum Soup", "8.99"),
                ("Green Curry", "14.99"),
            ],
        ),
        (
            "Sushi Wave",
            vec![
                ("California Roll", "10.99"),
                ("Salmon Nigiri", "13.99"),
                ("Miso Soup", "4.99"),
            ],
        ),
        (
            "Taco Libre",
            vec![
                ("Street Tacos", "9.99"),
                ("Burrito Bowl", "11.99"),
                ("Churros", "5.99"),
            ],
        ),
    ];

    for (name, items) in restaurants {
        let id = RestaurantId::new();
        let menu = items
            .into_iter()
            .map(|(item_name, price)| MenuItem {
                id: MenuItemId::new(),
                name: item_name.into(),
                price: Money::from(price),
                restaurant_id: id,
            })
            .collect();

        data.restaurants.insert(
            id,
            Restaurant {
                id,
                name: name.into(),
                zone_id: zone,
                menu,
                active: true,
            },
        );
    }
}
