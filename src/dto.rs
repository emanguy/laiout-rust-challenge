use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize)]
/// Contains information about the applicant
pub struct ApplicantInfo {
    pub applicant_name: String,
    pub email: String,
}

/// Error reporting a missing environment variable. It contains the name of the
/// missing variable.
pub struct EnvMissingError(pub String);

/// Retrieves applicant information from the environment to preserve privacy of the
/// author. The environment variables APPLICANT_NAME and APPLICANT_EMAIL must be filled.
pub fn info_from_env() -> Result<ApplicantInfo, EnvMissingError> {
    let applicant_name =
        env::var("APPLICANT_NAME").map_err(|_| EnvMissingError("APPLICANT_NAME".to_owned()))?;
    let email =
        env::var("APPLICANT_EMAIL").map_err(|_| EnvMissingError("APPLICANT_EMAIL".to_owned()))?;

    Ok(ApplicantInfo {
        applicant_name,
        email,
    })
}

#[derive(Serialize)]
/// The data structure containing the final challenge submission
pub struct ChallengeResult {
    pub applicant_name: String,
    pub email: String,
    pub secret: String,
}

#[derive(Deserialize)]
/// The data structure containing the final secret for the challenge
pub struct FinalSecret {
    pub secret: String,
}

#[derive(Deserialize)]
/// Contains the challenge information
pub struct Instructions {
    pub instructions: String,
}
