pub mod api;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // #[error("Failed to construct request URL")]
    // Url(#[from] url::ParseError),
    #[error("Failed to submit request")]
    Request(#[from] reqwest::Error),
    #[error("enka.network query failed")]
    Response(String),
    #[error("Failed to deserialize response")]
    Deserialization(#[from] serde::de::value::Error),
    #[error("Failed to parse Json")]
    Json(#[from] serde_json::Error),
}

#[cfg(all(feature = "logging", debug_assertions))]
macro_rules! log {
    ($msg:literal $(, $args:expr)*) => {
        {
            println!($msg $(, $args)*);
        }
    };
    ($expr:expr) => {{
        let x = $expr;
        println!("{x:?}");
        x
    }};
}

#[cfg(not(all(feature = "logging", debug_assertions)))]
macro_rules! log {
    ($msg:literal $(, $args:expr)*) => {};
    ($expr:expr) => {
        $expr
    };
}

// TODO: user-provided redis client

pub use self::r#async::*;
mod r#async {
    use super::{Error, Result, api};
    use reqwest::{Client, header::HeaderValue};
    use std::collections::HashMap;

    async fn fetch_json<T: serde::de::DeserializeOwned>(
        endpoint: &str,
        user_agent: Option<HeaderValue>,
        req_client: Option<&Client>,
    ) -> Result<T> {
        let request = log!({
            use reqwest::{Method, Request, Url, header};
            let mut r = Request::new(
                Method::GET,
                Url::parse("https://enka.network/")
                    .unwrap()
                    .join(endpoint)
                    .unwrap(),
            );
            r.headers_mut().insert(
                header::USER_AGENT,
                user_agent.unwrap_or_else(|| {
                    HeaderValue::from_static(concat!("enka-rs/", env!("GIT_HASH")))
                }),
            );
            r
        });
        let response = log!(
            req_client
                .unwrap_or(&Client::new())
                .execute(request)
                .await?
        );
        let status = response.status();
        if status.is_success() {
            let text = response.text().await?;
            serde_json::from_str::<T>(&text)
                .inspect_err(|_| {
                    #[allow(unused_variables)]
                    if let Ok(pretty_json) = serde_json::from_str::<serde_json::Value>(&text) {
                        log!("{}", serde_json::to_string_pretty(&pretty_json).unwrap());
                    } else {
                        log!("Invalid JSON response: {text}");
                    }
                })
                .map_err(Error::Json)
        } else {
            let error_message = match status.as_u16() {
                400 => "Bad Request: Wrong UID format",
                404 => "Not Found: Player does not exist (MHY server response)",
                424 => "Failed Dependency: Game maintenance or broken after update",
                429 => "Too Many Requests: Rate-limited (by enka server or MHY server)",
                500 => "Internal Server Error: General server issue",
                503 => "Service Unavailable: Possible major failure on enka end",
                _ => status.canonical_reason().unwrap_or("Unknown Error"),
            };
            #[allow(unused_variables)]
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| String::from("Failed to retrieve error body"));
            log!("Response Body: {text}");
            Err(Error::Response(format!("HTTP {status}: {error_message}")))
        }
    }

    pub async fn get_player(
        uid: u64,
        info_only: bool,
        user_agent: Option<HeaderValue>,
        req_client: Option<&Client>,
    ) -> Result<(api::player::info::Info, Option<Vec<api::AvatarInfo>>)> {
        let url = &format!("/api/uid/{uid}/{}", if info_only { "?info" } else { "" });

        if info_only {
            fetch_json::<api::player::info::Response>(url, user_agent, req_client)
                .await
                .map(|api::player::info::Response(v)| (v, None))
        } else {
            fetch_json::<api::player::Response>(url, user_agent, req_client)
                .await
                .map(|v| (v.info, v.avatar_info_list))
        }
    }

    pub async fn get_profile(
        username: &str,
        user_agent: Option<HeaderValue>,
        req_client: Option<&Client>,
    ) -> Result<api::profile::info::Info> {
        fetch_json(
            &format!("api/profile/{username}/?format=json"),
            user_agent,
            req_client,
        )
        .await
    }

    pub async fn get_hoyos(
        username: &str,
        user_agent: Option<HeaderValue>,
        req_client: Option<&Client>,
    ) -> Result<HashMap<String, api::profile::hoyo::Hoyo>> {
        fetch_json(
            &format!("api/profile/{username}/hoyos"),
            user_agent,
            req_client,
        )
        .await
    }

    pub async fn get_hoyo(
        username: &str,
        hash: &api::profile::hoyo::Hash,
        user_agent: Option<HeaderValue>,
        req_client: Option<&Client>,
    ) -> Result<api::profile::hoyo::Hoyo> {
        fetch_json(
            &format!("api/profile/{username}/hoyos/{hash}/?format=json"),
            user_agent,
            req_client,
        )
        .await
    }

    pub async fn get_builds(
        username: &str,
        hash: &api::profile::hoyo::Hash,
        user_agent: Option<HeaderValue>,
        req_client: Option<&Client>,
    ) -> Result<HashMap<api::AvatarId, Vec<api::profile::hoyo::build::Build>>> {
        fetch_json(
            &format!("api/profile/{username}/hoyos/{hash}/builds"),
            user_agent,
            req_client,
        )
        .await
    }

    pub async fn get_build(
        username: &str,
        hash: &api::profile::hoyo::Hash,
        build_id: u64,
        user_agent: Option<HeaderValue>,
        req_client: Option<&Client>,
    ) -> Result<api::profile::hoyo::build::Build> {
        fetch_json(
            &format!("api/profile/{username}/hoyos/{hash}/builds/{build_id}"),
            user_agent,
            req_client,
        )
        .await
    }

    #[cfg(feature = "stateful")]
    #[derive(Debug)]
    pub struct Wrapper<'a> {
        pub(crate) user_agent: &'a Option<HeaderValue>,
        pub(crate) req_client: &'a Option<Client>,
    }

    #[cfg(feature = "stateful")]
    impl Wrapper<'_> {
        pub async fn get_player(
            &self,
            uid: u64,
            info_only: bool,
        ) -> Result<(api::player::info::Info, Option<Vec<api::AvatarInfo>>)> {
            get_player(
                uid,
                info_only,
                self.user_agent.clone(),
                self.req_client.as_ref(),
            )
            .await
        }

        pub async fn get_profile(&self, username: &str) -> Result<api::profile::info::Info> {
            get_profile(username, self.user_agent.clone(), self.req_client.as_ref()).await
        }

        pub async fn get_hoyos(
            &self,
            username: &str,
        ) -> Result<HashMap<String, api::profile::hoyo::Hoyo>> {
            get_hoyos(username, self.user_agent.clone(), self.req_client.as_ref()).await
        }

        pub async fn get_hoyo(
            &self,
            username: &str,
            hash: &api::profile::hoyo::Hash,
        ) -> Result<api::profile::hoyo::Hoyo> {
            get_hoyo(
                username,
                hash,
                self.user_agent.clone(),
                self.req_client.as_ref(),
            )
            .await
        }

        pub async fn get_builds(
            &self,
            username: &str,
            hash: &api::profile::hoyo::Hash,
        ) -> Result<HashMap<api::AvatarId, Vec<api::profile::hoyo::build::Build>>> {
            get_builds(
                username,
                hash,
                self.user_agent.clone(),
                self.req_client.as_ref(),
            )
            .await
        }

        pub async fn get_build(
            &self,
            username: &str,
            hash: &api::profile::hoyo::Hash,
            build_id: u64,
        ) -> Result<api::profile::hoyo::build::Build> {
            get_build(
                username,
                hash,
                build_id,
                self.user_agent.clone(),
                self.req_client.as_ref(),
            )
            .await
        }
    }
}
