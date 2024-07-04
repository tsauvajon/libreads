//! Module http contains the web server exposing LibReads over an HTTP API.

use crate::{
    convert::{self, download_as},
    extension::Extension,
    libreads::{self, LibReads},
};

use actix_web::{
    error,
    http::header::{ContentDisposition, DispositionParam, DispositionType, CONTENT_TYPE},
    web, HttpResponse, Result,
};

pub async fn download(
    libreads: web::Data<LibReads>,
    goodreads_url: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let book_info = libreads
        .get_book_info_from_goodreads_url(&goodreads_url)
        .await?;

    let filename = download_as(book_info.into(), Extension::Mobi).await?;
    let buffer = load_file_to_memory(&filename).await?;

    let content_type = (CONTENT_TYPE, Extension::Mobi.content_type());
    let content_disposition = ContentDisposition {
        disposition: DispositionType::Attachment,
        parameters: vec![DispositionParam::Filename(filename)],
    };

    println!("Serving the converted file from memory!");

    Ok(HttpResponse::Ok()
        .append_header(content_disposition)
        .append_header(content_type)
        .body(buffer))
}

// Loads a file to memory and then delete it.
#[cfg_attr(tarpaulin, ignore)] // It would complexify the code too much to be able to test each error path individually
async fn load_file_to_memory(filename: &str) -> Result<Vec<u8>, std::io::Error> {
    // (1) Load file to memory
    let mut file = tokio::fs::File::open(&filename).await?;
    let metadata = tokio::fs::metadata(&filename).await?; // Untested.
    let mut buffer = vec![0; metadata.len() as usize];
    tokio::io::AsyncReadExt::read(&mut file, &mut buffer).await?; // Untested.

    // (2) Remove the file now that we have it in memory
    tokio::fs::remove_file(&filename).await?; // Untested.

    Ok(buffer)
}

#[tokio::test]
async fn test_load_file_to_memory_inexisting_file() {
    let got = load_file_to_memory("this file doesn't exist").await;
    assert!(got.is_err());
    let got = got.unwrap_err();

    assert_eq!(std::io::ErrorKind::NotFound, got.kind())
}

#[derive(Debug)]
pub struct Error {
    name: String,
    message: String,
}

impl error::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self.name.as_str() {
            "upstream" => actix_web::http::StatusCode::BAD_GATEWAY,
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[test]
fn test_error_status_code() {
    use actix_web::http::StatusCode;

    for (name, want) in vec![
        ("upstream", StatusCode::BAD_GATEWAY),
        ("http", StatusCode::INTERNAL_SERVER_ERROR),
        ("i/o", StatusCode::INTERNAL_SERVER_ERROR),
        ("application", StatusCode::INTERNAL_SERVER_ERROR),
        ("anything at all", StatusCode::INTERNAL_SERVER_ERROR),
    ] {
        let error = Error {
            name: name.to_string(),
            message: "doesn't matter".to_string(),
        };

        assert_eq!(want, actix_web::ResponseError::status_code(&error));
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
    use std::path::Path;

    use super::*;
    use crate::{
        goodreads::{BookIdentification, MockBookIdentificationGetter},
        libgen::{LibgenMetadata, MockMetadataStore},
        library_dot_lol::{DownloadLinks, MockDownloadLinksStore},
    };
    use actix_web::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
    use httpmock::{Method::GET, MockServer};
    use mockall::predicate::eq;

    #[actix_web::test]
    async fn test_download() {
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

        let cd = resp.headers().get(CONTENT_DISPOSITION).unwrap();
        assert_eq!(r#"attachment; filename="hello.mobi""#, cd);

        let ct = resp.headers().get(CONTENT_TYPE).unwrap();
        assert_eq!("application/x-mobipocket-ebook", ct);

        // Local file has been deleted
        assert!(!Path::new("hello.mobi").exists());
        endpoint_mock.assert();
    }

    #[actix_web::test]
    async fn test_download_error() {
        let mock_goodreads_url = web::Path::from("http://hello.world".to_string());

        let mut isbn_getter_mock = MockBookIdentificationGetter::new();
        isbn_getter_mock
            .expect_get_identification()
            .with(eq("http://hello.world"))
            .once()
            .returning(|_| Box::pin(async { Err(reqwest::get("Bad_Url").await.unwrap_err()) }));

        let mock_libreads = LibReads {
            isbn_getter: Box::new(isbn_getter_mock),
            metadata_store: Box::new(MockMetadataStore::new()),
            download_links_store: Box::new(MockDownloadLinksStore::new()),
        };

        let resp = download(web::Data::new(mock_libreads), mock_goodreads_url).await;
        assert!(resp.is_err())
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
