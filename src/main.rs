extern crate anyhow;
extern crate clap;
extern crate env_logger;
extern crate librespot_audio as audio;
extern crate librespot_core as core;
extern crate librespot_metadata as metadata;
extern crate rodio;
extern crate rpassword;
extern crate tokio;

mod file_format;
mod subfile;

use anyhow::{ensure, Context, Error, Result};
use audio::{AudioDecrypt, AudioFile};
use rodio::{Decoder, OutputStream};

use crate::subfile::Subfile;
use clap::Clap;
use core::{
    authentication::Credentials, config::SessionConfig, session::Session, spotify_id::SpotifyId,
};
use metadata::AudioItem;
use tokio::task::block_in_place;

#[derive(Clap)]
#[clap(name = "librespot-player")]
#[non_exhaustive]
struct Config {
    /// Username of your Spotify premium account
    username: String,
    /// URI of the track to play
    #[clap(default_value = "spotify:track:2fC30Rt5tFa6FA2jNb4fiz")]
    track: String,
}

async fn run() -> Result<()> {
    env_logger::init();
    let Config {
        username, track, ..
    } = Config::parse();

    let password = block_in_place(|| rpassword::prompt_password_stderr("Enter password: "))
        .context("Unable to get password")?;

    let session_config = SessionConfig::default();
    let credentials = Credentials::with_password(username, password);
    let cache = None;

    let session = Session::connect(session_config, credentials, cache)
        .await
        .context("Cannot establish session")?;

    let track_id = SpotifyId::from_uri(&track).map_err(|_| Error::msg("Invalid uri"))?;
    let audio = AudioItem::get_audio_item(&session, track_id)
        .await
        .map_err(|_| Error::msg("Cannot get audio item"))?;
    ensure!(audio.available, "Audio not available");
    ensure!(audio.duration >= 0, "Audio invalid");

    let (file_id, bytes_per_second) = file_format::get_file_id_and_data_rate(audio).await?;

    let encrypted_file = AudioFile::open(&session, file_id, bytes_per_second, true)
        .await
        .map_err(|_| Error::msg("Cannot open audio file"))?;

    encrypted_file
        .get_stream_loader_controller()
        .set_stream_mode();

    let audio_key = session
        .audio_key()
        .request(track_id, file_id)
        .await
        .map_err(|_| Error::msg("Cannot get audio key"))?;

    let decrypted_file = AudioDecrypt::new(audio_key, encrypted_file);
    let audio_file = Subfile::new(decrypted_file, 0xa7);

    let (_stream, handle) = OutputStream::try_default().context("Cannot create audio output")?;
    let sink = rodio::Sink::try_new(&handle).context("Cannot create sink")?;
    let dec = Decoder::new_vorbis(audio_file).context("Cannot decode audio")?;
    sink.append(dec);

    block_in_place(|| sink.sleep_until_end());
    Ok(())
}

#[tokio::main]
async fn main() {
    run().await.unwrap();
}
