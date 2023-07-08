use clap::Parser;
use indicatif::{HumanDuration, MultiProgress};
use owo_colors::OwoColorize;
use std::error::Error;

mod file;
mod select_menu;
mod table;
mod utils;

use crate::file::{read_package_from_file, update_package_dependencies_version};
use select_menu::select_menu;
use table::get_report_table;

#[derive(Parser)]
#[clap(version = "1.0", author = "Timmy")]
struct Cli {
    #[clap(short, long)]
    path: Option<String>,
    unstable_update_file: Option<bool>,
}

const DEFAULT_PATH: &str = "package.json";
fn main() {
    let args = Cli::parse();
    let file_path = args.path.unwrap_or(DEFAULT_PATH.to_string());
    let unstable_update_file = args.unstable_update_file.unwrap_or(false);

    execute(file_path, unstable_update_file).unwrap();
}

fn execute(file_path: String, unstable_update_file: bool) -> Result<(), Box<dyn Error>> {
    let start_time = std::time::Instant::now();
    let binding = std::env::current_dir()?;
    let root_name = binding.file_name().unwrap().to_str().unwrap();

    println!(
        "{} {} Resolving package.json in {}",
        "[1/3]".bold().dimmed(),
        "📦",
        root_name.blue().bold()
    );

    let mut package = read_package_from_file(file_path)?;
    let mut deps_list = Vec::new();

    let deps = &package.dependencies.as_ref();
    let dev_deps = &package.dev_dependencies.as_ref();

    if let Some(deps) = deps {
        for (name, version) in deps.as_object().unwrap() {
            deps_list.push((
                name.to_string(),
                version.to_string(),
                "dependencies".to_string(),
            ));
        }
    }

    if let Some(dev_deps) = dev_deps {
        for (name, version) in dev_deps.as_object().unwrap() {
            deps_list.push((
                name.to_string(),
                version.to_string(),
                "devDependencies".to_string(),
            ));
        }
    }

    if let Some(name) = &package.name {
        println!("Package name: {}", name);
    }

    if let Some(version) = &package.version {
        println!("Version: {}", version);
    }

    if deps_list.len() > 0 {
        println!("Found {} dependencies", deps_list.len());
    } else {
        println!("No dependencies found");
    }

    if let Some(dev_dependencies) = &package.dev_dependencies {
        println!(
            "Found {} dev dependencies",
            dev_dependencies.as_object().unwrap().len()
        );
    } else {
        println!("No dev dependencies found");
    }

    if deps_list.len() == 0 {
        println!("No dependencies to update");
        return Ok(());
    }

    println!(
        "{} {} Fetching latest versions...",
        "[2/3]".bold().dimmed(),
        "🔍"
    );

    let m = MultiProgress::new();

    let result_table = get_report_table(&deps_list, &m)?;

    println!("{} Done in {}", "✅", HumanDuration(start_time.elapsed()));
    println!("{}", result_table.0);

    if result_table.1.len() == 0 && unstable_update_file {
        println!("No dependencies to update");
        return Ok(());
    }

    if unstable_update_file {
        let options = result_table.1;
        let options_ref: &[&str] = &options.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
        let selected = select_menu(options_ref)
            .into_iter()
            .map(|s| options_ref[s])
            .collect::<Vec<&str>>();

        update_package_dependencies_version(&mut package, selected)?;
    }

    Ok(())
}
