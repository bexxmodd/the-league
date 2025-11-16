//! Binary to generate CRD YAML files from Rust CustomResource definitions.
//!
//! Run with: `cargo run --bin generate-crds`

use kube::CustomResourceExt;
use std::fs;
use std::path::Path;

use the_league::{GameResult, Standing, TheLeague};

const LEAGUE_NAME: &str = "league";

/// Generate filename for a CRD using the pattern: league.<group>.<plural>.yaml
fn generate_crd_filename(group: &str, plural: &str) -> String {
    format!(
        "{}.{}.{}.yaml",
        LEAGUE_NAME,
        group.replace('.', "_"),
        plural
    )
}

/// Generate and write a CRD to the specified directory
fn generate_crd_file<T: CustomResourceExt>(
    _crd_type: std::marker::PhantomData<T>,
    output_dir: &Path,
) -> anyhow::Result<String> {
    // Ensure output directory exists
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    let crd = T::crd();
    let yaml = serde_yaml::to_string(&crd)?;
    let filename = generate_crd_filename(&crd.spec.group, &crd.spec.names.plural);
    let file_path = output_dir.join(&filename);
    fs::write(&file_path, yaml)?;
    Ok(filename)
}

/// Generate all CRD files
fn generate_all_crds(output_dir: &Path) -> anyhow::Result<Vec<String>> {
    let mut generated_files = Vec::new();

    // Generate CRD for TheLeague
    let filename = generate_crd_file(std::marker::PhantomData::<TheLeague>, output_dir)?;
    println!("✓ Generated {}/{}", output_dir.display(), filename);
    generated_files.push(filename);

    // Generate CRD for Standing
    let filename = generate_crd_file(std::marker::PhantomData::<Standing>, output_dir)?;
    println!("✓ Generated {}/{}", output_dir.display(), filename);
    generated_files.push(filename);

    // Generate CRD for GameResult
    let filename = generate_crd_file(std::marker::PhantomData::<GameResult>, output_dir)?;
    println!("✓ Generated {}/{}", output_dir.display(), filename);
    generated_files.push(filename);

    Ok(generated_files)
}

fn main() -> anyhow::Result<()> {
    // Ensure standard directory exists (GatewayAPI-style structure)
    let standard_dir = Path::new("Config/crds/standard");
    if !standard_dir.exists() {
        fs::create_dir_all(standard_dir)?;
    }

    generate_all_crds(standard_dir)?;

    println!("\nAll CRDs generated successfully!");
    println!("Apply them with: kubectl apply -k Config/crds/");
    println!("Or directly: kubectl apply -f Config/crds/standard/");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_generate_crd_filename() {
        let filename = generate_crd_filename("bexxmodd.com", "theleagues");
        assert_eq!(filename, "league.bexxmodd_com.theleagues.yaml");

        let filename = generate_crd_filename("league.bexxmodd.com", "theleagues");
        assert_eq!(filename, "league.league_bexxmodd_com.theleagues.yaml");

        let filename = generate_crd_filename("example.com", "resources");
        assert_eq!(filename, "league.example_com.resources.yaml");
    }

    #[test]
    fn test_generate_crd_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path();

        // Generate TheLeague CRD
        let filename =
            generate_crd_file(std::marker::PhantomData::<TheLeague>, output_dir).unwrap();

        // Check filename format
        assert!(filename.starts_with("league."));
        assert!(filename.ends_with(".yaml"));
        assert!(filename.contains("theleagues"));

        // Check file exists
        let file_path = output_dir.join(&filename);
        assert!(file_path.exists(), "CRD file should be created");

        // Check file content is valid YAML
        let content = fs::read_to_string(&file_path).unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();
        assert_eq!(parsed["kind"].as_str(), Some("CustomResourceDefinition"));
        // The group should be "league.bexxmodd.com" based on the CRD definition
        let group = parsed["spec"]["group"].as_str().unwrap();
        assert!(
            group.contains("bexxmodd.com"),
            "Group should contain bexxmodd.com"
        );
    }

    #[test]
    fn test_generate_all_crds() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path();

        let generated_files = generate_all_crds(output_dir).unwrap();

        // Should generate 3 files
        assert_eq!(generated_files.len(), 3);

        // Check all files exist
        for filename in &generated_files {
            let file_path = output_dir.join(filename);
            assert!(file_path.exists(), "File {} should exist", filename);
            assert!(
                filename.ends_with(".yaml"),
                "File should have .yaml extension"
            );
            assert!(
                filename.starts_with("league."),
                "File should start with 'league.'"
            );
        }

        // Check specific filenames contain expected resource names
        let filenames_str = generated_files.join(" ");
        assert!(
            filenames_str.contains("theleagues"),
            "Should contain theleagues"
        );
        assert!(
            filenames_str.contains("standings"),
            "Should contain standings"
        );
        assert!(
            filenames_str.contains("gameresults"),
            "Should contain gameresults"
        );
    }

    #[test]
    fn test_crd_content_validity() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path();

        // Generate all CRDs and get the actual filenames
        let generated_files = generate_all_crds(output_dir).unwrap();

        // Verify each CRD has required fields
        let expected_kinds = vec!["TheLeague", "Standing", "GameResult"];

        for (i, filename) in generated_files.iter().enumerate() {
            let file_path = output_dir.join(filename);
            assert!(file_path.exists(), "File {} should exist", filename);

            let content = fs::read_to_string(&file_path).unwrap();
            let crd: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();

            // Check required CRD fields
            assert_eq!(
                crd["kind"].as_str(),
                Some("CustomResourceDefinition"),
                "CRD {} should have kind CustomResourceDefinition",
                filename
            );
            assert_eq!(
                crd["apiVersion"].as_str(),
                Some("apiextensions.k8s.io/v1"),
                "CRD {} should have correct apiVersion",
                filename
            );

            let group = crd["spec"]["group"].as_str().unwrap();
            assert!(
                group.contains("bexxmodd.com"),
                "CRD {} should have correct group containing bexxmodd.com, got: {}",
                filename,
                group
            );

            if i < expected_kinds.len() {
                let expected_kind = expected_kinds[i];
                assert_eq!(
                    crd["spec"]["names"]["kind"].as_str(),
                    Some(expected_kind),
                    "CRD {} should have correct kind in names",
                    filename
                );
            }
        }
    }

    #[test]
    fn test_directory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("nonexistent").join("subdir");

        // Directory shouldn't exist yet
        assert!(!output_dir.exists());

        // Generate CRDs (should create directory)
        generate_all_crds(&output_dir).unwrap();

        // Directory should now exist
        assert!(output_dir.exists(), "Output directory should be created");
        assert!(output_dir.is_dir(), "Output should be a directory");
    }
}
