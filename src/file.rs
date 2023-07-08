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

fn write_package_to_file<P: AsRef<Path>, V: Serialize>(
    path: P,
    package: V,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &package)?;
    Ok(())
}

pub fn update_package_dependencies_version(
    package: &mut PackageJson,
    dependencies: Vec<&str>,
) -> Result<(), Box<dyn Error>> {
    let mut deps = package.dependencies.as_ref().unwrap().clone();
    let mut dev_deps = package.dev_dependencies.as_ref().unwrap().clone();
    for dep in dependencies {
        let dep = strip_ansi(dep);
        let name = dep.split("(").next().expect("Invalid dependency name");
        let old_version = dep.split("(").last().unwrap().split("->").next().unwrap();
        let old_version = old_version.replace("\"", "");
        let latest_version = dep.split("->").last().unwrap().replace(")", "");
        let prefix = match old_version.chars().next() {
            Some('^') => "^",
            Some('~') => "~",
            _ => "",
        };
        let new_version = format!("{}{}", prefix, latest_version.trim());

        if deps.get(&name).is_some() {
            deps[&name] = serde_json::Value::String(new_version);
        } else if dev_deps.get(&name).is_some() {
            dev_deps[&name] = serde_json::Value::String(new_version);
        }
    }

    package.dependencies = Some(deps);
    package.dev_dependencies = Some(dev_deps);

    write_package_to_file("package.json", package)?;
    
    Ok(())
}
