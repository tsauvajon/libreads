mod goodreads;
mod libgen;
mod library_dot_lol;
mod libreads;

use libreads::{Error, LibReads};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let download_links = LibReads::default()
        .get_download_links_from_book_url(
            "https://www.goodreads.com/book/show/1048424.Governing_the_Commons",
        )
        .await?;
    println!("IPFS.io download link: {}", download_links.ipfs_dot_io);

    Ok(())
}
