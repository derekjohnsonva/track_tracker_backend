use super::utils::Item;
use super::utils::{add_item, delete_item, get_item, get_items};
use aws_sdk_dynamodb::{self, types::AttributeValue, Client};
use axum::routing::{delete, get, post};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub const TABLE_NAME: &str = "athlete_events";
pub const ATHLETE_ID_KEY: &str = "athlete_id";
pub const EVENT_ID_KEY: &str = "event_id";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct AthleteEvent {
    athlete_id: Uuid,
    event_id: Uuid,
}

impl Item for AthleteEvent {
    fn table_name() -> &'static str {
        TABLE_NAME
    }

    fn partition_key_name() -> &'static str {
        ATHLETE_ID_KEY
    }

    fn from_hashmap(map: HashMap<String, AttributeValue>) -> Option<Self> {
        let athlete_id = map.get(ATHLETE_ID_KEY)?.as_s().ok()?;
        let athlete_id = Uuid::parse_str(athlete_id).ok()?;

        let event_id = map.get(EVENT_ID_KEY)?.as_s().ok()?;
        let event_id = Uuid::parse_str(event_id).ok()?;

        Some(Self {
            athlete_id,
            event_id,
        })
    }

    fn into_hashmap(self) -> HashMap<String, AttributeValue> {
        let mut map = HashMap::new();
        map.insert(
            ATHLETE_ID_KEY.to_string(),
            AttributeValue::S(self.athlete_id.to_string()),
        );
        map.insert(
            EVENT_ID_KEY.to_string(),
            AttributeValue::S(self.event_id.to_string()),
        );
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_athlete_event_into_hashmap() {
        let athlete_event = AthleteEvent {
            athlete_id: Uuid::new_v4(),
            event_id: Uuid::new_v4(),
        };
        let cloned_athlete_event = athlete_event.clone();
        let hashmap = cloned_athlete_event.into_hashmap();
        let athlete_event2 = AthleteEvent::from_hashmap(hashmap).unwrap();
        assert_eq!(athlete_event, athlete_event2);
    }
}
