use std::collections::HashMap;

use super::utils::Item;
use super::utils::{add_item, delete_item, get_item, get_items};
use aws_sdk_dynamodb::{self, types::AttributeValue, Client};
use axum::routing::{delete, get, post};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use uuid::Uuid;

pub const TABLE_NAME: &str = "events";
pub const ID_KEY: &str = "id";
const COMPETITION_ID_KEY: &str = "competition_id";
const ATHLETE_ID_KEY: &str = "athlete_id";
const NAME_KEY: &str = "name";
const DATE_TIME_KEY: &str = "date_time";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct Event {
    id: Uuid,
    #[serde(flatten)]
    event_data: EventData,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct EventData {
    competition_id: Uuid,
    // TODO: Change this to a list of athlete_ids
    athlete_id: Uuid,
    name: String,
    date_time: DateTime<Utc>,
}

impl From<EventData> for Event {
    fn from(event_data: EventData) -> Self {
        let id = Uuid::new_v4();
        Self { id, event_data }
    }
}

impl Item for Event {
    fn table_name() -> &'static str {
        TABLE_NAME
    }
    fn partition_key_name() -> &'static str {
        ID_KEY
    }

    fn from_hashmap(map: HashMap<String, AttributeValue>) -> Option<Self> {
        // TODO: There should be a better way to handle unwrap() here
        let id = map.get(ID_KEY)?.as_s().unwrap();
        let id = Uuid::parse_str(id).unwrap();
        let competition_id = map.get(COMPETITION_ID_KEY)?.as_s().unwrap();
        let competition_id = Uuid::parse_str(competition_id).unwrap();
        let athlete_id = map.get(ATHLETE_ID_KEY)?.as_s().unwrap();
        let athlete_id = Uuid::parse_str(athlete_id).unwrap();
        let event_name: String = map.get(NAME_KEY)?.as_s().unwrap().to_string();
        let event_date_time_string: String = map.get(DATE_TIME_KEY)?.as_s().unwrap().to_string();
        let event_date_time = DateTime::parse_from_rfc3339(&event_date_time_string)
            .unwrap()
            .with_timezone(&Utc);
        Some(Self {
            id,
            event_data: EventData {
                competition_id,
                athlete_id,
                name: event_name,
                date_time: event_date_time,
            },
        })
    }

    fn into_hashmap(self) -> HashMap<String, AttributeValue> {
        let mut map = HashMap::new();
        map.insert(ID_KEY.to_string(), AttributeValue::S(self.id.to_string()));
        map.insert(
            COMPETITION_ID_KEY.to_string(),
            AttributeValue::S(self.event_data.competition_id.to_string()),
        );
        map.insert(
            ATHLETE_ID_KEY.to_string(),
            AttributeValue::S(self.event_data.athlete_id.to_string()),
        );
        map.insert(
            NAME_KEY.to_string(),
            AttributeValue::S(self.event_data.name),
        );
        map.insert(
            DATE_TIME_KEY.to_string(),
            AttributeValue::S(self.event_data.date_time.to_rfc3339()),
        );
        map
    }
}

pub fn event_routes() -> axum::Router<Client> {
    axum::Router::new()
        .route("/", post(add_item::<Event, EventData>))
        .route("/", get(get_items::<Event>))
        .route("/:competition_id", get(get_item::<Event>))
        .route("/:competition_id", delete(delete_item::<Event>))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_into_hashmap() {
        let event = Event {
            id: Uuid::new_v4(),
            event_data: EventData {
                competition_id: Uuid::new_v4(),
                athlete_id: Uuid::new_v4(),
                name: "100m".to_string(),
                date_time: Utc::now(),
            },
        };
        let cloned_event = event.clone();
        let hashmap = cloned_event.into_hashmap();
        let event2 = Event::from_hashmap(hashmap).unwrap();
        assert_eq!(event, event2);
    }
}
