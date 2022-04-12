mod find_isbn;
mod libgen;

use scraper::Html;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    libgen::get_metadata("Pride and Prejudice")?;

    goodreads().await
}

async fn goodreads() -> Result<(), Box<dyn std::error::Error>> {
    let body = reqwest::get("https://www.goodreads.com/book/show/1048424.Governing_the_Commons")
        .await?
        .text()
        .await?;

    let document = Html::parse_document(&body);
    let isbn = find_isbn::find_isbn_10(&document);
    let isbn13 = find_isbn::find_isbn_13(&document);

    println!("{:?}", isbn);
    println!("{:?}", isbn13);

    Ok(())
}
