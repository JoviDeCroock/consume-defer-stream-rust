use serde::{Deserialize};
use serde_json::{Value};
use json_dotpath::{DotPaths};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GraphQLResult {
    ExecutionResult(ExecutionResult),
    StreamedExecutionResult(StreamedExecutionResult),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionResult {
    data: Value,
    // TODO: error
    has_next: bool,
}

fn merge_path(path: &Vec<Value>) -> String {
    path.iter().fold(String::new(),|a, b| {
        if a.is_empty() {
            b.to_string().replace('"', "")
        } else {
            format!("{}.{}", a, b)
        }
    })
}


impl ExecutionResult {
    pub fn finalize(&mut self) -> &mut Self {
        self.has_next = false;
        self
    }

    // TODO: error-support and merging errors
    pub fn merge(&mut self, streamed_result: StreamedExecutionResult) -> &mut Self {
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
                    } else if let Value::Object(obj) = &self.data {
                        let mut execution_data = obj.clone();
                        let path = merge_path(&payload.path);

                        // TODO: this should be deep-merging
                        let _ = execution_data.dot_set(&path, &payload.data);
                        self.data = Value::Object(execution_data);
                    }
                }
                IncrementalPayload::StreamPayload(payload) => {
                    if let Value::Object(obj) = &self.data {
                        let mut execution_data = obj.clone();
                        let path = merge_path(&payload.path);

                        // TODO: this should be deep-merging
                        let _ = execution_data.dot_set(&path, payload.items.get(0).unwrap());
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
pub struct StreamedExecutionResult {
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
