extern crate clap;
extern crate pandoc;
extern crate rayon;
extern crate regex;
#[macro_use]
extern crate serde_derive;
extern crate serde_xml_rs;
extern crate serde_yaml;

use std::fs;
use std::path::Path;

use clap::{App, Arg};
use pandoc::{InputFormat, InputKind, OutputFormat, OutputKind, Pandoc, PandocOutput};
use rayon::prelude::*;
use regex::Regex;
use serde_xml_rs::deserialize;

mod evernote {
    #[derive(Debug, Deserialize)]
    pub struct Note {
        pub title: String,
        pub content: String,
        #[serde(rename = "tag", default)]
        pub tags: Vec<String>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Library {
        #[serde(rename = "note")]
        pub notes: Vec<Note>,
    }
}

mod markdown {
    #[derive(Serialize)]
    pub struct Metadata {
        pub title: String,
        pub tags: Vec<String>,
    }

    pub struct Note {
        pub metadata: Metadata,
        pub content: String,
    }
}

const INPUT: &str = "input_file";
const OUTPUT: &str = "output_dir";

fn main() -> Result<(), std::io::Error> {
    let args = cli();
    let path_to_input = args.value_of(INPUT).expect("Clap makes this required.");
    let file_contents = fs::read(path_to_input);

    let output_dir = Path::new(args.value_of(OUTPUT).expect("Clap makes this required."));
    if !output_dir.exists() {
        fs::create_dir(&output_dir).expect(&format!(
            "Could not create directory {}",
            output_dir.to_string_lossy()
        ));
    }

    if file_contents.is_err() {
        println!("{} could not be read", path_to_input);
        return Ok(());
    }

    let file_contents = file_contents.unwrap();

    let library: evernote::Library = deserialize(file_contents.as_slice()).expect("wat");
    let notes = library.notes;

    let punctuation = Regex::new(r#"[\s:;\[\]\{\}<>=@#$%^&\*\.,\?!-'"\|\(\)/\\•]"#)
        .expect(r#"`[\s:;\[\]\{\}<>=@#$%^&\*\.,\?!-'"\|\(\)/\\•]` is a valid regex"#);

    let repeated = Regex::new(r"-{2,}").expect(r"`-{2,}` is a valid regex");

    notes.par_iter().map(convert).for_each(|note| {
        let file_name = punctuation.replace_all(&note.metadata.title, "-");
        let file_name = repeated.replace_all(&file_name, "-").into_owned();
        let output_path = output_dir.join(file_name).with_extension("md");

        println!(
            "Writing {} to {}...",
            note.metadata.title,
            output_path.to_string_lossy()
        );

        let header = serde_yaml::to_string(&note.metadata).expect("Serde works");
        let body = header + "\n---\n\n" + &note.content;

        fs::write(output_path, body).expect(&format!("Could not write {}!", note.metadata.title));
    });

    Ok(())
}

fn cli<'a>() -> clap::ArgMatches<'a> {
    App::new("evernote2md")
        .version("0.1.0")
        .author("Chris Krycho <hello@chriskrycho.com>")
        .about("Converts Evernote export files (`.enex`) to Markdown with YAML metadata")
        .arg(
            Arg::with_name(INPUT)
                .value_name("input file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name(OUTPUT)
                .value_name("output directory")
                .required(true)
                .index(2),
        )
        .get_matches()
}

fn convert(note: &evernote::Note) -> markdown::Note {
    markdown::Note {
        content: {
            let mut pandoc = Pandoc::new();
            pandoc
                .set_input_format(InputFormat::Html, vec![])
                .set_output_format(OutputFormat::Markdown, vec![])
                .set_input(InputKind::Pipe(note.content.clone()))
                .set_output(OutputKind::Pipe);

            match pandoc
                .execute()
                .expect(&format!("pandoc failed on note with title {}", note.title))
            {
                PandocOutput::ToFile(f) => {
                    panic!(format!("printed to a file: {} -- wat", f.to_string_lossy()))
                }
                PandocOutput::ToBuffer(string) => string,
            }
        },
        metadata: markdown::Metadata {
            title: note.title.clone(),
            tags: note.tags.clone(),
        },
    }
}
