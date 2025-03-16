use std::{fs, path::PathBuf, str::FromStr};

use clap::{Parser, Subcommand};
use id3::{
    frame::{Picture, PictureType},
    Error, ErrorKind, Tag, TagLike,
};
//change album (batch)
//change artist (batch)
//change genre (batch)

//change title

#[derive(Debug, Parser)]
#[command(name = "mp3md")]
struct Mp3MetaDataCli {
    #[command(subcommand)]
    command: Mp3MetaDataCLiCommand,
}
#[derive(Debug, Subcommand)]
enum Mp3MetaDataCLiCommand {
    Show {
        path: String,
    },
    Edit {
        #[arg(long)]
        album: Option<String>,

        #[arg(long)]
        artist: Option<String>,

        #[arg(long)]
        coverpath: Option<String>,

        #[arg(long)]
        genre: Option<String>,

        #[arg(long)]
        duration: Option<u32>,

        #[arg(long)]
        title: Option<String>,

        #[arg(long)]
        destination: String,

        path: String,
    },
}

pub fn run_edit_metadata() {
    let args = Mp3MetaDataCli::parse();
    match args.command {
        Mp3MetaDataCLiCommand::Show { path } => show_metadata(path),
        Mp3MetaDataCLiCommand::Edit {
            album,
            artist,
            coverpath,
            genre,
            duration,
            title,
            destination,
            path,
        } => {
            let pathbuf = PathBuf::from_str(&path).expect("Invalid path");
            if pathbuf.is_dir() {
                edit_metadata_batch(
                    &album, &artist, &coverpath, &genre, &duration, &title, &path,
                );
            } else {
                edit_metadata(
                    &album, &artist, &coverpath, &genre, &duration, &title, &path,
                );
            }
        }
    }
}

fn show_metadata(path: String) {
    let tag = match Tag::read_from_path(&path) {
        Ok(tag) => tag,
        Err(Error {
            kind: ErrorKind::NoTag,
            ..
        }) => panic!("No mp3 tag is found for {}", path),
        Err(error) => panic!("Unable to read tags for {}, because: {}", path, error),
    };

    println!("title: {}", tag.title().unwrap_or("--"));
    println!("album: {}", tag.album().unwrap_or("--"));
    println!("artist: {}", tag.artist().unwrap_or("--"));
    println!("genre: {}", tag.genre().unwrap_or("--"));
}

fn edit_metadata_batch(
    album: &Option<String>,
    artist: &Option<String>,
    coverpath: &Option<String>,
    genre: &Option<String>,
    duration: &Option<u32>,
    title: &Option<String>,
    path: &String,
) {
    let path = PathBuf::from_str(&path).expect("Invalid path");

    fs::read_dir(path)
        .expect("Can't read the given path")
        .into_iter()
        .map_while(|e| e.ok())
        .filter(|e| !e.path().is_dir())
        .map(|e| e.path())
        .map_while(|e| e.to_str().map(|s| s.to_string()))
        .for_each(|s| {
            edit_metadata(album, artist, coverpath, genre, duration, title, &s);
        });
}

fn edit_metadata(
    album: &Option<String>,
    artist: &Option<String>,
    coverpath: &Option<String>,
    genre: &Option<String>,
    duration: &Option<u32>,
    title: &Option<String>,
    path: &String,
) {
    let pathbuf = PathBuf::from_str(path).unwrap();
    let name = pathbuf.components().last().unwrap().as_os_str();
    let temp_path = std::env::current_dir()
        .expect("Current dir is not accessible")
        .join(&name);

    fs::copy(&pathbuf, &temp_path).expect("Unable to copy file");

    let mut tag = match Tag::read_from_path(&temp_path.as_path()) {
        Ok(tag) => tag,
        Err(Error {
            kind: ErrorKind::NoTag,
            ..
        }) => Tag::new(),
        Err(err) => panic!("{err:?}"),
    };

    if let Some(album) = album {
        tag.set_album(album);
    }

    if let Some(artist) = artist {
        tag.set_artist(artist);
    }

    if let Some(genre) = genre {
        tag.set_genre(genre);
    }

    if let Some(title) = title {
        tag.set_title(title);
    }

    if let Some(duration) = duration {
        tag.set_duration(*duration);
    }

    if let Some(coverpath) = coverpath {
        match std::fs::read(&coverpath) {
            Ok(image_data) => {
                let path = PathBuf::from_str(&coverpath).unwrap();
                let ext = path.extension().unwrap();
                let description = String::from("");
                let mime_type = String::from("");
                let picture = Picture {
                    mime_type,
                    picture_type: PictureType::CoverFront,
                    description,
                    data: image_data,
                };
                tag.add_frame(picture);
            }
            Err(err) => println!("Invalid image path {err}"),
        }
    }

    tag.write_to_path(temp_path, id3::Version::Id3v24)
        .expect("Unable to write new tags");

    println!("{name:?} changed:");
}
