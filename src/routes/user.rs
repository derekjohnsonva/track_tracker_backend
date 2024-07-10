use std::collections::HashMap;

use super::user_athlete::UserAthlete;
use super::utils::Item;
use super::utils::{add_item, delete_item, get_item};
use aws_sdk_dynamodb::{self, types::AttributeValue, Client};
use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{delete, get, post};
use axum::Json;
use serde::{Deserialize, Serialize};

use uuid::Uuid;

pub const TABLE_NAME: &str = "users";
pub const ID_KEY: &str = "id";
pub const USERNAME_KEY: &str = "username";
pub const ATHLETES_FOLLOWING_KEY: &str = "athletes_following";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct User {
    id: Uuid,
    #[serde(flatten)]
    user_data: UserData,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct UserData {
    username: String,
    athletes_following: Vec<Uuid>,
}

impl From<UserData> for User {
    fn from(user_data: UserData) -> Self {
        let id = Uuid::new_v4();
        Self { id, user_data }
    }
}

impl Item for User {
    fn table_name() -> &'static str {
        TABLE_NAME
    }
    fn partition_key_name() -> &'static str {
        ID_KEY
    }

    fn from_hashmap(map: HashMap<String, AttributeValue>) -> Option<Self> {
        let id = map.get(ID_KEY)?.as_s().unwrap();
        let id = Uuid::parse_str(id).unwrap();
        let username: String = map.get(USERNAME_KEY)?.as_s().unwrap().to_string();
        let athletes_following: Vec<Uuid> = map
            .get(ATHLETES_FOLLOWING_KEY)?
            .as_ss()
            .unwrap()
            .iter()
            .map(|s| Uuid::parse_str(s).unwrap())
            .collect();
        Some(Self {
            id,
            user_data: UserData {
                username,
                athletes_following,
            },
        })
    }

    fn into_hashmap(self) -> HashMap<String, AttributeValue> {
        let mut map = HashMap::new();
        // collect the list of athletes_following into a list of strings
        let athletes_following: Vec<String> = self
            .user_data
            .athletes_following
            .iter()
            .map(|uuid| uuid.to_string())
            .collect();
        map.insert(ID_KEY.to_string(), AttributeValue::S(self.id.to_string()));
        map.insert(
            USERNAME_KEY.to_string(),
            AttributeValue::S(self.user_data.username),
        );
        map.insert(
            ATHLETES_FOLLOWING_KEY.to_string(),
            AttributeValue::Ss(athletes_following),
        );
        map
    }
}

async fn add_user_athlete(
    State(db_client): State<Client>,
    Path((user_id, athlete_id)): Path<(Uuid, Uuid)>,
) -> Response {
    // - Validate user_id and athlete_id (For now, no validation is needed)
    // - Create a UserAthlete object
    let user_athlete = UserAthlete::new(user_id, athlete_id);
    // - Add the UserAthlete to the table/database
    add_item::<UserAthlete, UserAthlete>(State(db_client), Json(user_athlete)).await
}

pub fn user_routes() -> axum::Router<Client> {
    axum::Router::new()
        .route("/", post(add_item::<User, UserData>))
        .route("/:id", get(get_item::<User>))
        .route("/:id", delete(delete_item::<User>))
        .route("/:user_id/follow/:athlete_id", post(add_user_athlete))
    // .route("/:user_id/follow/:athlete_id", delete(remove_user_athlete))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_user_into_hashmap() {
        let user = User {
            id: Uuid::new_v4(),
            user_data: UserData {
                username: "test".to_string(),
                athletes_following: vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()],
            },
        };
        let cloned_user = user.clone();
        let map = user.into_hashmap();
        let user2 = User::from_hashmap(map).unwrap();
        assert_eq!(cloned_user, user2);
    }
}
