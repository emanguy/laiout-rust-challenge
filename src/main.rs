mod dto;
mod rot13;

use chrono::Utc;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::{Display, Formatter};
use std::io::Write;

#[tokio::main]
async fn main() {
    let client = Client::new();
    let app_info = match dto::info_from_env() {
        Ok(info) => info,
        Err(error) => panic!(
            "Missing env var {}. Make sure applicant info is present in the environment!",
            error.0
        ),
    };

    // Retrieve instructions
    let instructions: dto::Instructions = post_to_endpoint(
        &client,
        "https://dev.laiout.app/api/applicant/getChallenge",
        &app_info,
    )
    .await
    .expect("API should be hit successfully");
    
    // Convert instructions to plaintext via the rot13 writer
    let mut writer = rot13::Rot13Writer::new(Vec::<u8>::new());
    writer
        .write_all(instructions.instructions.as_bytes())
        .expect("Should write to string successfully");
    writer.flush().expect("Should flush to buffer successfully");
    let mut decoded_string = writer.inner;

    // Calculate the nearest 30 second epoch via the current time, append to decrypted string
    let current_time = Utc::now().timestamp();
    let most_recent_30_seconds = current_time - (current_time % 30);

    write!(decoded_string, "{}", most_recent_30_seconds)
        .expect("Should append timestamp successfully");

    // Perform hash and send to final challenge endpoint
    let secret = blake3::hash(decoded_string.as_slice());
    let challenge_submission = dto::ChallengeResult {
        applicant_name: app_info.applicant_name,
        email: app_info.email,
        secret: secret.to_string(),
    };

    // Reveal the final secret
    let deserialized_response: dto::FinalSecret = post_to_endpoint(
        &client,
        "https://dev.laiout.app/api/applicant/checkChallengeSolution",
        &challenge_submission,
    )
    .await
    .expect("Should deserialize final secret");
    println!("Final response: {}", deserialized_response.secret);
}

#[derive(Debug)]
/// Reports a failure in the process of sending a request and deserializing
/// a response.
enum PostFailure {
    ReqwestError(reqwest::Error),
    SerdeError(serde_json::Error),
}

impl Display for PostFailure {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReqwestError(error) => write!(f, "Reqwest error: {}", error),
            Self::SerdeError(error) => write!(f, "Deserialization error: {}", error),
        }
    }
}

/// Posts to an endpoint returning json encoded in a string, deserializing the string in the process.
async fn post_to_endpoint<Response: DeserializeOwned, Request: Serialize>(
    client: &Client,
    url: &str,
    body: &Request,
) -> Result<Response, PostFailure> {
    let response_data: String = client
        .post(url)
        .json(body)
        .send()
        .await
        .map_err(PostFailure::ReqwestError)?
        .json()
        .await
        .map_err(PostFailure::ReqwestError)?;

    serde_json::from_str(&response_data).map_err(PostFailure::SerdeError)
}
