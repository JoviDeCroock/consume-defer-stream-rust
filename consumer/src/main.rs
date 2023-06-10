mod result;
mod parse;

use std::{error::Error};

use crate::{result::GraphQLResult, parse::{parse_text_event_stream_chunk, StreamState, parse_multipart_stream_chunk, parse_application_json_chunk}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // TODO: delimiter based chunking for multipart vs text/event-stream
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

    let headers = resp.headers().clone();
    let response_content_type = headers.get("Content-Type").unwrap().to_str().ok().unwrap();
    let mut final_result = None;
    while let Some(chunk) = resp.chunk().await? {
        let payload = String::from_utf8(chunk.to_vec()).ok().unwrap_or_default();
        let chunk = if response_content_type.contains("text/event-stream") {
            parse_text_event_stream_chunk(payload)
        } else if response_content_type.contains("multipart/mixed") {
            parse_multipart_stream_chunk(payload)
        } else {
            parse_application_json_chunk(payload)
        };

        match chunk.state {
            StreamState::InProgress => {
                let json = serde_json::from_str::<GraphQLResult>(&chunk.payload);
                match json {
                    Ok(GraphQLResult::ExecutionResult(val)) => {
                        final_result = Some(val);
                    },
                    Ok(GraphQLResult::StreamedExecutionResult(val)) => {
                        final_result = Some(final_result.clone().unwrap().merge(val).to_owned());
                    }
                    Err(err) => {
                        println!("failed to parese {} {:?}", chunk.payload, err);
                    }
                }
            },
            StreamState::Final => {
                final_result = Some(final_result.clone().unwrap().finalize().to_owned());
            },
        }
    }
    println!("final result: {:?}", &final_result.unwrap().data);

    let body = r#"{
        "query": "query { alphabet @stream }"
    }"#;

    let mut resp = client.post("http://localhost:4000/graphql")
        .body(body)
        .header("Accept", mode)
        .header("Content-Type", "application/json")
        .send()
        .await?;

    let headers = resp.headers().clone();
    let response_content_type = headers.get("Content-Type").unwrap().to_str().ok().unwrap();
    let mut final_result = None;
    while let Some(chunk) = resp.chunk().await? {
        let payload = String::from_utf8(chunk.to_vec()).ok().unwrap_or_default();
        let chunk = if response_content_type.contains("text/event-stream") {
            parse_text_event_stream_chunk(payload)
        } else if response_content_type.contains("multipart/mixed") {
            parse_multipart_stream_chunk(payload)
        } else {
            parse_application_json_chunk(payload)
        };

        match chunk.state {
            StreamState::InProgress => {
                let json = serde_json::from_str::<GraphQLResult>(&chunk.payload);
                match json {
                    Ok(GraphQLResult::ExecutionResult(val)) => {
                        final_result = Some(val);
                    },
                    Ok(GraphQLResult::StreamedExecutionResult(val)) => {
                        final_result = Some(final_result.clone().unwrap().merge(val).to_owned());
                    }
                    Err(err) => {
                        println!("failed to parse {} {:?}", chunk.payload, err);
                    }
                }
            },
            StreamState::Final => {
                final_result = Some(final_result.clone().unwrap().finalize().to_owned());
            },
        }
    }
    println!("final result: {:?}", &final_result.unwrap().data);

    Ok(())
}
