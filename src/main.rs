mod routes;

use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::create_table::CreateTableError;
use aws_sdk_dynamodb::{self, Client};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};

use routes::{athlete, event, user};
use routes::{competition, user_athlete};
use tracing::{info, warn};

async fn list_tables(State(db_client): State<Client>) -> Response {
    let result = db_client.list_tables().send().await;
    match result {
        Ok(result) => {
            let tables = result.table_names.unwrap();
            Json(tables).into_response()
        }
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
            .into_response(),
    }
}

async fn make_table(
    client: &Client,
    table_name: &str,
    partition_key: &str,
    sort_key: Option<&str>,
) -> Result<(), SdkError<CreateTableError>> {
    // The attributeDefinition for the partition key
    let ad = aws_sdk_dynamodb::types::AttributeDefinition::builder()
        .attribute_name(partition_key)
        .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::S)
        .build()
        .expect("creating AttributeDefinition");

    let ad = if let Some(sort_key) = sort_key {
        let sort_key = aws_sdk_dynamodb::types::AttributeDefinition::builder()
            .attribute_name(sort_key)
            .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::S)
            .build()
            .expect("creating AttributeDefinition for sort key");
        vec![ad, sort_key]
    } else {
        vec![ad]
    };
    let ks = aws_sdk_dynamodb::types::KeySchemaElement::builder()
        .attribute_name(partition_key)
        .key_type(aws_sdk_dynamodb::types::KeyType::Hash)
        .build()
        .expect("creating KeySchemaElement for partition key");
    let ks = if let Some(sort_key) = sort_key {
        let sort_key = aws_sdk_dynamodb::types::KeySchemaElement::builder()
            .attribute_name(sort_key)
            .key_type(aws_sdk_dynamodb::types::KeyType::Range)
            .build()
            .expect("creating KeySchemaElement for sort key");
        vec![ks, sort_key]
    } else {
        vec![ks]
    };
    let pt = aws_sdk_dynamodb::types::ProvisionedThroughput::builder()
        .read_capacity_units(10)
        .write_capacity_units(5)
        .build()
        .expect("creating ProvisionedThroughput");

    match client
        .create_table()
        .table_name(table_name)
        .set_key_schema(Some(ks))
        .set_attribute_definitions(Some(ad))
        .provisioned_throughput(pt)
        .send()
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

async fn check_and_create_table(
    client: &Client,
    table_name: &str,
    partition_key: &str,
    sort_key: Option<&str>,
) {
    // Check to see if the "competitions" table exists
    let tables = client.list_tables().send().await.unwrap();
    if tables
        .table_names
        .unwrap()
        .contains(&String::from(table_name))
    {
        info!("{} table exists", table_name);
    } else {
        /* Create table */
        info!("Creating the {} table.", table_name);
        match make_table(client, table_name, partition_key, sort_key).await {
            Err(e) => {
                warn!("Got an error creating the table:");
                warn!("{}", e);
                std::process::exit(1);
            }
            Ok(_) => {
                info!("Created the table.");
            }
        }
    }
}

async fn build_client() -> Client {
    let config = aws_config::defaults(BehaviorVersion::latest())
        .test_credentials()
        .load()
        .await;
    let dynamodb_local_config = aws_sdk_dynamodb::config::Builder::from(&config)
        // Override the endpoint in the config to use a local dynamodb server.
        .endpoint_url(
            // DynamoDB run locally uses port 8000 by default.
            "http://localhost:8000",
        )
        .build();
    info!("Creating DynamoDB client with local config at http://localhost:8000");

    Client::from_conf(dynamodb_local_config)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Starting server");
    let client = build_client().await;
    // Create Axum router
    check_and_create_table(&client, competition::TABLE_NAME, competition::ID_KEY, None).await;
    check_and_create_table(&client, athlete::TABLE_NAME, athlete::ID_KEY, None).await;
    check_and_create_table(&client, event::TABLE_NAME, event::ID_KEY, None).await;
    check_and_create_table(&client, user::TABLE_NAME, user::ID_KEY, None).await;
    check_and_create_table(
        &client,
        user_athlete::TABLE_NAME,
        user_athlete::USER_ID_KEY,
        Some(user_athlete::ATHLETE_ID_KEY),
    )
    .await;

    let app = Router::new()
        .route("/tables", get(list_tables)) // TODO: Remove this route. Only used to test things
        .nest("/competitions", competition::competition_routes())
        .nest("/athletes", athlete::athlete_routes())
        .nest("/events", event::event_routes())
        .nest("/users", user::user_routes())
        .with_state(client);

    // Start the Axum server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
