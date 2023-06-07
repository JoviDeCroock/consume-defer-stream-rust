mod result;

use std::{error::Error};

use crate::result::GraphQLResult;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // TODO: delimiter based chunking for multipart vs text/event-stream
    // TODO: handle normal results as well
    // TODO: base handling on the response-content-type
    // let mode = "multipart/mixed";
    // let mode = "application/json";
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

    let mut final_result = None;
    while let Some(chunk) = resp.chunk().await? {
        let payload = String::from_utf8(chunk.to_vec()).ok().unwrap_or_default();
        if payload.contains("event: next") {
            let parts = payload.split("data: ").collect::<Vec<&str>>();
            let chunk = parts.get(1).unwrap().replace("\n\n", "");
            let json = serde_json::from_str::<GraphQLResult>(&chunk);
            match json {
                Ok(GraphQLResult::ExecutionResult(val)) => {
                    println!("Got initial result {:?}", val);
                    final_result = Some(val);
                },
                Ok(GraphQLResult::StreamedExecutionResult(val)) => {
                    println!("Got streamed result {:?}", val);
                    final_result = Some(final_result.clone().unwrap().merge(val).to_owned());
                }
                Err(err) => {
                    println!("failed to parese {} {:?}", chunk, err);
                
                }
            }
        } else {
            final_result = Some(final_result.clone().unwrap().finalize().to_owned());
        }
    }
    println!("final result: {:?}", &final_result);

    let body = r#"{
        "query": "query { alphabet @stream }"
    }"#;

    let mut resp = client.post("http://localhost:4000/graphql")
        .body(body)
        .header("Accept", mode)
        .header("Content-Type", "application/json")
        .send()
        .await?;

    let mut final_result = None;
    while let Some(chunk) = resp.chunk().await? {
        let payload = String::from_utf8(chunk.to_vec()).ok().unwrap_or_default();
        println!("{}", payload);
        if payload.contains("event: next") {
            let parts = payload.split("data: ").collect::<Vec<&str>>();
            let chunk = parts.get(1).unwrap().replace("\n\n", "");
            let json = serde_json::from_str::<GraphQLResult>(&chunk);
            match json {
                Ok(GraphQLResult::ExecutionResult(val)) => {
                    println!("Got initial result {:?}", val);
                    final_result = Some(val);
                },
                Ok(GraphQLResult::StreamedExecutionResult(val)) => {
                    println!("Got streamed result {:?}", val);
                    final_result = Some(final_result.clone().unwrap().merge(val).to_owned());
                }
                Err(err) => {
                    println!("failed to parese {} {:?}", chunk, err);
                }
            }
        } else {
            final_result = Some(final_result.clone().unwrap().finalize().to_owned());
        }
    }
    println!("final result: {:?}", &final_result);

    Ok(())
}
