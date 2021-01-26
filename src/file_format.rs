use anyhow::{Error, Result};
use librespot_core::spotify_id::FileId;
use metadata::{AudioItem, FileFormat};

const SUPPORTED_FILE_FORMATS: [FileFormat; 3] = [
    FileFormat::OGG_VORBIS_160,
    FileFormat::OGG_VORBIS_96,
    FileFormat::OGG_VORBIS_320,
];

pub async fn get_file_id_and_data_rate(audio: AudioItem) -> Result<(FileId, usize)> {
    let (file_id, format) = SUPPORTED_FILE_FORMATS
        .iter()
        .find_map(|format| Some((*audio.files.get(format)?, *format)))
        .ok_or_else(|| Error::msg("Audio not available"))?;

    let bytes_per_second = match format {
        FileFormat::OGG_VORBIS_96 => 12 * 1024,
        FileFormat::OGG_VORBIS_160 => 20 * 1024,
        FileFormat::OGG_VORBIS_320 => 40 * 1024,
        FileFormat::MP3_256 => 32 * 1024,
        FileFormat::MP3_320 => 40 * 1024,
        FileFormat::MP3_160 => 20 * 1024,
        FileFormat::MP3_96 => 12 * 1024,
        FileFormat::MP3_160_ENC => 20 * 1024,
        FileFormat::MP4_128_DUAL => 16 * 1024,
        FileFormat::OTHER3 => 40 * 1024, // better some high guess than nothing
        FileFormat::AAC_160 => 20 * 1024,
        FileFormat::AAC_320 => 40 * 1024,
        FileFormat::MP4_128 => 16 * 1024,
        FileFormat::OTHER5 => 40 * 1024, // better some high guess than nothing
    };

    Ok((file_id, bytes_per_second))
}
