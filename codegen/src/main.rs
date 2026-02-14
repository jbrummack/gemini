const URL: &str = "https://github.com/googleapis/googleapis/archive/refs/heads/master.zip";
use std::collections::HashMap;
use std::fs::{self, File, read_dir};
use std::io::{self, Read, Seek};
use std::path::Path;

use anyhow::Result;
use reqwest::blocking::Client;
use zip::ZipArchive;

fn download_and_extract_zip(url: &str, out_dir: &Path) -> Result<()> {
    let client = Client::new();

    // Start request
    let response = client.get(url).send()?.error_for_status()?;

    // The body implements Read
    let reader = response;

    // ZipArchive expects something implementing Read + Seek.
    // Since streaming sources can't seek, we must wrap with a temp file.
    let mut temp = tempfile::tempfile()?;
    io::copy(&mut reader.take(u64::MAX), &mut temp)?;
    temp.rewind()?;

    let mut archive = ZipArchive::new(temp)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if should_include(file.name()) {
            let outpath = out_dir.join(file.name());

            if file.is_dir() {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    fs::create_dir_all(parent)?;
                }

                let mut outfile = File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
            }
        }
    }

    Ok(())
}
fn should_include(filename: &str) -> bool {
    filename.contains("google/cloud/aiplatform")
        || filename.contains("google/api")
        || filename.contains("google/ai/generativelanguage")
        || filename.contains("google/type")
}

fn main() -> Result<()> {
    let url = URL;

    let out = Path::new(".");

    download_and_extract_zip(url, out)?;
    let _ = fs::create_dir("../generated")
        .inspect_err(|_e| println!("generated folder already exists!"));
    tonic_prost_build::configure()
        .out_dir("../generated")
        .build_server(false) //not needed
        .compile_well_known_types(true) //those are definitely needed
        .compile_protos(
            &[
                "./googleapis-master/google/ai/generativelanguage/v1/generative_service.proto",
                "./googleapis-master/google/cloud/aiplatform/v1/prediction_service.proto",
            ],
            &["googleapis-master"],
        )?;
    println!("Done!");
    Ok(())
}

struct LibTree {
    name: String,
    include: Option<String>,
    children: HashMap<String, Self>,
}
impl LibTree {
    fn render(&self) -> String {
        let include = self
            .include
            .as_ref()
            .map(|txt| format!(r#"include!(concat!(env!("OUT_DIR"), "{txt}"));"#))
            .unwrap_or(String::new());
        let children = self
            .children
            .iter()
            .map(|c| c.1.render())
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            r#"pub mod {} {{
            {include}
            {children}
            }}"#,
            self.name
        )
    }
}
pub fn generate_lib_rs() -> Result<()> {
    let files = read_dir("./generated")?;
    let mut tree = LibTree {
        name: String::from("google"),
        include: None,
        children: HashMap::new(),
    };
    let fnames: Vec<_> = files
        .into_iter()
        .flat_map(|f| f)
        .flat_map(|f| f.file_name().into_string())
        .collect();

    println!("{}", tree.render());

    Ok(())
}
