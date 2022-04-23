//! Module http contains the web server exposing LibReads over an HTTP API.

use crate::{
    convert::{self, download_as},
    extension::Extension,
    libreads::{self, BookDownloader},
};
use actix_files::NamedFile;
use actix_web::{error, web, Result};

pub async fn download(
    libreads: web::Data<Box<dyn BookDownloader>>,
    goodreads_url: web::Path<String>,
) -> Result<NamedFile, Error> {
    let book_info = libreads
        .get_book_info_from_goodreads_url(&goodreads_url)
        .await?;

    let filename = download_as(book_info.into(), Extension::Mobi).await?;

    Ok(NamedFile::open(filename)?)
}

#[derive(Debug)]
pub struct Error {
    name: String,
    message: String,
}

impl error::ResponseError for Error {
    fn status_code(&self) -> reqwest::StatusCode {
        match self.name.as_str() {
            "upstream" => reqwest::StatusCode::BAD_GATEWAY,
            _ => reqwest::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.message)
    }
}

impl From<libreads::Error> for Error {
    fn from(err: libreads::Error) -> Self {
        match err {
            libreads::Error::HttpError(message) => Error {
                name: "upstream".to_string(),
                message,
            },
            libreads::Error::ApplicationError(message) => Error {
                name: "application".to_string(),
                message,
            },
        }
    }
}

impl From<convert::Error> for Error {
    fn from(err: convert::Error) -> Self {
        match err {
            convert::Error::IoError(message) => Error {
                name: "i/o".to_string(),
                message, // TODO: hide me
            },
            convert::Error::HttpError(message) => Error {
                name: "upstream".to_string(),
                message,
            },
            convert::Error::ConversionError(message) => Error {
                name: "conversion".to_string(),
                message,
            },
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error {
            name: "i/o".to_string(),
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::libreads::MockBookDownloader;
    use mockall::predicate::eq;

    #[actix_web::test]
    async fn test_todo() {
        let mock_goodreads_url = web::Path::from("http://hello.world".to_string());

        let mock_libreads = MockBookDownloader::new();
        let mock_libreads = web::Data::new(Box::new(mock_libreads));

        let resp = download(mock_libreads, mock_goodreads_url)
            .await
            .expect("the call should succeed");

        println!("{:?}", resp.path())
    }
}
