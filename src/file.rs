use serde::Deserialize;
use serde::Serialize;

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;

use crate::utils::strip_ansi;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackageJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scripts: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "devDependencies")]
    pub dev_dependencies: Option<serde_json::Value>,
    #[serde(flatten)]
    pub other: serde_json::Value,
}

pub fn read_package_from_file<P: AsRef<Path>>(path: P) -> Result<PackageJson, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let package = serde_json::from_reader(reader)?;
    Ok(package)
}

fn write_package_to_file<P: AsRef<Path>, V: ?Sized>(
    path: P,
    package: PackageJson,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, &package)?;
    Ok(())
}

pub fn update_package_dependencies_version(
    package: PackageJson,
    dependencies: Vec<&str>,
) -> Result<(), Box<dyn Error>> {
    let mut deps = package.dependencies.unwrap();
    for dep in dependencies {
        let dep = strip_ansi(dep);
        let name = dep.split("(").next();
        let old_version = dep.split("(").last().unwrap().split("->").next().unwrap();
        let old_version = old_version.replace("\"", "");
        let latest_version = dep.split("->").last().unwrap().replace(")", "");
        let prefix = if old_version.starts_with("^") {
            "^"
        } else if old_version.starts_with("~") {
            "~"
        } else {
            ""
        };
        let new_version = format!("{}{}", prefix, latest_version);

        deps[&name.unwrap()] = serde_json::Value::String(new_version);
    }

    let package = PackageJson {
        name: package.name,
        version: package.version,
        description: package.description,
        private: package.private,
        scripts: package.scripts,
        other: package.other,
        dependencies: Some(deps),
        dev_dependencies: package.dev_dependencies,
    };
    write_package_to_file::<_, PackageJson>("package.json", package)?;
    Ok(())
}
