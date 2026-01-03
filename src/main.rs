mod config;

use miette::Context;

use self::config::Config;

fn main() -> miette::Result<()> {
  let config =
    Config::from_env().context("failed to gather config from env")?;
  dbg!(&config);

  Ok(())
}
