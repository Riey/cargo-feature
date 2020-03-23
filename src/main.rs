use structopt::StructOpt;

use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use toml_edit::{Array, Document, InlineTable, Item, Value};

#[derive(StructOpt)]
struct Command {
    #[structopt(name = "crate")]
    krate: String,
    #[structopt(name = "features")]
    features: Vec<String>,
}

#[derive(Eq, PartialEq)]
enum DependencyType {
    Normal,
    Build,
    Dev,
}

enum DependencyCommand {
    Add,
    Remove,
}

#[derive(StructOpt)]
#[structopt(bin_name = "cargo")]
struct Opt {
    #[structopt(long = "manifest-path", parse(from_os_str))]
    manifest_path: Option<PathBuf>,

    #[structopt(
        short = "p",
        long,
        about = "Don't write manifest file just print to stdout"
    )]
    preview: bool,

    #[structopt(flatten)]
    command: Command,
}

fn try_process_dependency(doc: &mut Document, func: impl Fn(&mut Item, DependencyType)) {
    macro_rules! try_find {
        ($key:expr, $ty:expr) => {
            let item = doc.as_table_mut().entry($key);
            if !item.is_none() {
                func(item, $ty);
            }
        };
    }

    try_find!("dependencies", DependencyType::Normal);
    try_find!("build-dependencies", DependencyType::Normal);
    try_find!("dev-dependencies", DependencyType::Normal);
}

fn parse_feature(mut feature: &str) -> (DependencyType, DependencyCommand, &str) {
    let ty = if feature.starts_with(&['+', '-'][..]) {
        DependencyType::Normal
    } else {
        let (ty, other) = feature.split_at(1);
        feature = other;
        match ty {
            "b" => DependencyType::Build,
            "d" => DependencyType::Dev,
            "n" => DependencyType::Normal,
            _ => panic!("Unknown dependenty type: {}", ty),
        }
    };

    let command = {
        let (command, other) = feature.split_at(1);
        feature = other;
        match command {
            "+" => DependencyCommand::Add,
            "-" => DependencyCommand::Remove,
            _ => panic!("Unknown command: {}", command),
        }
    };

    (ty, command, feature)
}

fn find_feature(feature: &str) -> impl Fn(&Value) -> bool + '_ {
    move |value: &Value| value.as_str() == Some(feature)
}

fn main() {
    let opt: Opt = Opt::from_args();

    let command = opt.command;

    let manifest_path = opt
        .manifest_path
        .unwrap_or_else(|| PathBuf::from_str("Cargo.toml").expect("Cargo.toml path"));

    let manifest = std::fs::read_to_string(&manifest_path).expect("Read Cargo.toml");

    let mut document = Document::from_str(&manifest).expect("Parse Cargo.toml");

    try_process_dependency(&mut document, |item, dep_ty| {
        let dependencies_table = item.as_table_mut().expect("dependencies is not table");

        let dep = dependencies_table.entry(&command.krate);

        if dep.is_none() {
            return;
        }

        if let Some(version) = dep.as_str() {
            let mut table = InlineTable::default();
            table.get_or_insert("version", version.to_string());
            *dep.as_value_mut().unwrap() = toml_edit::decorated(table.into(), " ", "");
        }

        let features = if let Some(table) = dep.as_inline_table_mut() {
            table
                .get_or_insert("features", Value::Array(Array::default()))
                .as_array_mut()
        } else if let Some(table) = dep.as_table_mut() {
            table
                .entry("features")
                .or_insert(Item::Value(Value::Array(Array::default())))
                .as_array_mut()
        } else {
            panic!("dependency is unknown type")
        };

        let features = features.expect("features array");

        for feature in command.features.iter() {
            let (ty, command, feature) = parse_feature(feature);

            if dep_ty != ty {
                continue;
            }

            let finder = find_feature(feature);
            match command {
                DependencyCommand::Add => {
                    if !features.iter().any(finder) {
                        features.push(feature.clone());
                        features.fmt();
                    }
                }
                DependencyCommand::Remove => {
                    let pos = features.iter().position(finder);

                    if let Some(pos) = pos {
                        features.remove(pos);
                        features.fmt();
                    }
                }
            }
        }

        let dep_table = dep.as_table_like().unwrap();
        if dep_table.len() == 1 {
            let version = dep_table.get("version").unwrap().clone();
            *dep = version;
        }

        if let Some(table) = dep.as_inline_table_mut() {
            table.fmt();
        }
    });

    if opt.preview {
        println!("{}", document);
    } else {
        let mut file = std::fs::File::create(manifest_path).expect("Create manifest");
        write!(file, "{}", document).expect("Write manifest");
        file.flush().expect("Flush manifest");
    }
}
