use json_dotpath::DotPaths;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GraphQLResult {
    ExecutionResult(ExecutionResult),
    StreamedExecutionResult(StreamedExecutionResult),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionResult {
    pub data: Value,
    pub errors: Option<Vec<Value>>,
    has_next: bool,
}

fn merge_path(path: &[Value]) -> String {
    path.iter().fold(String::new(), |a, b| {
        if a.is_empty() {
            b.to_string().replace('"', "")
        } else {
            format!("{}.{}", a, b)
        }
    })
}

fn merge(a: &mut Value, b: Value) {
    match (a, b) {
        (a @ &mut Value::Object(_), Value::Object(b)) => {
            let a = a
                .as_object_mut()
                .expect("A mutable object reference to be obtainable for a Map");
            for (k, v) in b {
                merge(a.entry(k).or_insert(Value::Null), v);
            }
        }
        (a @ &mut Value::Array(_), Value::Array(b)) => {
            let a = a
                .as_array_mut()
                .expect("A mutable object reference to be obtainable for an Array");
            for (k, v) in b.iter().enumerate() {
                merge(a.get_mut(k).unwrap_or(&mut Value::Null), v.clone());
            }
        }
        (a, b) => *a = b,
    }
}

impl ExecutionResult {
    pub fn finalize(&mut self) -> &mut Self {
        self.has_next = false;
        self
    }

    // TODO: error-support and merging errors
    pub fn merge(&mut self, streamed_result: &StreamedExecutionResult) -> &mut Self {
        if let Some(incremental_result) = streamed_result.incremental.as_ref() {
            incremental_result
                .iter()
                .for_each(|incremental_payload| match incremental_payload {
                    IncrementalPayload::DeferPayload(payload) => {
                        if payload.path.is_empty() {
                            let deferred_data =
                                payload.data.as_object().expect("data to be an object");

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

                            let data = execution_data.dot_get::<Value>(&path);
                            if let Ok(Some(mut item)) = data {
                                merge(&mut item, payload.data.clone());
                                let _ = execution_data.dot_set(&path, item);
                                self.data = Value::Object(execution_data);
                            }
                        }

                        if let Some(mut errors) = payload.errors.clone() {
                            if let Some(mut execution_errors) = self.errors.clone() {
                                execution_errors.append(&mut errors);
                                self.errors = Some(execution_errors)
                            } else {
                                self.errors = Some(errors.clone());
                            }
                        }
                    }
                    IncrementalPayload::StreamPayload(payload) => {
                        if let Value::Object(obj) = &self.data {
                            let mut execution_data = obj.clone();
                            let path = merge_path(&payload.path);

                            let _ = execution_data.dot_set(
                                &path,
                                payload
                                    .items
                                    .get(0)
                                    .expect("the items array to carry an item"),
                            );
                            self.data = Value::Object(execution_data);
                        }

                        if let Some(mut errors) = payload.errors.clone() {
                            if let Some(mut execution_errors) = self.errors.clone() {
                                execution_errors.append(&mut errors);
                                self.errors = Some(execution_errors)
                            } else {
                                self.errors = Some(errors.clone());
                            }
                        }
                    }
                });
        }

        self
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum IncrementalPayload {
    DeferPayload(DeferPayload),
    StreamPayload(StreamPayload),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamedExecutionResult {
    pub has_next: bool,
    incremental: Option<Vec<IncrementalPayload>>,
}

#[derive(Debug, Deserialize)]
struct DeferPayload {
    data: Value,
    errors: Option<Vec<Value>>,
    _extensions: Option<Value>,
    path: Vec<Value>,
}

#[derive(Debug, Deserialize)]
struct StreamPayload {
    items: Vec<Value>,
    errors: Option<Vec<Value>>,
    _extensions: Option<Value>,
    path: Vec<Value>,
}
