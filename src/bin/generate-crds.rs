//! Binary to generate CRD YAML files from Rust CustomResource definitions.
//!
//! Run with: `cargo run --bin generate-crds`

use kube::CustomResourceExt;
use std::fs;
use std::path::Path;

use the_league::{GameResult, Standing, TheLeague};

const LEAGUE_NAME: &str = "league";

fn main() -> anyhow::Result<()> {
    // Ensure standard directory exists (GatewayAPI-style structure)
    let standard_dir = Path::new("Config/crds/standard");
    if !standard_dir.exists() {
        fs::create_dir_all(standard_dir)?;
    }

    // Generate CRD for TheLeague
    let theleague_crd = TheLeague::crd();
    let theleague_yaml = serde_yaml::to_string(&theleague_crd)?;
    // Use GatewayAPI naming: group_kind.yaml
    let theleague_filename = format!(
        "{}.{}.{}.yaml",
        LEAGUE_NAME,
        theleague_crd.spec.group.replace('.', "_"),
        theleague_crd.spec.names.plural
    );
    fs::write(standard_dir.join(&theleague_filename), theleague_yaml)?;
    println!("✓ Generated Config/crds/standard/{}", theleague_filename);

    // Generate CRD for Standing
    let standing_crd = Standing::crd();
    let standing_yaml = serde_yaml::to_string(&standing_crd)?;
    let standing_filename = format!(
        "{}.{}.{}.yaml",
        LEAGUE_NAME,
        standing_crd.spec.group.replace('.', "_"),
        standing_crd.spec.names.plural
    );
    fs::write(standard_dir.join(&standing_filename), standing_yaml)?;
    println!("✓ Generated Config/crds/standard/{}", standing_filename);

    // Generate CRD for GameResult
    let gameresult_crd = GameResult::crd();
    let gameresult_yaml = serde_yaml::to_string(&gameresult_crd)?;
    let gameresult_filename = format!(
        "{}.{}.{}.yaml",
        LEAGUE_NAME,
        gameresult_crd.spec.group.replace('.', "_"),
        gameresult_crd.spec.names.plural
    );
    fs::write(standard_dir.join(&gameresult_filename), gameresult_yaml)?;
    println!("✓ Generated Config/crds/standard/{}", gameresult_filename);

    println!("\nAll CRDs generated successfully!");
    println!("Apply them with: kubectl apply -k Config/crds/");
    println!("Or directly: kubectl apply -f Config/crds/standard/");

    Ok(())
}
