extern crate clap;
extern crate env_logger;
extern crate librespot;
extern crate rpassword;
extern crate tokio;

use clap::{App, Arg};
use librespot::{
    core::{
        authentication::Credentials, config::SessionConfig, session::Session, spotify_id::SpotifyId,
    },
    playback::{audio_backend, config::PlayerConfig, mixer, mixer::MixerConfig, player::Player},
};
use tokio::task::block_in_place;

#[tokio::main]
async fn main() {
    env_logger::init();
    let matches = App::new("Spotify Player")
        .arg(
            Arg::with_name("USERNAME")
                .short("u")
                .long("username")
                .takes_value(true)
                .required(true),
        )
        .arg(Arg::with_name("BACKEND").long("backend").takes_value(true))
        .get_matches();

    let user = matches.value_of("USERNAME").unwrap().to_string();
    let password = block_in_place(|| rpassword::prompt_password_stderr("Enter password: "))
        .expect("Cannot get password!");
    let credentials = Credentials::with_password(user, password);

    let session_config = SessionConfig::default();

    let session = Session::connect(session_config, credentials, None)
        .await
        .unwrap_or_else(|e| panic!("Cannot connect: {}", e));

    let mixer_config = MixerConfig::default();
    let mixer = (mixer::find::<String>(None).expect("No mixer found"))(Some(mixer_config));
    let backend = matches.value_of("BACKEND").map(str::to_string);
    let player_config = PlayerConfig::default();

    let (mut player, _) = Player::new(
        player_config,
        session.clone(),
        mixer.get_audio_filter(),
        || (audio_backend::find(backend).expect("No audio backend found"))(None),
    );

    let track_id =
        SpotifyId::from_uri("spotify:track:2fC30Rt5tFa6FA2jNb4fiz").expect("Invalid URI");

    player.load(track_id, true, 0);
    player.get_end_of_track_future().await;
    player.stop();
}
