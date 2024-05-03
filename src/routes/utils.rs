use std::collections::HashMap;

use aws_sdk_dynamodb::{self, types::AttributeValue, Client};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use uuid::Uuid;

/// A item is something that can be stored in the database
/// It must be able to convert itself into a hashmap and be created from a hashmap
///
pub trait Item {
    fn table_name() -> &'static str;
    fn primary_key_name() -> &'static str;
    fn into_hashmap(self) -> HashMap<String, AttributeValue>;
    fn from_hashmap(map: HashMap<String, AttributeValue>) -> Option<Self>
    where
        Self: Sized;
}

/// An endpoint that will return all items in the database
pub async fn get_items<T: Serialize + Item>(State(db_client): State<Client>) -> Response {
    let result = db_client.scan().table_name(T::table_name()).send().await;
    match result {
        Ok(result) => {
            let items = result.items.unwrap();
            let converted_items: Vec<T> = items
                .into_iter()
                .filter_map(|item| T::from_hashmap(item))
                .collect();
            Json(converted_items).into_response()
        }
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
            .into_response(),
    }
}

/// Endpoint that will accept a primary_key in the path and return the item that has that primary key
///
pub async fn get_item<T: Serialize + Item>(
    Path(primary_key): Path<Uuid>,
    State(db_client): State<Client>,
) -> Response {
    let result = db_client
        .get_item()
        .table_name(T::table_name())
        .key(
            T::primary_key_name(),
            AttributeValue::S(primary_key.to_string()),
        )
        .send()
        .await;
    match result {
        Ok(result) => {
            if let Some(item) = result.item {
                if let Some(item) = T::from_hashmap(item) {
                    return Json(item).into_response();
                } else {
                    // return an internal server error as well as an error message saying that the item could not be
                    // converted from the hashmap
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "Could not convert item from hashmap".to_string(),
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

/// Add an item to the database
/// The item is passed in the request body as a JSON object
/// The item takes two generic parameters, `T` and `U`
///
/// `T` is the type of the item that will be stored in the database
///
/// `U` is the type of the item that is passed in the request body
///
pub async fn add_item<T, U>(State(db_client): State<Client>, Json(item): Json<U>) -> Response
where
    T: Serialize + Item + From<U>,
{
    let item = T::from(item);
    let result = db_client
        .put_item()
        .table_name(T::table_name())
        .set_item(Some(item.into_hashmap()))
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

/// Endpoint that will try to delete an item with the given primary key
///
pub async fn delete_item<T: Serialize + Item>(
    Path(primary_key): Path<Uuid>,
    State(db_client): State<Client>,
) -> Response {
    let result = db_client
        .delete_item()
        .table_name(T::table_name())
        .key(
            T::primary_key_name(),
            AttributeValue::S(primary_key.to_string()),
        )
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
