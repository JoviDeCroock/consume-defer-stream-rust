use std::{error::Error, fmt::format};
use serde::{Deserialize};
use serde_json::{Value};
use json_dotpath::{DotPaths};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum GraphQLResult {
    ExecutionResult(ExecutionResult),
    StreamedExecutionResult(StreamedExecutionResult),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ExecutionResult {
    data: Value,
    // TODO: error
    has_next: bool,
}


impl ExecutionResult {
    fn finalize(&mut self) -> &mut Self {
        self.has_next = false;
        self
    }

    fn merge(&mut self, streamed_result: StreamedExecutionResult) -> &mut Self {
        streamed_result.incremental.iter().for_each(|incremental_payload| {
            match incremental_payload {
                IncrementalPayload::DeferPayload(payload) => {
                    if payload.path.is_empty() {
                        let deferred_data = payload.data.as_object().unwrap();

                        if let Value::Object(obj) = &self.data {
                            let mut execution_data = obj.clone();
                            deferred_data.keys().for_each(|key| {
                                let value = deferred_data.get(key).expect("Key to be present");
                                execution_data.insert(key.to_owned(), value.to_owned());
                            });
                            self.data = Value::Object(execution_data);
                        } 
                    } else {
                        // TODO: nested case
                    }
                }
                IncrementalPayload::StreamPayload(payload) => {
                    if let Value::Object(obj) = &self.data {
                        let mut execution_data = obj.clone();
                        let path = payload.path.iter().fold(String::new(),|a, b| {
                            if a.is_empty() {
                                b.to_string().replace('"', "")
                            } else {
                                format!("{}.{}", a, b)
                            }
                        });

                        execution_data.dot_set(&path, payload.items.get(0).unwrap());
                        self.data = Value::Object(execution_data);
                    } 
                }
            }
        });

        self
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum IncrementalPayload {
    DeferPayload(DeferPayload),
    StreamPayload(StreamPayload)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamedExecutionResult {
    has_next: bool,
    incremental: Vec<IncrementalPayload>
}

#[derive(Debug, Deserialize)]
struct DeferPayload {
    data: Value,
    path: Vec<Value>
}

#[derive(Debug, Deserialize)]
struct StreamPayload {
    items: Vec<Value>,
    path: Vec<Value>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // TODO: delimiter based chunking for multipart vs text/event-stream
    // TODO: handle normal results as well
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
