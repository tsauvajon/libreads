//! Module libgen can find book metadata from their ISBN, and return a list
//! of search matches sorted by relevance for this application. It leverages
//! the LibGen API for that.
//!
//! Example request:
//! http://libgen.rs/json.php?isbn=9788853001351&fields=Title,Author,Year,Extension,MD5
//!
//! Example response:
//! [{"title":"Pride and Prejudice","author":"Jane Austen","year":"2000","extension":"pdf","md5":"ab13556b96d473c8dfad7165c4704526"}]

use async_trait::async_trait;
use serde::{de, Deserialize, Deserializer};
use serde_json::Value;

const BASE_URL: &str = "http://libgen.rs/json.php";

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait MetadataStore {
    async fn get_metadata(&self, isbn: &str) -> Result<Vec<LibgenMetadata>, reqwest::Error>;
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Extension {
    Mobi,
    Epub,
    Azw3,
    Djvu,
    Pdf,
    Doc,
    Other(String),
}

impl std::fmt::Display for Extension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Extension::Mobi => "mobi",
                Extension::Epub => "epub",
                Extension::Azw3 => "azw3",
                Extension::Djvu => "djvu",
                Extension::Pdf => "pdf",
                Extension::Doc => "doc",
                Extension::Other(ext) => ext.as_str(),
            }
        )
    }
}

#[derive(Default)]
pub struct Libgen {}

#[async_trait]
impl MetadataStore for Libgen {
    async fn get_metadata(&self, isbn: &str) -> Result<Vec<LibgenMetadata>, reqwest::Error> {
        let url = format!(
            "{base_url}?isbn={isbn}&fields=Title,Author,Year,Extension,MD5",
            base_url = BASE_URL,
            isbn = isbn,
        );

        reqwest::get(url).await?.json().await
    }
}

#[tokio::test]
#[ignore = "This test calls the LibGen API, don't run it with every file change"]
async fn integration_test_get_metadata_from_libgen_api() {
    let got = Libgen::default()
        .get_metadata("9788853001351")
        .await
        .expect("The call to LibGen should succeed");
    assert_eq!(1, got.len());
    let got = &got[0];

    assert_eq!("Pride and Prejudice", got.title.as_str());
    assert_eq!("Jane Austen", got.author.as_str());
    assert_eq!(Extension::Pdf, got.extension);

    println!("{:?}", got);
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

impl<'de> Deserialize<'de> for Extension {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Value::deserialize(deserializer)?;
        Ok(
            match Option::deserialize(&v["extension"]).map_err(de::Error::custom)? {
                Some(ext) => match ext {
                    "mobi" => Self::Mobi,
                    "epub" => Self::Epub,
                    "azw3" => Self::Azw3,
                    "djvu" => Self::Djvu,
                    "pdf" => Self::Pdf,
                    "doc" => Self::Doc,
                    ext => Self::Other(ext.to_string()),
                },
                None => Self::Other(String::new()),
            },
        )
    }
}

impl Ord for Extension {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        fn val(ext: &Extension) -> u8 {
            match ext {
                Extension::Mobi => 1,
                Extension::Epub => 2,
                Extension::Azw3 => 3,
                Extension::Djvu => 4,
                Extension::Pdf => 90,
                Extension::Doc => 91,
                Extension::Other(_) => 92,
            }
        }

        val(self).cmp(&val(other))
    }
}

impl PartialOrd for Extension {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[test]
fn test_sort_extensions() {
    let mut extensions = vec![
        Extension::Pdf,
        Extension::Other("whatever".to_string()),
        Extension::Mobi,
        Extension::Pdf,
        Extension::Djvu,
        Extension::Epub,
        Extension::Azw3,
        Extension::Doc,
        Extension::Pdf,
        Extension::Mobi,
        Extension::Epub,
        Extension::Doc,
        Extension::Mobi,
        Extension::Pdf,
    ];

    extensions.sort();

    assert_eq!(
        vec![
            Extension::Mobi,
            Extension::Mobi,
            Extension::Mobi,
            Extension::Epub,
            Extension::Epub,
            Extension::Azw3,
            Extension::Djvu,
            Extension::Pdf,
            Extension::Pdf,
            Extension::Pdf,
            Extension::Pdf,
            Extension::Doc,
            Extension::Doc,
            Extension::Other("whatever".to_string()),
        ],
        extensions
    );
}
