//! Module http contains the web server exposing LibReads over an HTTP API.

use crate::{
    convert::{self, download_as},
    extension::Extension,
    libreads::{self, LibReads},
};
use actix_files::NamedFile;
use actix_web::{error, web, Result};

pub async fn download(
    libreads: web::Data<LibReads>,
    goodreads_url: web::Path<String>,
) -> Result<NamedFile, Error> {
    let book_info = libreads
        .get_book_info_from_goodreads_url(&goodreads_url)
        .await?;

    let filename = download_as(book_info.into(), Extension::Mobi).await?;

    // TODO: find a good way to clean up the file after it has been served.
    // I'm thinking of 1/ opening the file in memory, 2/ deleting the file, 3/ serving the file from memory.

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

#[test]
fn test_error_from_libreads_error() {
    for (err, want) in vec![
        (
            libreads::Error::HttpError("something bad".to_string()),
            "upstream: something bad",
        ),
        (
            libreads::Error::ApplicationError("oh no".to_string()),
            "application: oh no",
        ),
    ] {
        let got_err = Error::from(err);
        assert_eq!(want, format!("{}", got_err))
    }
}

impl From<convert::Error> for Error {
    fn from(err: convert::Error) -> Self {
        match err {
            convert::Error::Io(message) => Error {
                name: "i/o".to_string(),
                message, // TODO: hide me
            },
            convert::Error::Http(message) => Error {
                name: "upstream".to_string(),
                message,
            },
            convert::Error::Conversion(message) => Error {
                name: "conversion".to_string(),
                message,
            },
        }
    }
}

#[test]
fn test_error_from_convert_error() {
    for (err, want) in vec![
        (convert::Error::Io("failure".to_string()), "i/o: failure"),
        (
            convert::Error::Http("failure!!1".to_string()),
            "upstream: failure!!1",
        ),
        (
            convert::Error::Conversion("unknown format provided".to_string()),
            "conversion: unknown format provided",
        ),
    ] {
        let got_err = Error::from(err);
        assert_eq!(want, format!("{}", got_err))
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

#[test]
fn test_error_from_stdio_error() {
    let got_err: Error = std::io::Error::new(std::io::ErrorKind::AddrInUse, "big failure").into();
    assert_eq!("i/o: big failure", format!("{}", got_err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        goodreads::{BookIdentification, MockBookIdentificationGetter},
        libgen::{LibgenMetadata, MockMetadataStore},
        library_dot_lol::{DownloadLinks, MockDownloadLinksStore},
    };
    use httpmock::{Method::GET, MockServer};
    use mockall::predicate::eq;

    #[actix_web::test]
    async fn test_todo() {
        let mock_download_server = MockServer::start();
        let endpoint_mock = mock_download_server.mock(|when, then| {
            when.method(GET).path("/book.mobi");
            then.status(200)
                .body(include_bytes!("../tests/testdata/dummy_ebook.mobi"));
        });
        let url = mock_download_server.url("/book.mobi").to_owned();
        let download_link: &'static str = Box::leak(url.into_boxed_str()); // Leaks memory!! TODO: find another way to do this.

        let mock_goodreads_url = web::Path::from("http://hello.world".to_string());
        let mock_libreads = web::Data::new(get_mock_libreads(download_link));

        let resp = download(mock_libreads, mock_goodreads_url)
            .await
            .expect("the call should succeed");

        tokio::fs::remove_file("hello.mobi")
            .await
            .expect("Delete output file");
        endpoint_mock.assert();

        println!("{:?}", resp.path())
    }

    // TODO: make the whole flow easier to mock, by wrapping it in a higher level thing.
    fn get_mock_libreads(book_download_url: &'static str) -> LibReads {
        let mut isbn_getter_mock = MockBookIdentificationGetter::new();
        isbn_getter_mock
            .expect_get_identification()
            .with(eq("http://hello.world"))
            .once()
            .returning(|_| {
                Box::pin(async {
                    Ok(BookIdentification {
                        isbn10: Some("fake_isbn_10".to_string()),
                        isbn13: None,
                        title: None,
                        author: None,
                    })
                })
            });

        let mut metadata_store_mock = MockMetadataStore::new();
        metadata_store_mock
            .expect_get_metadata()
            .with(eq(BookIdentification {
                isbn10: Some("fake_isbn_10".to_string()),
                isbn13: None,
                title: None,
                author: None,
            }))
            .once()
            .returning(|_| {
                Box::pin(async {
                    Ok(vec![LibgenMetadata {
                        title: "hello".to_string(),
                        author: "hello".to_string(),
                        year: "hello".to_string(),
                        extension: Extension::Mobi,
                        md5: "MYBOOKMD5".to_string(),
                    }])
                })
            });

        let mut download_links_store_mock = MockDownloadLinksStore::new();
        download_links_store_mock
            .expect_get_download_links()
            .with(eq("MYBOOKMD5"))
            .once()
            .returning(|_| {
                Box::pin(async {
                    Ok(DownloadLinks {
                        cloudflare: book_download_url.to_string(),
                        ipfs_dot_io: "fake_ipfs_dot_io_link".to_string(),
                        infura: "fake_infura_link".to_string(),
                        pinata: "fake_pinata_link".to_string(),
                        http: "fake_http_link".to_string(),
                    })
                })
            });

        LibReads {
            isbn_getter: Box::new(isbn_getter_mock),
            metadata_store: Box::new(metadata_store_mock),
            download_links_store: Box::new(download_links_store_mock),
        }
    }
}
