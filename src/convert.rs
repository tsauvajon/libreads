use crate::{extension::Extension, libreads::BookInfo};
use tokio::{fs::File, io, process::Command};

const EBOOK_CONVERT_EXECUTABLE: &str = "ebook-convert";

#[derive(Debug, PartialEq)]
pub struct InputBookInfo {
    title: String,
    extension: Extension,
    download_link: String,
}

impl From<BookInfo> for InputBookInfo {
    fn from(book: BookInfo) -> Self {
        Self {
            title: book.metadata.title,
            extension: book.metadata.extension,
            download_link: book.download_links.cloudflare,
        }
    }
}

#[test]
fn test_input_from_book_info() {
    let book_info = BookInfo {
        metadata: crate::libgen::LibgenMetadata {
            title: "Alice in Wonderland".to_string(),
            author: "this field should be ignored".to_string(),
            year: "this field should be ignored".to_string(),
            extension: Extension::Mobi,
            md5: "this field should be ignored".to_string(),
        },
        download_links: crate::library_dot_lol::DownloadLinks {
            cloudflare: "https://hello.com".to_string(),
            ipfs_dot_io: "this field should be ignored".to_string(),
            infura: "this field should be ignored".to_string(),
            pinata: "this field should be ignored".to_string(),
            http: "this field should be ignored".to_string(),
        },
    };
    let got = InputBookInfo::from(book_info);

    let want = InputBookInfo {
        title: "Alice in Wonderland".to_string(),
        extension: Extension::Mobi,
        download_link: "https://hello.com".to_string(),
    };
    assert_eq!(want, got);
}

// This takes some book metadata, download the book, convert it if needed and
// return the converted book filename.
pub async fn download_as(
    book: InputBookInfo,
    wanted_extension: Extension,
) -> Result<String, Error> {
    let title = sanitise_title(book.title.as_str());

    let in_filename = format!("{}.{}", title, book.extension);
    download(book.download_link.as_str(), &in_filename).await?;

    if book.extension == wanted_extension {
        return Ok(in_filename);
    }

    let out_filename = format!("{}.{}", title, wanted_extension);

    println!("Converting book to {:?}...", wanted_extension);
    let output = Command::new(EBOOK_CONVERT_EXECUTABLE)
        .arg(&in_filename)
        .arg(&out_filename)
        .output()
        .await?;

    tokio::fs::remove_file(&in_filename)
        .await
        .expect("Delete input file");

    let output = String::from_utf8_lossy(&output.stdout);
    if !output.contains("Output saved to") {
        // Something probably went wrong.
        // We return the full command output as an error.
        return Err(Error::Conversion(
            String::from_utf8_lossy(output.as_bytes()).to_string(),
        ));
    }

    Ok(out_filename)
}

#[cfg(test)]
mod conversion_tests {
    use super::*;
    use httpmock::{Method::GET, MockServer};

    #[tokio::test]
    async fn convert() {
        let mock_server = MockServer::start();
        let endpoint_mock = mock_server.mock(|when, then| {
            when.method(GET).path("/book.epub");
            then.status(200)
                .body(include_bytes!("../tests/testdata/dummy_ebook.epub"));
        });

        let book = InputBookInfo {
            title: "Governing the Commons".to_string(),
            extension: Extension::Epub,
            download_link: mock_server.url("/book.epub"),
        };

        let output_filename = download_as(book, Extension::Mobi).await.unwrap();
        std::fs::remove_file(output_filename).expect("Delete output file");
        endpoint_mock.assert();
    }

    #[tokio::test]
    async fn conversion_fails() {
        let mock_server = MockServer::start();
        let endpoint_mock = mock_server.mock(|when, then| {
            when.method(GET).path("/book.pdf");
            then.status(200)
                .body(include_bytes!("../tests/testdata/dummy_invalid_ebook.pdf"));
        });

        let book = InputBookInfo {
            title: "Dummy invalid ebook 1".to_string(),
            extension: Extension::Pdf,
            download_link: mock_server.url("/book.pdf"),
        };

        let got = download_as(book, Extension::Mobi).await;
        assert!(got.is_err());
        endpoint_mock.assert();
    }

    #[tokio::test]
    async fn returns_early_if_no_conversion_is_needed() {
        let mock_server = MockServer::start();
        let endpoint_mock = mock_server.mock(|when, then| {
            when.method(GET).path("/book.pdf");
            then.status(200)
                .body(include_bytes!("../tests/testdata/dummy_invalid_ebook.pdf"));
        });

        let book = InputBookInfo {
            title: "Dummy invalid ebook 2".to_string(),
            extension: Extension::Pdf,
            download_link: mock_server.url("/book.pdf"),
        };

        // Note: when the input format and output format are the same (here PDF),
        // if should not try to perform any conversion.
        // Therefore, it should not matter whether the ebook is valid or invalid.
        let output_filename = download_as(book, Extension::Pdf)
            .await
            .expect("Should exit early and not perform validations");
        std::fs::remove_file(output_filename).expect("Delete output file");
        endpoint_mock.assert();
    }
}

#[tokio::test]
async fn propagates_reqwest_errors() {
    let book = InputBookInfo {
        title: "Dummy invalid ebook".to_string(),
        extension: Extension::Djvu,
        download_link: "malformed_url".to_string(),
    };

    let got = download_as(book, Extension::Djvu).await;
    assert_eq!(
        Err(Error::Http(
            "builder error: relative URL without a base".to_string(),
        )),
        got
    );
}

async fn download(url: &str, filename: &str) -> Result<(), Error> {
    println!("Downloading {}...", &filename);

    let resp = reqwest::get(url).await?;
    let mut out = File::create(filename).await?;
    io::copy(&mut resp.bytes().await?.as_ref(), &mut out).await?;

    Ok(())
}

#[tokio::test]
async fn test_download_incorrect_filename() {
    use httpmock::{Method::GET, MockServer};

    let mock_server = MockServer::start();
    let endpoint_mock = mock_server.mock(|when, then| {
        when.method(GET).path("/");
        then.status(200);
    });

    let got = download(mock_server.url("/").as_str(), "   /\\ Invalid file name").await;
    assert_eq!(
        Err(Error::Io(
            "No such file or directory (os error 2)".to_string()
        )),
        got,
    );

    endpoint_mock.assert();
}

fn sanitise_title(title: &str) -> String {
    title
        .replace(|c: char| c.is_ascii_punctuation(), " ")
        .replace(|c: char| !c.is_whitespace() && !c.is_alphanumeric(), "")
        .trim()
        .to_string()
}

#[test]
fn test_sanitise_title() {
    for (title, want) in vec![
        ("hello", "hello"),
        ("hello world", "hello world"),
        ("Hello World", "Hello World"),
        ("Hello World¶¶", "Hello World"),
        ("Hello_World¶¶", "Hello World"),
        ("Hello-World¶¶", "Hello World"),
        ("Hello-World¶¶", "Hello World"),
        ("Hello.World¶¶", "Hello World"),
        ("       Hello.World     ", "Hello World"),
        ("Héllô Wørld¶¶", "Héllô Wørld"),
    ] {
        assert_eq!(want, sanitise_title(title));
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    Io(String),
    Http(String),
    Conversion(String),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Http(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err.to_string())
    }
}
