// SPDX-FileCopyrightText: © 2024 David Bliss
//
// SPDX-License-Identifier: GPL-3.0-or-later

use relm4::prelude::*;
use relm4::Worker;
use anyhow::*;
use fotema_core::video::metadata;

#[derive(Debug)]
pub enum EnrichVideosInput {
    Start,
}

#[derive(Debug)]
pub enum EnrichVideosOutput {
    // Thumbnail generation has started for a given number of videos.
    Started(usize),

    // Thumbnail has been generated for a photo.
    Generated,

    // Thumbnail generation has completed
    Completed,

}

pub struct EnrichVideos {
    repo: fotema_core::video::Repository,
}

impl EnrichVideos {

    fn enrich(
        mut repo: fotema_core::video::Repository,
        sender: &ComponentSender<EnrichVideos>) -> Result<()>
     {
        let start = std::time::Instant::now();

        let unprocessed_vids: Vec<fotema_core::video::model::Video> = repo
            .all()?
            .into_iter()
            .filter(|vid| {
                let has_thumbnail = vid.thumbnail_path.as_ref().is_some_and(|p| p.exists());
                let needs_transcode = vid.is_transcode_required()
                    && !vid.transcoded_path.as_ref().is_some_and(|p| p.exists());

                !has_thumbnail || needs_transcode
            })
            .collect();

        let vids_count = unprocessed_vids.len();
        if let Err(e) = sender.output(EnrichVideosOutput::Started(vids_count)){
            println!("Failed sending gen started: {:?}", e);
        }

        let metadatas = unprocessed_vids
            //.par_iter() // don't multiprocess until memory usage is better understood.
            .iter()
            .flat_map(|vid| {
                let result = metadata::from_path(&vid.path);
                result.map(|m| (vid.video_id, m))
            })
            .collect();

        repo.add_metadata(metadatas)?;

        println!("Enriched {} videos in {} seconds.", vids_count, start.elapsed().as_secs());

        if let Err(e) = sender.output(EnrichVideosOutput::Completed) {
            println!("Failed sending EnrichVideosOutput::Completed: {:?}", e);
        }

        Ok(())
    }
}

impl Worker for EnrichVideos {
    type Init = fotema_core::video::Repository;
    type Input = EnrichVideosInput;
    type Output = EnrichVideosOutput;

    fn init(repo: Self::Init, _sender: ComponentSender<Self>) -> Self  {
        let model = EnrichVideos {
            repo,
        };
        model
    }


    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            EnrichVideosInput::Start => {
                println!("Enriching videos...");
                let repo = self.repo.clone();

                // Avoid runtime panic from calling block_on
                rayon::spawn(move || {
                    if let Err(e) = EnrichVideos::enrich(repo, &sender) {
                        println!("Failed to enrich videos: {}", e);
                    }
                });
            }
        };
    }
}
