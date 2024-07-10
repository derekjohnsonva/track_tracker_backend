use std::collections::HashMap;

use aws_sdk_dynamodb::{self, types::AttributeValue, Client};

use super::utils::Item;
use super::utils::{add_item, delete_item, get_item, get_items};
use axum::routing::{delete, get, post};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const TABLE_NAME: &str = "competitions";
pub const ID_KEY: &str = "Id";
const NAME_KEY: &str = "Name";
const LOCATION_KEY: &str = "Location";
const START_DATE_KEY: &str = "StartDate";
const END_DATE_KEY: &str = "EndDate";

// Define your Competition struct
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct Competition {
    id: Uuid,
    #[serde(flatten)]
    competition_data: CompetitionData,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct CompetitionData {
    name: String,
    location: String,
    start_date: NaiveDate,
    end_date: NaiveDate,
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
    fn partition_key_name() -> &'static str {
        ID_KEY
    }

    fn from_hashmap(map: HashMap<String, AttributeValue>) -> Option<Self> {
        let id = map.get(ID_KEY)?.as_s().unwrap();
        let id = Uuid::parse_str(id).unwrap();
        let name: String = map.get(NAME_KEY)?.as_s().unwrap().to_string();
        let location: String = map.get(LOCATION_KEY)?.as_s().unwrap().to_string();
        let start_date_string: String = map.get(START_DATE_KEY)?.as_s().unwrap().to_string();
        let start_date = NaiveDate::parse_from_str(&start_date_string, "%Y-%m-%d").unwrap();
        let end_date_string: String = map.get(END_DATE_KEY)?.as_s().unwrap().to_string();
        let end_date = NaiveDate::parse_from_str(&end_date_string, "%Y-%m-%d").unwrap();
        Some(Self {
            id,
            competition_data: CompetitionData {
                name,
                location,
                start_date,
                end_date,
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
            START_DATE_KEY.to_string(),
            AttributeValue::S(self.competition_data.start_date.to_string()),
        );
        map.insert(
            END_DATE_KEY.to_string(),
            AttributeValue::S(self.competition_data.end_date.to_string()),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_competition_into_hashmap() {
        let competition = Competition {
            id: Uuid::new_v4(),
            competition_data: CompetitionData {
                name: "Test Competition".to_string(),
                location: "Test Location".to_string(),
                start_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2021, 1, 2).unwrap(),
            },
        };
        let cloned_competition = competition.clone();
        let map = competition.into_hashmap();
        let competition2 = Competition::from_hashmap(map).unwrap();
        assert_eq!(cloned_competition, competition2);
    }
}
