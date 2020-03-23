use structopt::StructOpt;

use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use toml_edit::{Array, Document, InlineTable, Item, Value};

enum CommandType {
    Add,
    Sub,
}

impl FromStr for CommandType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "add" | "+" => Ok(CommandType::Add),
            "sub" | "-" => Ok(CommandType::Sub),
            _ => Err(format!("Unknown command: {}", s)),
        }
    }
}

#[derive(StructOpt)]
struct Command {
    #[structopt(name = "command_type", parse(try_from_str))]
    ty: CommandType,
    #[structopt(name = "crate")]
    krate: String,
    #[structopt(name = "features")]
    features: Vec<String>,
}

#[derive(StructOpt)]
#[structopt(about = "command target")]
struct CommandTarget {
    #[structopt(
        name = "build-dependency",
        short = "b",
        long,
        about = "Include build-dependency"
    )]
    build: bool,
    #[structopt(
        name = "dev-dependency",
        short = "d",
        long,
        about = "Include dev-dependency"
    )]
    dev: bool,
    #[structopt(name = "dependency", short = "n", long, about = "Include dependency")]
    normal: bool,
}

#[derive(StructOpt)]
#[structopt(bin_name = "cargo")]
struct Opt {
    #[structopt(long = "manifest-path", parse(from_os_str))]
    manifest_path: Option<PathBuf>,

    #[structopt(flatten)]
    target: CommandTarget,

    #[structopt(flatten)]
    command: Command,
}

fn try_process_dependency(doc: &mut Document, target: CommandTarget, func: impl Fn(&mut Item)) {
    macro_rules! try_find {
        ($key:expr) => {
            let item = doc.as_table_mut().entry($key);
            if !item.is_none() {
                func(item);
            }
        };
    }

    if target.normal {
        try_find!("dependencies");
    }

    if target.build {
        try_find!("build-dependencies");
    }

    if target.dev {
        try_find!("dev-dependencies");
    }
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

    try_process_dependency(&mut document, opt.target, |item| {
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
            let finder = find_feature(feature);
            match command.ty {
                CommandType::Add => {
                    if !features.iter().any(finder) {
                        features.push(feature.clone());
                        features.fmt();
                    }
                }
                CommandType::Sub => {
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

    let mut file = std::fs::File::create(manifest_path).expect("Create manifest");
    write!(file, "{}", document).expect("Write manifest");
    file.flush().expect("Flush manifest");
}
