// SPDX-FileCopyrightText: © 2024 David Bliss
//
// SPDX-License-Identifier: GPL-3.0-or-later

use chrono::prelude::*;
use chrono::{DateTime, FixedOffset, TimeDelta, Utc};
use jsonpath_rust::{JsonPathFinder, JsonPathInst, JsonPathQuery, JsonPathValue};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

// TODO video::Enricher should use this class for ffprobe parsing

#[derive(Debug, Default, Clone)]
pub struct Metadata {
    pub created_at: Option<DateTime<Utc>>,

    pub width: Option<u64>, // 64?

    pub height: Option<u64>,

    pub duration: Option<TimeDelta>,

    pub container_format: Option<String>,

    pub video_codec: Option<String>,

    pub audio_codec: Option<String>,
}

pub enum Error {
    Probe(String),
    Json(String),
}

impl Metadata {
    pub fn from(path: &Path) -> Result<Metadata, Error> {
        println!("Video path: {:?}", path);

        // ffprobe is part of the ffmpeg-full flatpak extension
        let output = Command::new("/usr/bin/ffprobe")
            .arg("-v")
            .arg("quiet")
            .arg("-i")
            .arg(path.as_os_str())
            .arg("-print_format")
            .arg("json")
            .arg("-show_entries")
            .arg("format=duration,format_long_name:stream_tags=creation_time:stream=codec_name,codec_type,width,height")
            .output()
            .map_err(|e| Error::Probe(e.to_string()))?;

        let v: Value = serde_json::from_slice(output.stdout.as_slice())
            .map_err(|e| Error::Json(e.to_string()))?;

        let mut metadata = Metadata::default();

        metadata.duration = v["format"]["duration"] // seconds with decimal
            .as_str()
            .and_then(|x| {
                let fractional_secs = x.parse::<f64>();
                let millis = fractional_secs.map(|s| s * 1000.0).ok();
                millis.and_then(|m| TimeDelta::try_milliseconds(m as i64))
            });

        metadata.container_format = v["format"]["format_long_name"]
            .as_str()
            .map(|x| x.to_string());

        if let Ok(video_stream) = v.clone().path("$.streams[?(@.codec_type == 'video')]") {
            println!("Video_stream = {}", video_stream);
            metadata.video_codec = video_stream[0]["codec_name"]
                .as_str()
                .map(|x| x.to_string());
            metadata.width = video_stream[0]["width"].as_u64();
            metadata.height = video_stream[0]["height"].as_u64();

            let created_at = video_stream[0]["tags"]["creation_time"].as_str();
            metadata.created_at = created_at.and_then(|x| {
                let dt = DateTime::parse_from_rfc3339(x).ok();
                dt.map(|y| y.to_utc())
            });
        }

        if let Ok(audio_stream) = v.path("$.streams[?(@.codec_type == 'audio')]") {
            metadata.audio_codec = audio_stream[0]["codec_name"]
                .as_str()
                .map(|x| x.to_string());
        }

        Ok(metadata)
    }
}
