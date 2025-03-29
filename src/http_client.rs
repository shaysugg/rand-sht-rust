use std::collections::HashMap;

use serde::{Deserialize, Serialize};

struct Urls;

impl Urls {
    const GET: &str = "https://pie.dev/get";
    const POST: &str = "https://pie.dev/postsss";
}

#[derive(Debug)]
enum Error {
    InvalidURL,
    Network(ErrorStatus),
    Unknown,
    JsonDeserialize,
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if err.is_status() {
            err.status()
                .map(|s| Error::Network(ErrorStatus::from(s.as_u16())))
                .unwrap_or(Error::Unknown)
        } else if err.is_decode() {
            Error::JsonDeserialize
        } else {
            Error::Unknown
        }
    }
}

#[derive(Debug)]
enum ErrorStatus {
    NotFound,
    InternalServer,
    Other(u16),
}

impl ErrorStatus {
    fn from(code: u16) -> ErrorStatus {
        match code {
            404 => ErrorStatus::NotFound,
            500 => ErrorStatus::InternalServer,
            other => ErrorStatus::Other(other),
        }
    }
}

pub async fn run_http_client() {
    match get_message_back("hello world").await {
        Ok(msg) => println!("{msg}"),
        Err(err) => println!("An error accured {:?}", err),
    }

    //TODO: Errors are not get handled properly
    match post_something().await {
        Ok(_) => println!("posted successfully"),
        Err(err) => println!("An error accured {:?}", err),
    };
}

async fn get_message_back(message: &str) -> Result<String, Error> {
    let mut queries: HashMap<&str, &str> = HashMap::new();
    queries.insert("message", message);
    let url = match reqwest::Url::parse_with_params(Urls::GET, &queries) {
        Ok(url) => url,
        Err(err) => return Err(Error::InvalidURL),
    };
    //Intentionally manual decoding is used here
    reqwest::get(url)
        .await?
        .json::<HashMap<String, serde_json::Value>>()
        .await?
        .get("args")
        .and_then(|v| match v {
            serde_json::Value::Object(value) => Some(value),
            _ => None,
        })
        .ok_or(Error::JsonDeserialize)?
        .get("message")
        .and_then(|v| match v {
            serde_json::Value::String(value) => Some(value),
            _ => None,
        })
        .ok_or(Error::JsonDeserialize)
        .cloned()
}

#[derive(Serialize, Deserialize, Debug)]
struct SomthingToPost {
    message: String,
    foo: String,
}
#[derive(Deserialize, Debug)]
struct SomthingToPostResponse {
    json: SomthingToPost,
}
async fn post_something() -> Result<(), Error> {
    let client = reqwest::Client::new();
    let body = SomthingToPost {
        message: "Hello, World".to_string(),
        foo: "bar".to_string(),
    };

    client
        .post(Urls::POST)
        .json(&body)
        .send()
        .await
        .inspect(|res| println!("{:?}", res))?
        .json::<SomthingToPostResponse>()
        .await
        .map(|_| ())
        .map_err(|e| Error::from(e))
}
