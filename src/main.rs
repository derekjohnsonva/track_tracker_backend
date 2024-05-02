mod routes;

use aws_config;
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

use routes::athlete;
use routes::competition;

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
) -> Result<(), SdkError<CreateTableError>> {
    let ad = aws_sdk_dynamodb::types::AttributeDefinition::builder()
        .attribute_name(partition_key)
        .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::S)
        .build()
        .expect("creating AttributeDefinition");

    let ks = aws_sdk_dynamodb::types::KeySchemaElement::builder()
        .attribute_name(partition_key)
        .key_type(aws_sdk_dynamodb::types::KeyType::Hash)
        .build()
        .expect("creating KeySchemaElement");

    let pt = aws_sdk_dynamodb::types::ProvisionedThroughput::builder()
        .read_capacity_units(10)
        .write_capacity_units(5)
        .build()
        .expect("creating ProvisionedThroughput");

    match client
        .create_table()
        .table_name(table_name)
        .key_schema(ks)
        .attribute_definitions(ad)
        .provisioned_throughput(pt)
        .send()
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

async fn check_and_create_table(client: &Client, table_name: &str, partition_key: &str) {
    // Check to see if the "competitions" table exists
    let tables = client.list_tables().send().await.unwrap();
    if tables
        .table_names
        .unwrap()
        .contains(&String::from(table_name))
    {
        println!("{} table exists", table_name);
    } else {
        /* Create table */
        println!("Creating the {} table.", table_name);
        match make_table(&client, table_name, partition_key).await {
            Err(e) => {
                println!("Got an error creating the table:");
                println!("{}", e);
                std::process::exit(1);
            }
            Ok(_) => {
                println!("Created the table.");
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
    Client::from_conf(dynamodb_local_config)
}

#[tokio::main]
async fn main() {
    let client = build_client().await;
    // Create Axum router
    check_and_create_table(&client, competition::TABLE_NAME, competition::ID_KEY).await;
    check_and_create_table(&client, athlete::TABLE_NAME, athlete::ID_KEY).await;
    let app = Router::new()
        .route("/tables", get(list_tables)) // TODO: Remove this route. Only used to test things
        .nest("/competitions", competition::competition_routes())
        .nest("/athletes", athlete::athlete_routes())
        .with_state(client);

    // Start the Axum server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
