// SPDX-FileCopyrightText: © 2024 David Bliss
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::repo;
use crate::scanner;
use crate::Result;

/// Aggregate API for the scanner and the repository.
#[derive(Debug)]
pub struct Controller {
    repo: repo::Repository,
    scan: scanner::Scanner,
}

impl Controller {
    pub fn new(repo: repo::Repository, scan: scanner::Scanner) -> Controller {
        Controller { repo, scan }
    }

    /// Scans all photos and adds them to the repository.
    pub fn scan(&mut self) -> Result<()> {
        fn as_repo_pic(pic: scanner::Picture) -> repo::Picture {
            let exif_date_time = pic.exif.and_then(|x| x.created_at);
            let fs_date_time = pic.fs.and_then(|x| x.created_at);
            let order_by_ts = exif_date_time.map(|d| d.to_utc()).or(fs_date_time);

            repo::Picture {
                path: pic.path,
                order_by_ts,
            }
        }

        match self.scan.scan_all() {
            Ok(pics) => {
                // TODO can an interator be passed to add_all instead of a vector?
                let pics = pics.into_iter().map(|p| as_repo_pic(p)).collect();
                self.repo.add_all(&pics)?;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Gets all photos.
    pub fn all(&self) -> Result<Vec<repo::Picture>> {
        self.repo.all()
    }
}
