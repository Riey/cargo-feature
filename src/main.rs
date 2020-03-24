use structopt::StructOpt;

use ansi_term::Color;
use pad::PadStr;
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
struct Opt {
    #[structopt(long = "manifest-path", parse(from_os_str))]
    manifest_path: Option<PathBuf>,

    #[structopt(
        short = "p",
        long,
        about = "Don't write manifest file just print to stdout"
    )]
    preview: bool,

    #[structopt(short = "i", long, about = "Don't print progress output")]
    ignore_progress: bool,

    #[structopt(flatten)]
    command: Command,
}

#[derive(StructOpt)]
#[structopt(bin_name = "cargo")]
enum CargoOpt {
    #[structopt(name = "feature")]
    Feature(Opt),
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
    let CargoOpt::Feature(opt) = CargoOpt::from_args();

    let command = opt.command;
    let ignore_progress = opt.ignore_progress;

    let manifest_path = opt
        .manifest_path
        .unwrap_or_else(|| PathBuf::from_str("Cargo.toml").expect("Cargo.toml path"));

    let metadata = {
        let mut cmd = cargo_metadata::MetadataCommand::new();
        cmd.manifest_path(&manifest_path);
        cmd.exec().expect("Run cargo-metadata")
    };

    let package = metadata
        .packages
        .into_iter()
        .find(|package| package.name == command.krate)
        .expect("Get package metadata");

    let manifest = std::fs::read_to_string(&manifest_path).expect("Read Cargo.toml");

    if command.features.is_empty() {
        println!(
            "{} features for {}",
            Color::Cyan
                .bold()
                .paint("Avaliable".pad_to_width_with_alignment(12, pad::Alignment::Right)),
            command.krate
        );

        for (feature, sub_features) in package.features {
            println!(
                "{}{:?}",
                Color::Green.paint(feature),
                sub_features
            );
        }

        return;
    }

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
            let (ty, dep_command, feature) = parse_feature(feature);

            if !package.features.contains_key(feature) {
                if !ignore_progress {
                    eprintln!(
                        "{} crate {} don't have feature {}",
                        Color::Yellow.bold().paint(
                            "Skipping".pad_to_width_with_alignment(12, pad::Alignment::Right)
                        ),
                        command.krate,
                        feature
                    );
                }
                continue;
            }

            if dep_ty != ty {
                continue;
            }

            let finder = find_feature(feature);
            match dep_command {
                DependencyCommand::Add => {
                    if !features.iter().any(finder) {
                        if !ignore_progress {
                            println!(
                                "{} feature {} to crate {}",
                                Color::Green.bold().paint(
                                    "Adding".pad_to_width_with_alignment(12, pad::Alignment::Right)
                                ),
                                feature,
                                command.krate
                            );
                        }
                        features.push(feature.clone());
                        features.fmt();
                    }
                }
                DependencyCommand::Remove => {
                    let pos = features.iter().position(finder);

                    if let Some(pos) = pos {
                        if !ignore_progress {
                            println!(
                                "{} feature {} to crate {}",
                                Color::Green.bold().paint(
                                    "Removing"
                                        .pad_to_width_with_alignment(12, pad::Alignment::Right)
                                ),
                                feature,
                                command.krate
                            );
                        }
                        features.remove(pos);
                        features.fmt();
                    }
                }
            }
        }

        if features.is_empty() {
            if let Some(table) = dep.as_table_mut() {
                table.remove("features");
            } else if let Some(table) = dep.as_inline_table_mut() {
                table.remove("features");
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
