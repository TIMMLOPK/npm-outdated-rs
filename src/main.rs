use clap::Parser;
use serde::Deserialize;
use std::io::BufReader;
mod ver_check;

#[derive(Parser)]
#[clap(author, about, version)]
struct Cli {
    check_lastest: bool,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    version: String,
    dependencies: serde_json::Value,
    #[serde(alias = "devDependencies")]
    dev_dependencies: serde_json::Value,
}

fn main() {
    let args = Cli::parse();
    println!("🪄 App started");
    std::fs::read_to_string("package.json").expect("package.json not found");
    println!("🚀 Found package.json file");

    if args.check_lastest {
        read_file();
        check_dependencies();
    } else {
        read_file();
    }
}

fn read_file() {
    let file = std::fs::File::open("package.json").expect("file not found");
    let reader = BufReader::new(file);

    let package: Package = serde_json::from_reader(reader).expect("error parsing json");

    println!("Name: {}", package.name);
    println!("Version: {}", package.version);

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
