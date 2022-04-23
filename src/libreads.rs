//! Module libreads is the "domain" of this application.
//! It contains the rules on how to plug the different moving parts together
//! (for example, Goodreads -> LibGen -> Library.lol -> Calibre).
//!
//! In other words, it acts as glue between the other modules in this repo.

use crate::{
    goodreads::{BookIdentificationGetter, Goodreads},
    libgen::{self, Libgen, LibgenMetadata, MetadataStore},
    library_dot_lol::{DownloadLinks, DownloadLinksStore, LibraryDotLol},
};

pub struct LibReads {
    pub(crate) isbn_getter: Box<dyn BookIdentificationGetter>,
    pub(crate) metadata_store: Box<dyn MetadataStore>,
    pub(crate) download_links_store: Box<dyn DownloadLinksStore>,
}

#[derive(Debug, PartialEq)]
pub struct BookInfo {
    pub metadata: LibgenMetadata,
    pub download_links: DownloadLinks,
}

impl LibReads {
    pub async fn get_book_info_from_goodreads_url(
        &self,
        goodreads_book_url: &str,
    ) -> Result<BookInfo, Error> {
        let book_identification = self
            .isbn_getter
            .get_identification(goodreads_book_url)
            .await?;

        let books_metadata = self
            .metadata_store
            .get_metadata(&book_identification)
            .await?;
        let book_metadata = match libgen::find_most_relevant(&books_metadata) {
            None => return Err("Nothing found on LibGen for this book")?,
            Some(book_metadata) => book_metadata,
        };

        println!(
            "Formats found: {:?} -> {:?} selected",
            books_metadata
                .iter()
                .map(|book| &book.extension)
                .collect::<Vec<_>>(),
            &book_metadata.extension
        );

        let download_links = self
            .download_links_store
            .get_download_links(book_metadata.md5.as_str())
            .await?;

        Ok(BookInfo {
            metadata: book_metadata,
            download_links,
        })
    }
}

