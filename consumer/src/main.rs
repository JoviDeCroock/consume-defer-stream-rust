use std::{error::Error, collections::HashMap};
use serde::{Deserialize,Deserializer};
use serde_json::{Value};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum GraphQLResult {
    ExecutionResult(ExecutionResult),
    StreamedExecutionResult(StreamedExecutionResult),
}

#[derive(Debug, Deserialize)]
struct ExecutionResult {
    data: Value,
    // TODO: error
    hasNext: bool,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum IncrementalPayload {
    DeferPayload(DeferPayload),
    StreamPayload(StreamPayload)
}

#[derive(Debug, Deserialize)]
struct StreamedExecutionResult {
    hasNext: bool,
    incremental: Vec<DeferPayload>
}

#[derive(Debug, Deserialize)]
struct DeferPayload {
    data: Value,
    path: Vec<String>
}

#[derive(Debug, Deserialize)]
struct StreamPayload {
    items: Vec<Value>,
    path: Vec<String>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // let mode = "multipart/mixed";
    let mode = "text/event-stream";
    let client = reqwest::Client::new();

    let body = r#"{
        "query": "query { fastField ... on Query @defer { slowField } }"
    }"#;

    let mut resp = client.post("http://localhost:4000/graphql")
        .body(body)
        .header("Accept", mode)
        .header("Content-Type", "application/json")
        .send()
        .await?;

    while let Some(chunk) = resp.chunk().await? {
        let payload = String::from_utf8(chunk.to_vec()).ok().unwrap_or_default();
        if payload.contains("event: next") {
            let parts = payload.split("data: ").collect::<Vec<&str>>();
            let chunk = parts.get(1).unwrap().replace("\n\n", "");
            let json = serde_json::from_str::<GraphQLResult>(&chunk);
            match json {
                Ok(GraphQLResult::ExecutionResult(val)) => {
                    println!("Got initial result {:?}", val);
                },
                Ok(GraphQLResult::StreamedExecutionResult(val)) => {
                    println!("Got streamed result {:?}", val);
                }
                Err(err) => {
                    println!("failed to parese {} {:?}", chunk, err);
                
                }
            }
        }
    }

    // let body = r#"{
    //     "query": "query { alphabet @stream }"
    // }"#;

    // let mut resp = client.post("http://localhost:4000/graphql")
    //     .body(body)
    //     .header("Accept", mode)
    //     .header("Content-Type", "application/json")
    //     .send()
    //     .await?;

    // while let Some(chunk) = resp.chunk().await? {
    //     println!("Chunk: {:?}", chunk);
    // }

    Ok(())
}
