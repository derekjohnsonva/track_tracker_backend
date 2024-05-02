use std::collections::HashMap;

use aws_sdk_dynamodb::{self, types::AttributeValue, Client};

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const TABLE_NAME: &'static str = "athletes";
pub const ID_KEY: &'static str = "Id";
pub const FIRST_NAME_KEY: &'static str = "FirstName";
pub const LAST_NAME_KEY: &'static str = "LastName";
pub const BIRTHDAY_KEY: &'static str = "Birthday";

// Define your Competition struct
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Athlete {
    id: Uuid,
    #[serde(flatten)]
    competition_data: AthleteData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AthleteData {
    first_name: String,
    last_name: String,
    birthday: NaiveDate,
}

impl Athlete {
    fn new(first_name: String, last_name: String, birthday: NaiveDate) -> Self {
        let id = Uuid::new_v4();
        Self {
            id,
            competition_data: AthleteData {
                first_name,
                last_name,
                birthday,
            },
        }
    }

    fn from_hashmap(map: HashMap<String, AttributeValue>) -> Option<Self> {
        let id = map.get(ID_KEY)?.as_s().unwrap();
        let id = Uuid::parse_str(id).unwrap();
        let first_name: String = map.get(FIRST_NAME_KEY)?.as_s().unwrap().to_string();
        let last_name: String = map.get(LAST_NAME_KEY)?.as_s().unwrap().to_string();
        let birthday_string: String = map.get(BIRTHDAY_KEY)?.as_s().unwrap().to_string();
        let birthday = NaiveDate::parse_from_str(&birthday_string, "%Y-%m-%d").unwrap();
        Some(Self {
            id,
            competition_data: AthleteData {
                first_name,
                last_name,
                birthday,
            },
        })
    }
    fn into_hashmap(self) -> HashMap<String, AttributeValue> {
        let mut map = HashMap::new();
        map.insert(ID_KEY.to_string(), AttributeValue::S(self.id.to_string()));
        map.insert(
            FIRST_NAME_KEY.to_string(),
            AttributeValue::S(self.competition_data.first_name),
        );
        map.insert(
            LAST_NAME_KEY.to_string(),
            AttributeValue::S(self.competition_data.last_name),
        );
        map.insert(
            BIRTHDAY_KEY.to_string(),
            AttributeValue::S(self.competition_data.birthday.to_string()),
        );
        map
    }
}

async fn add_athlete(
    State(db_client): State<Client>,
    Json(athlete_data): Json<AthleteData>,
) -> Response {
    let new_competition = Athlete::new(
        athlete_data.first_name.clone(),
        athlete_data.last_name.clone(),
        athlete_data.birthday.clone(),
    );
    let result = db_client
        .put_item()
        .table_name(TABLE_NAME)
        .set_item(Some(new_competition.into_hashmap()))
        .send()
        .await;
    match result {
        Ok(_) => "Athlete Added successfully".into_response(),
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
            .into_response(),
    }
}
/// An endpoint that will return all athletes in the database
async fn get_athletes(State(db_client): State<Client>) -> Response {
    let result = db_client.scan().table_name(TABLE_NAME).send().await;
    match result {
        Ok(result) => {
            let items = result.items.unwrap();
            let athletes: Vec<Athlete> = items
                .into_iter()
                .filter_map(|item| Athlete::from_hashmap(item))
                .collect();
            Json(athletes).into_response()
        }
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
            .into_response(),
    }
}

/// Endpoint that will accept a athlete_id in the path and return the athlete with that id
///
async fn get_athlete(Path(athlete_id): Path<Uuid>, State(db_client): State<Client>) -> Response {
    let result = db_client
        .get_item()
        .table_name(TABLE_NAME)
        .key(ID_KEY, AttributeValue::S(athlete_id.to_string()))
        .send()
        .await;
    match result {
        Ok(result) => {
            if let Some(item) = result.item {
                if let Some(athlete) = Athlete::from_hashmap(item) {
                    return Json(athlete).into_response();
                } else {
                    // return an internal server error as well as an error message saying that the athlete could not be
                    // converted from the hashmap
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "Could not convert athlete from hashmap".to_string(),
                    )
                        .into_response();
                }
            } else {
                axum::http::StatusCode::NOT_FOUND.into_response()
            }
        }
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
            .into_response(),
    }
}

/// Endpoint that will try to delete an athlete with the given id
///
async fn delete_athlete(Path(athlete_id): Path<Uuid>, State(db_client): State<Client>) -> Response {
    let result = db_client
        .delete_item()
        .table_name(TABLE_NAME)
        .key(ID_KEY, AttributeValue::S(athlete_id.to_string()))
        .send()
        .await;
    // .map_err(|e| DynamoDbError::from(e));
    match result {
        Ok(_) => axum::http::StatusCode::OK.into_response(),
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
            .into_response(),
    }
}

pub fn athlete_routes() -> axum::Router<Client> {
    axum::Router::new()
        .route("/", post(add_athlete))
        .route("/", get(get_athletes))
        .route("/:competition_id", get(get_athlete))
        .route("/:competition_id", delete(delete_athlete))
}
