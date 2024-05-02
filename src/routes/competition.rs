use std::collections::HashMap;

use aws_sdk_dynamodb::{self, types::AttributeValue, Client};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const TABLE_NAME: &'static str = "competitions";
pub const ID_KEY: &'static str = "Id";
const NAME_KEY: &'static str = "Name";
const LOCATION_KEY: &'static str = "Location";
const DATE_KEY: &'static str = "Date";

// Define your Competition struct
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Competition {
    id: Uuid,
    #[serde(flatten)]
    competition_data: CompetitionData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CompetitionData {
    name: String,
    location: String,
    date: NaiveDate,
}

impl Competition {
    fn new(name: String, location: String, date: NaiveDate) -> Self {
        let id = Uuid::new_v4();
        Self {
            id,
            competition_data: CompetitionData {
                name,
                location,
                date,
            },
        }
    }

    fn from_hashmap(map: HashMap<String, AttributeValue>) -> Option<Self> {
        let id = map.get(ID_KEY)?.as_s().unwrap();
        let id = Uuid::parse_str(id).unwrap();
        let name: String = map.get(NAME_KEY)?.as_s().unwrap().to_string();
        let location: String = map.get(LOCATION_KEY)?.as_s().unwrap().to_string();
        let date_string: String = map.get(DATE_KEY)?.as_s().unwrap().to_string();
        let date = NaiveDate::parse_from_str(&date_string, "%Y-%m-%d").unwrap();
        Some(Self {
            id,
            competition_data: CompetitionData {
                name,
                location,
                date,
            },
        })
    }
}

// Function to add a competition to the database
async fn add_competition_to_db(
    client: &Client,
    competition: Competition,
) -> Result<(), aws_sdk_dynamodb::Error> {
    let competition_av = AttributeValue::S(competition.competition_data.name);
    let location_av = AttributeValue::S(competition.competition_data.location);
    let date_av = AttributeValue::S(
        competition
            .competition_data
            .date
            .format("%Y-%m-%d")
            .to_string(),
    );
    let id_av = AttributeValue::S(competition.id.to_string());

    match client
        .put_item()
        .table_name(TABLE_NAME)
        .item(ID_KEY, id_av)
        .item(NAME_KEY, competition_av)
        .item(LOCATION_KEY, location_av)
        .item(DATE_KEY, date_av)
        .send()
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

async fn list_competitions(client: &Client) -> Result<Vec<Competition>, aws_sdk_dynamodb::Error> {
    let result = client.scan().table_name(TABLE_NAME).send().await;
    let items = result?.items.unwrap();
    let competitions: Vec<Competition> = items
        .into_iter()
        .filter_map(|item| Competition::from_hashmap(item))
        .collect();
    Ok(competitions)
}

async fn add_competition(
    State(db_client): State<Client>,
    Json(competition_data): Json<CompetitionData>,
) -> Response {
    let new_competition = Competition::new(
        competition_data.name,
        competition_data.location,
        competition_data.date,
    );
    match add_competition_to_db(&db_client, new_competition).await {
        Ok(_) => "Competition added successfully!".into_response(),
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
            .into_response(),
    }
}
// #[axum::debug_handler]
async fn get_competitions(State(db_client): State<Client>) -> Response {
    match list_competitions(&db_client).await {
        Ok(competitions) => Json(competitions).into_response(),
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
            .into_response(),
    }
}

pub fn competition_routes() -> axum::Router<Client> {
    axum::Router::new()
        .route("/", post(add_competition))
        .route("/", get(get_competitions))
}
