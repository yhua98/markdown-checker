use std::{env::args, path::Path, process::exit};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Frontmatter {
    title: String,
    author: String,
    tags: Vec<String>,
}

fn read_file_path<P>(path: P, is_loop: bool) -> Vec<String>
where
    P: AsRef<Path>,
{
    let mut paths: Vec<String> = vec![];
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            paths.push(String::from(path.as_path().to_str().unwrap()));
        } else if path.is_dir() {
            let mut subdir = read_file_path(path.as_path(), is_loop);
            paths.append(&mut subdir);
        }
    }
    paths
}

fn main() {
    let mut args = args();
    match args.nth(1) {
        Some(dir) => {
            let paths = read_file_path(dir, true);

            let mut commonmark = markdown::Constructs::default();
            commonmark.frontmatter = true;

            for path in paths {
                let content = markdown::to_mdast(
                    &std::fs::read_to_string(&path).unwrap(),
                    &markdown::ParseOptions {
                        constructs: commonmark.clone(),
                        ..markdown::ParseOptions::default()
                    },
                )
                .unwrap();
                let child = content.children().unwrap();
                if let Some(node) = child.get(0) {
                    match node {
                        markdown::mdast::Node::Yaml(yaml) => {
                            match serde_yml::from_str::<Frontmatter>(&yaml.value) {
                                Ok(frontmatter) => {
                                    println!("{}: {:?}", path, frontmatter);
                                }
                                Err(err) => {
                                    println!("{}: {:?}", path, err);
                                    exit(1);
                                }
                            }
                        }
                        _ => {
                            eprintln!("No frontmatter content found at the beginning of \"{}\".", path);
                            exit(1);
                        }
                    }
                } else {
                    eprintln!("No frontmatter content found at the beginning of \"{}\".", path);
                    exit(1);
                }
            }
        }
        _ => {
            eprintln!("Unspecified check directory.");
            exit(1);
        }
    }
}