impl Default for LibReads {
    fn default() -> Self {
        Self {
            isbn_getter: Box::new(Goodreads::default()),
            metadata_store: Box::new(Libgen::default()),
            download_links_store: Box::new(LibraryDotLol::default()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    HttpError(String),
    ApplicationError(String),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::HttpError(err.to_string())
    }
}

impl From<libgen::Error> for Error {
    fn from(err: libgen::Error) -> Self {
        match err {
            libgen::Error::MissingIndentificationInfo => Self::ApplicationError(
                "Not enough info about the book found in this page".to_string(),
            ),
            libgen::Error::NoIsbn { title, author } => Self::ApplicationError(format!(
                "No ISBN found for \"{title}\" by {author}",
                title = title,
                author = author
            )),
            libgen::Error::HttpError(err) => Self::HttpError(err),
        }
    }
}

#[test]
fn test_libgen_error_to_error() {
    for (err, want) in vec![
        (
            libgen::Error::MissingIndentificationInfo,
            Error::ApplicationError(
                "Not enough info about the book found in this page".to_string(),
            ),
        ),
        (
            libgen::Error::NoIsbn {
                title: "1984".to_string(),
                author: "George Orwell".to_string(),
            },
            Error::ApplicationError(r#"No ISBN found for "1984" by George Orwell"#.to_string()),
        ),
        (
            libgen::Error::HttpError("Oh no!!".to_string()),
            Error::HttpError("Oh no!!".to_string()),
        ),
    ] {
        assert_eq!(want, Error::from(err));
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error::ApplicationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        extension::Extension,
        goodreads::{BookIdentification, MockBookIdentificationGetter},
        libgen::{LibgenMetadata, MockMetadataStore},
        library_dot_lol::MockDownloadLinksStore,
    };
    use mockall::predicate::eq;
    use std::vec;

    #[tokio::test]
    #[ignore = "This test calls live web pages and APIs, no need to run it with every file save."]
    async fn third_party_test_get_download_links_from_goodreads_url() {
        let test_url = "https://www.goodreads.com/book/show/1048424.Governing_the_Commons";
        let libreads = LibReads::default();
        let got = libreads
            .get_book_info_from_goodreads_url(test_url)
            .await
            .expect("Should get download links");

        assert_eq!(
            "https://ipfs.io/ipfs/bafykbzacedqn6erurfdw45jy4xbwldyh3ihqykr2kp3sx7knm6lslzcj66m76?filename=%28Political%20Economy%20of%20Institutions%20and%20Decisions%29%20Elinor%20Ostrom%20-%20Governing%20the%20Commons_%20The%20Evolution%20of%20Institutions%20for%20Collective%20Action%20%28Political%20Economy%20of%20Institutions%20and%20Decisions%29-Cambridge.djvu",
            got.download_links.ipfs_dot_io
        );
        assert_eq!(
            "http://31.42.184.140/main/501000/b41ce081c95a5c4864bec8488a7a6387/%28Political%20Economy%20of%20Institutions%20and%20Decisions%29%20Elinor%20Ostrom%20-%20Governing%20the%20Commons_%20The%20Evolution%20of%20Institutions%20for%20Collective%20Action%20%28Political%20Economy%20of%20Institutions%20and%20Decisions%29-Cambridge.djvu",
            got.download_links.http
        );
    }

    #[tokio::test]
    // TODO: make this test simpler (split into several smaller tests)
    async fn test_get_download_links_no_isbn_found() {
        let mut isbn_getter_mock = MockBookIdentificationGetter::new();
        isbn_getter_mock
            .expect_get_identification()
            .with(eq("http://hello.world"))
            .once()
            .returning(move |_| Box::pin(async { Ok(BookIdentification::default()) }));

        let libreads = LibReads {
            isbn_getter: Box::new(isbn_getter_mock),
            metadata_store: Box::new(Libgen::default()),
            download_links_store: Box::new(MockDownloadLinksStore::new()),
        };
        let got = libreads
            .get_book_info_from_goodreads_url("http://hello.world")
            .await;

        assert!(got.is_err());
        println!("{:?}", got);

        assert_eq!(
            Err(Error::ApplicationError(
                "Not enough info about the book found in this page".to_string()
            )),
            got
        );
    }

    #[tokio::test]
    async fn test_get_download_links_propagates_reqwest_errors() {
        let mut isbn_getter_mock = MockBookIdentificationGetter::new();
        isbn_getter_mock
            .expect_get_identification()
            .with(eq("http://hello.world"))
            .once()
            .returning(move |_| {
                // Using a badly formatted URL is the best way I found of returning a reqwest::Error.
                // TODO: change `get_isbn` to wrap the error in a custom type instead.
                Box::pin(async { Err(reqwest::get("Bad_Url").await.unwrap_err()) })
            });

        let libreads = LibReads {
            isbn_getter: Box::new(isbn_getter_mock),
            metadata_store: Box::new(MockMetadataStore::new()),
            download_links_store: Box::new(MockDownloadLinksStore::new()),
        };
        let got = libreads
            .get_book_info_from_goodreads_url("http://hello.world")
            .await;

        assert_eq!(
            Err(Error::HttpError(
                "builder error: relative URL without a base".to_string()
            )),
            got
        );
    }

    #[tokio::test]
    async fn test_get_download_links_nothing_found_on_libgen() {
        let mut isbn_getter_mock = MockBookIdentificationGetter::new();
        isbn_getter_mock
            .expect_get_identification()
            .with(eq("http://hello.world"))
            .once()
            .returning(move |_| {
                Box::pin(async {
                    Ok(BookIdentification {
                        isbn10: None,
                        isbn13: Some("fake_isbn_13".to_string()),
                        title: None,
                        author: None,
                    })
                })
            });

        let mut metadata_store_mock = MockMetadataStore::new();
        metadata_store_mock
            .expect_get_metadata()
            .with(eq(BookIdentification {
                isbn10: None,
                isbn13: Some("fake_isbn_13".to_string()),
                title: None,
                author: None,
            }))
            .once()
            .returning(move |_| Box::pin(async { Ok(vec![]) }));

        let libreads = LibReads {
            isbn_getter: Box::new(isbn_getter_mock),
            metadata_store: Box::new(metadata_store_mock),
            download_links_store: Box::new(MockDownloadLinksStore::new()),
        };
        let got = libreads
            .get_book_info_from_goodreads_url("http://hello.world")
            .await;

        assert_eq!(
            Err(Error::ApplicationError(
                "Nothing found on LibGen for this book".to_string()
            )),
            got
        );
    }

    #[tokio::test]
    async fn test_get_download_links_found_some_links() {
        let mut isbn_getter_mock = MockBookIdentificationGetter::new();
        isbn_getter_mock
            .expect_get_identification()
            .with(eq("http://hello.world"))
            .once()
            .returning(move |_| {
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
            .returning(move |_| {
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
                        cloudflare: "fake_cloudflare_link".to_string(),
                        ipfs_dot_io: "fake_ipfs_dot_io_link".to_string(),
                        infura: "fake_infura_link".to_string(),
                        pinata: "fake_pinata_link".to_string(),
                        http: "fake_http_link".to_string(),
                    })
                })
            });

        let libreads = LibReads {
            isbn_getter: Box::new(isbn_getter_mock),
            metadata_store: Box::new(metadata_store_mock),
            download_links_store: Box::new(download_links_store_mock),
        };
        let got = libreads
            .get_book_info_from_goodreads_url("http://hello.world")
            .await;

        assert_eq!(
            Ok(BookInfo {
                metadata: LibgenMetadata {
                    title: "hello".to_string(),
                    author: "hello".to_string(),
                    year: "hello".to_string(),
                    extension: Extension::Mobi,
                    md5: "MYBOOKMD5".to_string(),
                },
                download_links: DownloadLinks {
                    cloudflare: "fake_cloudflare_link".to_string(),
                    ipfs_dot_io: "fake_ipfs_dot_io_link".to_string(),
                    infura: "fake_infura_link".to_string(),
                    pinata: "fake_pinata_link".to_string(),
                    http: "fake_http_link".to_string(),
                }
            }),
            got
        );
    }

    #[tokio::test]
    async fn test_get_download_links_failed_to_get_library_dot_lol_links() {
        let mut isbn_getter_mock = MockBookIdentificationGetter::new();
        isbn_getter_mock
            .expect_get_identification()
            .with(eq("http://hello.world"))
            .once()
            .returning(move |_| {
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
            .returning(move |_| {
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

        let libreads = LibReads {
            isbn_getter: Box::new(isbn_getter_mock),
            metadata_store: Box::new(metadata_store_mock),
            download_links_store: Box::new(LibraryDotLol {
                base_url: "bad url".to_string(),
            }),
        };
        let got = libreads
            .get_book_info_from_goodreads_url("http://hello.world")
            .await;

        assert_eq!(
            Err(Error::HttpError(
                "builder error: relative URL without a base".to_string()
            )),
            got
        )
    }
}
