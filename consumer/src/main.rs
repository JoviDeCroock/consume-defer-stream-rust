use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    let body = r#"{
        "query": "query { fastField ... on Query @defer { slowField } }"
    }"#;

    let mut resp = client.post("http://localhost:4000/graphql")
        .body(body)
        .header("Accept", "multipart/mixed")
        .header("Content-Type", "application/json")
        .send()
        .await?;

    while let Some(chunk) = resp.chunk().await? {
        println!("Chunk: {:?}", chunk);
    }

    let body = r#"{
        "query": "query { alphabet @stream }"
    }"#;

    let mut resp = client.post("http://localhost:4000/graphql")
        .body(body)
        .header("Accept", "multipart/mixed")
        .header("Content-Type", "application/json")
        .send()
        .await?;

    while let Some(chunk) = resp.chunk().await? {
        println!("Chunk: {:?}", chunk);
    }

    Ok(())
}
