use std::collections::HashMap;

use aws_sdk_dynamodb::{self, types::AttributeValue, Client};
use axum::routing::{delete, get, post};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::utils::Item;
use super::utils::{add_item, delete_item, get_item, get_items};
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
    athlete_data: AthleteData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AthleteData {
    first_name: String,
    last_name: String,
    birthday: NaiveDate,
}

impl From<AthleteData> for Athlete {
    fn from(athlete_data: AthleteData) -> Self {
        let id = Uuid::new_v4();
        Self { id, athlete_data }
    }
}

impl Item for Athlete {
    fn table_name() -> &'static str {
        TABLE_NAME
    }
    fn primary_key_name() -> &'static str {
        ID_KEY
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
            athlete_data: AthleteData {
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
            AttributeValue::S(self.athlete_data.first_name),
        );
        map.insert(
            LAST_NAME_KEY.to_string(),
            AttributeValue::S(self.athlete_data.last_name),
        );
        map.insert(
            BIRTHDAY_KEY.to_string(),
            AttributeValue::S(self.athlete_data.birthday.to_string()),
        );
        map
    }
}

pub fn athlete_routes() -> axum::Router<Client> {
    axum::Router::new()
        .route("/", post(add_item::<Athlete, AthleteData>))
        .route("/", get(get_items::<Athlete>))
        .route("/:competition_id", get(get_item::<Athlete>))
        .route("/:competition_id", delete(delete_item::<Athlete>))
}
