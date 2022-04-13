//! Module libreads is the "domain" of this application.
//! It contains the rules on how to plug the different moving parts together
//! (for example, Goodreads -> LibGen -> Library.lol -> Calibre).
//!
//! In other words, it acts as glue between the other modules in this repo.

use crate::{
    download_registry::{self, DownloadLinks},
    goodreads, libgen,
};

pub async fn get_download_links_from_goodreads_url(
    goodreads_book_url: &str,
) -> Result<DownloadLinks, Error> {
    let (isbn10, isbn13) = goodreads::get_isbn(goodreads_book_url).await?;

    let isbn = if let Some(isbn) = isbn13 {
        println!("Using ISBN13: {}", isbn);
        isbn
    } else if let Some(isbn) = isbn10 {
        println!("Using ISBN10: {}", isbn);
        isbn
    } else {
        return Err("No ISBN found on this page")?;
    };

    let books_metadata = libgen::get_metadata(isbn.as_str()).await?;
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

    Ok(download_registry::get_download_links(book_metadata.md5.as_str()).await?)
}

#[tokio::test]
#[ignore = "This test calls live web pages and APIs, no need to run it with every file save."]
async fn test_get_download_links_from_goodreads_url() {
    let test_url = "https://www.goodreads.com/book/show/1048424.Governing_the_Commons";
    let got = get_download_links_from_goodreads_url(test_url)
        .await
        .expect("Should get download links");

    assert_eq!(
        "https://ipfs.io/ipfs/bafykbzacedqn6erurfdw45jy4xbwldyh3ihqykr2kp3sx7knm6lslzcj66m76?filename=%28Political%20Economy%20of%20Institutions%20and%20Decisions%29%20Elinor%20Ostrom%20-%20Governing%20the%20Commons_%20The%20Evolution%20of%20Institutions%20for%20Collective%20Action%20%28Political%20Economy%20of%20Institutions%20and%20Decisions%29-Cambridge.djvu",
        got.ipfs_dot_io
    );
    assert_eq!(
        "http://31.42.184.140/main/501000/b41ce081c95a5c4864bec8488a7a6387/%28Political%20Economy%20of%20Institutions%20and%20Decisions%29%20Elinor%20Ostrom%20-%20Governing%20the%20Commons_%20The%20Evolution%20of%20Institutions%20for%20Collective%20Action%20%28Political%20Economy%20of%20Institutions%20and%20Decisions%29-Cambridge.djvu",
        got.http
    );
}

#[derive(Debug)]
pub enum Error {
    HttpError(String),
    ApplicationError(String),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::HttpError(err.to_string())
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error::ApplicationError(err.to_string())
    }
}
