use clap::Parser;
use serde::Deserialize;
use std::io::BufReader;
use std::fs::File;
mod ver_check;

#[derive(Parser)]
#[clap(author, about, version)]
struct Cli {
    check_lastest: String,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: Option<String>,
    version: Option<String>,
    dependencies: serde_json::Value,
    #[serde(alias = "devDependencies")]
    dev_dependencies: serde_json::Value,
}

fn main() {
    let args = Cli::parse();
    println!("🪄 App started\n");
    std::fs::read_to_string("package.json").expect("package.json not found");
    println!("🚀 Found package.json file");

    if args.check_lastest == "true" {
        read_file();
        check_dependencies();
    } else {
        read_file();
    }
}

fn read_file() {
    let file = File::open("package.json").expect("package.json not found");
    let reader = BufReader::new(file);

    let package: Package = serde_json::from_reader(reader).expect("error parsing json");

    if package.name.is_some() {
        println!("📦 Package name: {}", package.name.unwrap());
    } else {
        println!("📦 Package name: not found");
    }

    if package.version.is_some() {
        println!("Version: {}", package.version.unwrap());
    } else {
        println!("\n");
    } 

    println!(
        "Found {} dependencies",
        package
            .dependencies
            .as_object()
            .expect("unknown dependencies")
            .len()
    );
    for (key, value) in package
        .dependencies
        .as_object()
        .expect("unknown dependencies")
        .iter()
    {
        println!("{}: {}", key, value);
    }
    println!(
        "Found {} devDependencies",
        package
            .dev_dependencies
            .as_object()
            .expect("unknown devDependencies")
            .len()
    );

    for (key, value) in package
        .dev_dependencies
        .as_object()
        .expect("unknown devDependencies")
        .iter()
    {
        println!("{}: {}", key, value);
    }
}

fn check_dependencies() {
    let file = std::fs::File::open("package.json").expect("file not found");
    let reader = BufReader::new(file);
    let package: Package = serde_json::from_reader(reader).expect("error parsing json");
    ver_check::dependencies_version_check(&package.dependencies, &package.dev_dependencies)
        .unwrap();
}
