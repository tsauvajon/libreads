//! Module libgen can find book metadata from their ISBN, and return a list
//! of search matches sorted by relevance for this application. It leverages
//! the LibGen API for that.
//!
//! Example request:
//! http://libgen.rs/json.php?isbn=9788853001351&fields=Title,Author,Year,Extension,MD5
//!
//! Example response:
//! [{"title":"Pride and Prejudice","author":"Jane Austen","year":"2000","extension":"pdf","md5":"ab13556b96d473c8dfad7165c4704526"}]

use crate::{extension::Extension, goodreads::BookIdentification};
use async_trait::async_trait;
use serde::Deserialize;

const BASE_URL: &str = "http://libgen.rs/json.php";

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait MetadataStore {
    async fn get_metadata(
        &self,
        book_identification: &BookIdentification,
    ) -> Result<Vec<LibgenMetadata>, Error>;
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct LibgenMetadata {
    pub title: String,
    pub author: String,
    pub year: String,
    #[serde(flatten)]
    pub extension: Extension,
    pub md5: String,
}

pub struct Libgen {
    base_url: String,
}

#[async_trait]
impl MetadataStore for Libgen {
    async fn get_metadata(
        &self,
        book_identification: &BookIdentification,
    ) -> Result<Vec<LibgenMetadata>, Error> {
        let query = if let Some(isbn10) = &book_identification.isbn10 {
            format!("isbn={isbn}", isbn = &isbn10)
        } else if let Some(isbn13) = &book_identification.isbn13 {
            format!("isbn={isbn}", isbn = &isbn13)
        } else if let (Some(title), Some(author)) =
            (&book_identification.title, &book_identification.author)
        {
            return Err(Error::NoIsbn {
                title: title.to_owned(),
                author: author.to_owned(),
            });
        } else {
            return Err(Error::MissingIndentificationInfo);
        };

        let url = format!(
            "{base_url}?{query}&fields=Title,Author,Year,Extension,MD5",
            base_url = self.base_url,
            query = query,
        );

        let resp = reqwest::get(url).await?.json().await?;
        Ok(resp)
    }
}

#[tokio::test]
#[ignore = "This test calls the LibGen API, don't run it with every file change"]
async fn third_party_test_get_metadata_from_libgen_api() {
    let book_identification = BookIdentification {
        isbn10: None,
        isbn13: Some("9788853001351".to_string()),
        title: None,
        author: None,
    };

    let got = Libgen::default()
        .get_metadata(&book_identification)
        .await
        .expect("The call to LibGen should succeed");
    assert_eq!(1, got.len());
    let got = &got[0];

    assert_eq!("Pride and Prejudice", got.title.as_str());
    assert_eq!("Jane Austen", got.author.as_str());
    assert_eq!(Extension::Pdf, got.extension);

    println!("{:?}", got);
}

#[tokio::test]
async fn test_get_metadata_no_isbn() {
    let book_identification = BookIdentification {
        isbn10: None,
        isbn13: None,
        title: Some("Hello".to_string()),
        author: Some("World".to_string()),
    };
    let got = Libgen::default().get_metadata(&book_identification).await;

    assert_eq!(
        Err(Error::NoIsbn {
            title: "Hello".to_string(),
            author: "World".to_string()
        }),
        got
    );
}

#[tokio::test]
async fn test_get_metadata_http_error() {
    let book_identification = BookIdentification {
        isbn10: None,
        isbn13: Some("123".to_string()),
        title: None,
        author: None,
    };
    let libgen = Libgen {
        base_url: "bad url".to_string(),
    };
    let got = libgen.get_metadata(&book_identification).await;

    assert_eq!(
        Err(Error::HttpError(
            "builder error: relative URL without a base".to_string()
        )),
        got
    );
}

pub fn find_most_relevant(books_metadata: &[LibgenMetadata]) -> Option<LibgenMetadata> {
    if books_metadata.is_empty() {
        return None;
    }

    let mut books_metadata = books_metadata.to_owned();
    books_metadata.sort_by(|a, b| a.extension.cmp(&b.extension));

    Some(books_metadata[0].clone())
}

#[test]
fn test_find_most_relevant() {
    let books_metadata = vec![
        LibgenMetadata {
            title: "Pride and Prejudice".to_string(),
            author: "Jane Austen".to_string(),
            year: "2000".to_string(),
            extension: Extension::Pdf,
            md5: "ABCD".to_string(),
        },
        LibgenMetadata {
            title: "Pride and Prejudice".to_string(),
            author: "Jane Austen".to_string(),
            year: "2000".to_string(),
            extension: Extension::Azw3,
            md5: "EF12".to_string(),
        },
        // This is the most relevant, because it has the Mobi extension.
        LibgenMetadata {
            title: "Pride and Prejudice".to_string(),
            author: "Jane Austen".to_string(),
            year: "2000".to_string(),
            extension: Extension::Mobi,
            md5: "3456".to_string(),
        },
        LibgenMetadata {
            title: "Pride and Prejudice".to_string(),
            author: "Jane Austen".to_string(),
            year: "2000".to_string(),
            extension: Extension::Epub,
            md5: "7890".to_string(),
        },
    ];

    assert_eq!(
        Some(books_metadata[2].clone()),
        find_most_relevant(&books_metadata)
    )
}

#[test]
fn test_find_most_relevant_no_books() {
    assert_eq!(None, find_most_relevant(&vec![]));
}

impl Default for Libgen {
    fn default() -> Self {
        Self {
            base_url: BASE_URL.to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    MissingIndentificationInfo,
    NoIsbn { title: String, author: String },
    HttpError(String),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::HttpError(err.to_string())
    }
}
