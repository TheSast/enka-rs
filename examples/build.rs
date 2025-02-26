use enka_rs as lib;
use lib::gi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    println!(
        "Parsed: {:#?}",
        gi::get_build(
            args.get(1).expect("Missing username"),
            args.get(2).expect("Missing hoyo hash"),
            args.get(3)
                .expect("Missing build id")
                .parse::<u64>()
                .expect("Invalid build id"),
            None,
            None
        )
        .await
        .unwrap()
    );
    Ok(())
}
