use enka_rs as lib;
use lib::gi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    println!(
        "Parsed: {:#?}",
        gi::get_player(
            args.get(1)
                .expect("Missing UID")
                .parse::<u64>()
                .expect("Invalid UID"),
            args.get(2)
                .map(|a| a
                    .parse::<bool>()
                    .expect("Invalid info-only flag, use \"true\" or \"false\""))
                .unwrap_or(false),
            None,
            None
        )
        .await
        .unwrap()
    );
    Ok(())
}
