use clap::Parser;
use dependency::Dependency;
use file::PackageJson;
use indicatif::MultiProgress;
use owo_colors::OwoColorize;
use select_menu::SelectMenu;
use std::error::Error;
use table::get_report_table;
use utils::colorize_number;

mod dependency;
mod file;
mod select_menu;
mod table;
mod utils;

#[derive(Parser)]
#[clap(version = "1.0", author = "Timmy")]
struct Cli {
    #[clap(short, long)]
    path: Option<String>,
    update_file: Option<bool>,
}

// binary.exe --path package.json --update-file

const DEFAULT_PATH: &str = "package.json";

struct Executor {
    file_path: String,
    update_file: bool,
}

impl Executor {
    fn new(file_path: String, update_file: bool) -> Self {
        Self {
            file_path,
            update_file,
        }
    }

    fn execute(&self) -> Result<(), Box<dyn Error>> {
        let start_time = std::time::Instant::now();
        let file_path = &self.file_path;

        let binding = std::env::current_dir()?;
        let root_name = if file_path == DEFAULT_PATH {
            binding
                .file_name()
                .expect("Cannot get file name")
                .to_str()
                .expect("Cannot convert to string")
                .to_string()
        } else {
            file_path
                .split("/")
                .next()
                .expect("Cannot get file name")
                .to_string()
        };

        println!(
            "{} {} Resolving package.json in {}",
            "[1/3]".bold().dimmed(),
            "📦",
            root_name.blue().bold()
        );

        let mut package = PackageJson::read_from_file(&self.file_path)?;

        let deps = &package.dependencies;
        let dev_deps = &package.dev_dependencies;
        let mut deps_list = Vec::new();

        if let Some(deps) = deps {
            for (name, version) in deps.as_object().expect("Cannot get dependencies") {
                let dep = Dependency::new(name.to_string(), version.to_string());
                deps_list.push(dep);
            }
        }

        if let Some(dev_deps) = dev_deps {
            for (name, version) in dev_deps.as_object().expect("Cannot get dev dependencies") {
                let dep = Dependency::new(name.to_string(), version.to_string());
                deps_list.push(dep);
            }
        }

        if deps_list.len() == 0 {
            println!("No dependencies found");
            return Ok(());
        }

        println!("Found {} dependencies", colorize_number(&deps_list.len()));

        println!(
            "{} {} Fetching latest versions...",
            "[2/3]".bold().dimmed(),
            "🔍"
        );

        let m = MultiProgress::new();

        let report_table = get_report_table(deps_list, &m);

        println!("{}", report_table.table);

        println!(
            "{} Done in {}",
            "✅",
            utils::colorize_time(&start_time.elapsed()),
        );

        if report_table.outdated_deps.len() == 0 && self.update_file {
            println!("No dependencies to update");
            return Ok(());
        }

        if self.update_file {
            let options = report_table.outdated_deps;
            let options_ref: &[&str] = &options.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
            let selected = SelectMenu::new(options_ref, "Select dependencies to update")
                .interact()
                .into_iter()
                .map(|s| options_ref[s])
                .collect::<Vec<&str>>();

            package.update_dependencies_version(selected, &self.file_path)?;
        }

        Ok(())
    }
}

fn main() {
    let args = Cli::parse();
    let file_path = args.path.unwrap_or(DEFAULT_PATH.to_string());
    let update_file = args.update_file.unwrap_or(false);

    let executor = Executor::new(file_path, update_file);
    executor.execute().expect("Cannot execute");
}
