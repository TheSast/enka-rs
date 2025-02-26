pub mod api;
pub type Err = Box<dyn std::error::Error>;

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
    use super::{Err, api};
    use reqwest::{Client, header::HeaderValue};
    use std::collections::HashMap;

    async fn fetch_json<T: serde::de::DeserializeOwned>(
        endpoint: &str,
        user_agent: Option<HeaderValue>,
        req_client: Option<&Client>,
    ) -> Result<T, Err> {
        let request = log!({
            use reqwest::{Method, Request, Url, header::HeaderName};
            let mut r = Request::new(
                Method::GET,
                Url::parse(&format!("https://enka.network{endpoint}")).unwrap(),
            );
            r.headers_mut().insert(
                HeaderName::from_bytes(b"User-Agent").unwrap(),
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
            serde_json::from_str::<T>(&text).map_err(|e| {
                match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(pretty_json) => {
                        eprintln!("{}", serde_json::to_string_pretty(&pretty_json).unwrap())
                    }
                    Err(_) => eprintln!("Invalid JSON response: {text}"),
                }
                e.into()
            })
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
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| String::from("Failed to retrieve error body"));
            eprintln!(
                "HTTP {} - {}\nResponse Body: {}",
                status, error_message, text
            );
            Err(format!("HTTP {}: {}", status, error_message).into())
        }
    }

    pub async fn get_player(
        uid: u64,
        info_only: bool,
        user_agent: Option<HeaderValue>,
        req_client: Option<&Client>,
    ) -> Result<(api::player::info::Info, Option<Vec<api::AvatarInfo>>), Err> {
        let url = &format!("/api/uid/{uid}/{}", if info_only { "?info" } else { "" });

        if info_only {
            fetch_json::<api::player::info::Info>(url, user_agent, req_client)
                .await
                .map(|v| (v, None))
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
    ) -> Result<api::profile::info::Info, Err> {
        fetch_json(
            &format!("/api/profile/{username}/?format=json"),
            user_agent,
            req_client,
        )
        .await
    }

    pub async fn get_hoyos(
        username: &str,
        user_agent: Option<HeaderValue>,
        req_client: Option<&Client>,
    ) -> Result<HashMap<String, api::profile::hoyo::Hoyo>, Err> {
        fetch_json(
            &format!("/api/profile/{username}/hoyos/"),
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
    ) -> Result<api::profile::hoyo::Hoyo, Err> {
        fetch_json(
            &format!("/api/profile/{username}/hoyos/{hash}/?format=json"),
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
    ) -> Result<HashMap<api::AvatarId, Vec<api::profile::hoyo::build::Build>>, Err> {
        fetch_json(
            &format!("/api/profile/{username}/hoyos/{hash}/builds/"),
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
    ) -> Result<api::profile::hoyo::build::Build, Err> {
        fetch_json(
            &format!("/api/profile/{username}/hoyos/{hash}/builds/{build_id}"),
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
        ) -> Result<(api::player::info::Info, Option<Vec<api::AvatarInfo>>), Err> {
            get_player(
                uid,
                info_only,
                self.user_agent.clone(),
                self.req_client.as_ref(),
            )
            .await
        }

        pub async fn get_profile(&self, username: &str) -> Result<api::profile::info::Info, Err> {
            get_profile(username, self.user_agent.clone(), self.req_client.as_ref()).await
        }

        pub async fn get_hoyos(
            &self,
            username: &str,
        ) -> Result<HashMap<String, api::profile::hoyo::Hoyo>, Err> {
            get_hoyos(username, self.user_agent.clone(), self.req_client.as_ref()).await
        }

        pub async fn get_hoyo(
            &self,
            username: &str,
            hash: &api::profile::hoyo::Hash,
        ) -> Result<api::profile::hoyo::Hoyo, Err> {
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
        ) -> Result<HashMap<api::AvatarId, Vec<api::profile::hoyo::build::Build>>, Err> {
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
        ) -> Result<api::profile::hoyo::build::Build, Err> {
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
