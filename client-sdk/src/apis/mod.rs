use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ResponseContent<T> {
    pub status: reqwest::StatusCode,
    pub content: String,
    pub entity: Option<T>,
}

#[derive(Debug)]
pub enum Error<T> {
    Reqwest(reqwest::Error),
    Serde(serde_json::Error),
    Io(std::io::Error),
    ResponseError(ResponseContent<T>),
}

impl <T> fmt::Display for Error<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (module, e) = match self {
            Error::Reqwest(e) => ("reqwest", e.to_string()),
            Error::Serde(e) => ("serde", e.to_string()),
            Error::Io(e) => ("IO", e.to_string()),
            Error::ResponseError(e) => ("response", format!("status code {}", e.status)),
        };
        write!(f, "error in {}: {}", module, e)
    }
}

impl <T: fmt::Debug> error::Error for Error<T> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(match self {
            Error::Reqwest(e) => e,
            Error::Serde(e) => e,
            Error::Io(e) => e,
            Error::ResponseError(_) => return None,
        })
    }
}

impl <T> From<reqwest::Error> for Error<T> {
    fn from(e: reqwest::Error) -> Self {
        Error::Reqwest(e)
    }
}

impl <T> From<serde_json::Error> for Error<T> {
    fn from(e: serde_json::Error) -> Self {
        Error::Serde(e)
    }
}

impl <T> From<std::io::Error> for Error<T> {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

pub fn urlencode<T: AsRef<str>>(s: T) -> String {
    ::url::form_urlencoded::byte_serialize(s.as_ref().as_bytes()).collect()
}

pub fn parse_deep_object(prefix: &str, value: &serde_json::Value) -> Vec<(String, String)> {
    if let serde_json::Value::Object(object) = value {
        let mut params = vec![];

        for (key, value) in object {
            match value {
                serde_json::Value::Object(_) => params.append(&mut parse_deep_object(
                    &format!("{}[{}]", prefix, key),
                    value,
                )),
                serde_json::Value::Array(array) => {
                    for (i, value) in array.iter().enumerate() {
                        params.append(&mut parse_deep_object(
                            &format!("{}[{}][{}]", prefix, key, i),
                            value,
                        ));
                    }
                },
                serde_json::Value::String(s) => params.push((format!("{}[{}]", prefix, key), s.clone())),
                _ => params.push((format!("{}[{}]", prefix, key), value.to_string())),
            }
        }

        return params;
    }

    unimplemented!("Only objects are supported with style=deepObject")
}

pub mod cats_api;
pub mod dogs_api;
pub mod horses_api;

pub mod configuration;

use std::sync::Arc;

pub trait Api {
    fn cats_api(&self) -> &dyn cats_api::CatsApi;
    fn dogs_api(&self) -> &dyn dogs_api::DogsApi;
    fn horses_api(&self) -> &dyn horses_api::HorsesApi;
}

pub struct ApiClient {
    cats_api: Box<dyn cats_api::CatsApi>,
    dogs_api: Box<dyn dogs_api::DogsApi>,
    horses_api: Box<dyn horses_api::HorsesApi>,
}

impl ApiClient {
    pub fn new(configuration: Arc<configuration::Configuration>) -> Self {
        Self {
            cats_api: Box::new(cats_api::CatsApiClient::new(configuration.clone())),
            dogs_api: Box::new(dogs_api::DogsApiClient::new(configuration.clone())),
            horses_api: Box::new(horses_api::HorsesApiClient::new(configuration.clone())),
        }
    }
}

impl Api for ApiClient {
    fn cats_api(&self) -> &dyn cats_api::CatsApi {
        self.cats_api.as_ref()
    }
    fn dogs_api(&self) -> &dyn dogs_api::DogsApi {
        self.dogs_api.as_ref()
    }
    fn horses_api(&self) -> &dyn horses_api::HorsesApi {
        self.horses_api.as_ref()
    }
}

#[cfg(feature = "mockall")]
pub struct MockApiClient {
    pub cats_api_mock: cats_api::MockCatsApi,
    pub dogs_api_mock: dogs_api::MockDogsApi,
    pub horses_api_mock: horses_api::MockHorsesApi,
}

#[cfg(feature = "mockall")]
impl MockApiClient {
    pub fn new() -> Self {
        Self {
            cats_api_mock: cats_api::MockCatsApi::new(),
            dogs_api_mock: dogs_api::MockDogsApi::new(),
            horses_api_mock: horses_api::MockHorsesApi::new(),
        }
    }
}

#[cfg(feature = "mockall")]
impl Api for MockApiClient {
    fn cats_api(&self) -> &dyn cats_api::CatsApi {
        &self.cats_api_mock
    }
    fn dogs_api(&self) -> &dyn dogs_api::DogsApi {
        &self.dogs_api_mock
    }
    fn horses_api(&self) -> &dyn horses_api::HorsesApi {
        &self.horses_api_mock
    }
}

