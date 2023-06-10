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
        parts.get(1).unwrap().replace("\n\n", "")
    } else {
        String::default()
    };

    StreamedChunk { state, payload }
}

pub fn parse_multipart_stream_chunk(chunk: String) -> StreamedChunk {
    let state = if chunk.contains("event: next") {
        StreamState::InProgress
    } else {
        StreamState::Final
    };

    let payload = if state == StreamState::InProgress {
        let parts = chunk.split("data: ").collect::<Vec<&str>>();
        parts.get(1).unwrap().replace("\n\n", "")
    } else {
        String::default()
    };

    StreamedChunk { state, payload }
}

pub fn parse_application_json_chunk(chunk: String) -> StreamedChunk {
    StreamedChunk { state: StreamState::Final, payload: chunk }
}
