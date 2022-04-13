mod download_registry;
mod goodreads;
mod libgen;
mod libreads;

use libreads::{get_download_links_from_goodreads_url, Error};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let download_links = get_download_links_from_goodreads_url(
        "https://www.goodreads.com/book/show/1048424.Governing_the_Commons",
    )
    .await?;
    println!("IPFS.io download link: {}", download_links.ipfs_dot_io);

    Ok(())
}
