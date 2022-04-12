mod goodreads;
mod libgen;
mod library;

use tokio;

#[tokio::main]
async fn main() -> Result<(), Error> {
    process("https://www.goodreads.com/book/show/1048424.Governing_the_Commons").await
}

async fn process(goodreads_book_url: &str) -> Result<(), Error> {
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

    let download_links = library::get_download_links(book_metadata.md5.as_str()).await?;
    println!("IPFS.io download link: {}", download_links.ipfs_dot_io);

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
