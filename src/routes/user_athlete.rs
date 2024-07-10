use super::utils::Item;
use aws_sdk_dynamodb::{self, types::AttributeValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub const TABLE_NAME: &str = "user_athlete";
pub const USER_ID_KEY: &str = "user_id";
pub const ATHLETE_ID_KEY: &str = "athlete_id";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserAthlete {
    user_id: Uuid,
    athlete_id: Uuid,
}
// make a new user athlete
impl UserAthlete {
    pub fn new(user_id: Uuid, athlete_id: Uuid) -> Self {
        Self {
            user_id,
            athlete_id,
        }
    }
}

impl Item for UserAthlete {
    fn table_name() -> &'static str {
        TABLE_NAME
    }

    fn partition_key_name() -> &'static str {
        USER_ID_KEY
    }

    fn from_hashmap(map: HashMap<String, AttributeValue>) -> Option<Self> {
        let user_id = map.get(USER_ID_KEY)?.as_s().ok()?;
        let user_id = Uuid::parse_str(user_id).ok()?;

        let athlete_id = map.get(ATHLETE_ID_KEY)?.as_s().ok()?;
        let athlete_id = Uuid::parse_str(athlete_id).ok()?;

        Some(Self {
            user_id,
            athlete_id,
        })
    }

    fn into_hashmap(self) -> HashMap<String, AttributeValue> {
        let mut map = HashMap::new();
        map.insert(
            USER_ID_KEY.to_string(),
            AttributeValue::S(self.user_id.to_string()),
        );
        map.insert(
            ATHLETE_ID_KEY.to_string(),
            AttributeValue::S(self.athlete_id.to_string()),
        );
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_athlete_into_hashmap() {
        let user_athlete = UserAthlete {
            user_id: Uuid::new_v4(),
            athlete_id: Uuid::new_v4(),
        };
        let cloned_user_athlete = user_athlete.clone();
        let hashmap = cloned_user_athlete.into_hashmap();
        let user_athlete2 = UserAthlete::from_hashmap(hashmap).unwrap();
        assert_eq!(user_athlete, user_athlete2);
    }
}
