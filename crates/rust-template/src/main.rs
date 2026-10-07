//! Command-line interface of `rust-template`.

use clap::Parser;
use tracing_subscriber::EnvFilter;

/// Command-line interface of `rust-template`.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// Who to greet.
    #[arg(default_value = "world")]
    name: String,
}

fn main() -> miette::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    tracing::debug!(?cli, "parsed arguments");
    println!("{}", rust_template_core::greet(&cli.name)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        super::Cli::command().debug_assert();
    }
}
