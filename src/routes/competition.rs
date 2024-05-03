use std::collections::HashMap;

use aws_sdk_dynamodb::{self, types::AttributeValue, Client};

use super::utils::Item;
use super::utils::{add_item, delete_item, get_item, get_items};
use axum::routing::{delete, get, post};
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

impl From<CompetitionData> for Competition {
    fn from(competition_data: CompetitionData) -> Self {
        let id = Uuid::new_v4();
        Self {
            id,
            competition_data,
        }
    }
}

impl Item for Competition {
    fn table_name() -> &'static str {
        TABLE_NAME
    }
    fn primary_key_name() -> &'static str {
        ID_KEY
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

    fn into_hashmap(self) -> HashMap<String, AttributeValue> {
        let mut map = HashMap::new();
        map.insert(ID_KEY.to_string(), AttributeValue::S(self.id.to_string()));
        map.insert(
            NAME_KEY.to_string(),
            AttributeValue::S(self.competition_data.name),
        );
        map.insert(
            LOCATION_KEY.to_string(),
            AttributeValue::S(self.competition_data.location),
        );
        map.insert(
            DATE_KEY.to_string(),
            AttributeValue::S(self.competition_data.date.to_string()),
        );
        map
    }
}

pub fn competition_routes() -> axum::Router<Client> {
    axum::Router::new()
        .route("/", post(add_item::<Competition, CompetitionData>))
        .route("/", get(get_items::<Competition>))
        .route("/:competition_id", get(get_item::<Competition>))
        .route("/:competition_id", delete(delete_item::<Competition>))
}
