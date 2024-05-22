// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rodio::{dynamic_mixer, Decoder, OutputStream, Sink, Source};
use serde::{Deserialize, Serialize};
use std::{io::Cursor, time::Duration};
use tauri::State;
use std::sync::Mutex;
use bytes::Bytes;

struct Storage {
    store: Mutex<Vec<Cursor<Bytes>>>,
}

#[derive(Serialize, Deserialize)]
enum AudioPath {
    Link(String),
    Path(String),
}

#[tauri::command]
fn load_file(link: String, storage: State<Storage>) -> Result<(), Error> {
    let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(60)).build()?;
    let audio = client.get(link).send()?
        .bytes()?;
    let cursor = Cursor::new(audio.clone());
    let mut audio_sources = storage.store.lock().map_err(|_| Error::MutexPoisoned)?;

    audio_sources.push(cursor);

    Ok(())
}

// A custom error type that represents all possible in our command
#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),
    #[error("File is not valid utf8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Failed to create audio stream: {0}")]
    AudioStream(#[from] rodio::StreamError),
    #[error("Failed to create sink: {0}")]
    Play(#[from] rodio::PlayError),
    #[error("Failed to download audio: {0}")]
    Download(#[from] reqwest::Error),
    #[error("Failed to decode audio: {0}")]
    AudioDecode(#[from] rodio::decoder::DecoderError),
    #[error("Failed access storage mutex")]
    MutexPoisoned,
}

#[derive(serde::Serialize)]
struct ErrorWrapper {
    error: String,
}

// we must also implement serde::Serialize
impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let wrapper = ErrorWrapper {
            error: self.to_string(),
        };
        wrapper.serialize(serializer)
    }
}

#[tauri::command(async)]
fn play(_files: Vec<String>, storage: State<Storage>) -> Result<(), Error> {
    let audio_sources = storage.store.lock().map_err(|_| Error::MutexPoisoned)?;
    let decoded_sources = audio_sources.iter().map(|src| Decoder::new(src.clone())).flatten();

    let (controller, mixer) = dynamic_mixer::mixer::<f32>(2, 44100);
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    for source in decoded_sources.into_iter() {
        controller.add(source.convert_samples().repeat_infinite());
    }

    sink.append(mixer);
    sink.sleep_until_end();

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(Storage {
            store: Default::default(),
        })
        .invoke_handler(tauri::generate_handler![load_file, play])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
