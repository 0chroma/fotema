// SPDX-FileCopyrightText: © 2024 David Bliss
//
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Debug)]
pub enum Error {
    RepositoryError(String),
    FileSystemError(String),
    MetadataError(String),
    ScannerError(String),
}
