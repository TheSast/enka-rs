use enka_rs as lib;
use lib::gi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    println!(
        "Parsed: {:#?}",
        gi::get_profile(args.get(1).expect("Missing username"), None, None)
            .await
            .unwrap()
    );
    Ok(())
}
