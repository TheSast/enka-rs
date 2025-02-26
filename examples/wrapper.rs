use enka_rs as lib;
use lib::{Wrapper, gi::api::profile::hoyo::Hoyo};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    let username = args.get(1).expect("Missing username");
    let wrapper = Wrapper {
        user_agent: None,
        req_client: Some(reqwest::Client::new()),
    };
    println!(
        "Parsed: {:#?}",
        wrapper
            .gi()
            .get_builds(
                username,
                wrapper
                    .gi()
                    .get_hoyos(username)
                    .await?
                    .iter()
                    .find(|(_, v)| matches!(v, Hoyo::Genshin(_)))
                    .ok_or("No hoyos found")?
                    .0,
            )
            .await?
    );
    Ok(())
}
