#[cfg(feature = "gi")]
pub mod gi;

#[cfg(feature = "stateful")]
use reqwest::{Client, header::HeaderValue};

/// Struct holding reusable data for the different API endpoint functions while implementing them as
/// methods
#[cfg(feature = "stateful")]
#[derive(Debug)]
pub struct Wrapper {
    pub user_agent: Option<HeaderValue>,
    pub req_client: Option<Client>,
}

#[cfg(feature = "stateful")]
impl Wrapper {
    #[cfg(feature = "gi")]
    pub fn gi(&self) -> gi::Wrapper<'_> {
        let Wrapper {
            user_agent,
            req_client,
        } = self;
        gi::Wrapper {
            user_agent,
            req_client,
        }
    }
}
