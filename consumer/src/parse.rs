use regex::Regex;

#[derive(Debug, PartialEq, Eq)]
pub enum StreamState {
    Final,
    InProgress,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StreamedChunk {
    pub state: StreamState,
    pub payload: String,
}

pub fn parse_text_event_stream_chunk(chunk: String) -> StreamedChunk {
    let state = if chunk.contains("event: next") {
        StreamState::InProgress
    } else {
        StreamState::Final
    };

    let payload = if state == StreamState::InProgress {
        let parts = chunk.split("data: ").collect::<Vec<&str>>();
        parts
            .get(1)
            .expect("data to be present on an event: next")
            .replace("\n\n", "")
    } else {
        String::default()
    };

    StreamedChunk { state, payload }
}

pub fn parse_multipart_stream_chunk(chunk: String, content_type: &str) -> StreamedChunk {
    let re: Regex = Regex::new(r#"boundary="?([^=";]+)"?"#).expect("This to be a valid regex");
    let res = re.captures(content_type);

    let boundary = if let Some(captures) = res {
        if let Some(boundary_capture) = captures.get(1) {
            boundary_capture.as_str()
        } else {
            "-"
        }
    } else {
        "-"
    };
    let boundary_delimiter = format!("--{}", boundary);

    let parts = chunk.split("\r\n\r\n").collect::<Vec<&str>>();
    let sliced = parts.get(1);
    if let Some(slice) = sliced {
        let slice_parts = slice.split(boundary).collect::<Vec<&str>>();
        if let Some(slice) = slice_parts.get(0) {
            return StreamedChunk {
                state: StreamState::InProgress,
                payload: slice.trim_end().to_string(),
            };
        }
    }

    if chunk.contains(boundary_delimiter.as_str()) {
        StreamedChunk {
            state: StreamState::InProgress,
            payload: String::default(),
        }
    } else {
        StreamedChunk {
            state: StreamState::Final,
            payload: String::default(),
        }
    }
}

pub fn parse_application_json_chunk(chunk: String) -> StreamedChunk {
    StreamedChunk {
        state: StreamState::Final,
        payload: chunk,
    }
}
