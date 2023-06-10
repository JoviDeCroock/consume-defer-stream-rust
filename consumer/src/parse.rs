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

// async function* parseMultipartMixed(
//     contentType: string,
//     response: Response
//   ): AsyncIterableIterator<ExecutionResult> {
//     const boundaryHeader = contentType.match(boundaryHeaderRe);
//     const boundary = '--' + (boundaryHeader ? boundaryHeader[1] : '-');
//     let isPreamble = true;
//     let payload: any;
//     for await (let chunk of split(streamBody(response), '\r\n' + boundary)) {
//       if (isPreamble) {
//         isPreamble = false;
//         const preambleIndex = chunk.indexOf(boundary);
//         if (preambleIndex > -1) {
//           chunk = chunk.slice(preambleIndex + boundary.length);
//         } else {
//           continue;
//         }
//       }
//       try {
//         yield (payload = JSON.parse(chunk.slice(chunk.indexOf('\r\n\r\n') + 4)));
//       } catch (error) {
//         if (!payload) throw error;
//       }
//       if (payload && payload.hasNext === false) break;
//     }
//     if (payload && payload.hasNext !== false) {
//       yield { hasNext: false };
//     }
//   }
pub fn parse_multipart_stream_chunk(chunk: String, content_type: &str) -> StreamedChunk {
    todo!()
}

pub fn parse_application_json_chunk(chunk: String) -> StreamedChunk {
    StreamedChunk { state: StreamState::Final, payload: chunk }
}
