mod goodreads;
mod libgen;

use tokio;

#[tokio::main]
async fn main() -> Result<(), Error> {
    process("https://www.goodreads.com/book/show/1048424.Governing_the_Commons").await
}

async fn process(goodreads_book_url: &str) -> Result<(), Error> {
    let (isbn10, isbn13) = goodreads::get_isbn(goodreads_book_url).await?;

    println!("ISBN10: {:?}, ISBN13: {:?}", isbn10, isbn13);

    let isbn = if let Some(isbn) = isbn13 {
        isbn
    } else if let Some(isbn) = isbn10 {
        isbn
    } else {
        return Err("No ISBN found on this page")?;
    };

    let book_metadata = libgen::get_metadata(isbn.as_str()).await?;
    if book_metadata.is_none() {
        return Err("Nothing found on LibGen for this book")?;
    }

    Ok(())
}

#[derive(Debug)]
enum Error {
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
