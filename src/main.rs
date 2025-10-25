mod config;
use config::load_config;

fn main() -> anyhow::Result<()> {
    let config = load_config("ciron.toml")?;
    println!("{:#?}", config);

    Ok(())
}
