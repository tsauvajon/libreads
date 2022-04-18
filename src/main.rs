mod convert;
mod goodreads;
mod libgen;
mod library_dot_lol;
mod libreads;

use convert::download_as;
use libgen::Extension;
use libreads::{Error, LibReads};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let book_info = LibReads::default()
        .get_book_info_from_goodreads_url(
            "https://www.goodreads.com/book/show/22463.The_Origin_of_Species",
        )
        .await?;
    println!(
        "IPFS.io download link: {}",
        book_info.download_links.ipfs_dot_io
    );

    let filename = download_as(book_info.into(), Extension::Mobi)
        .await
        .expect("Download and convert the ebook");
    println!("Ebook downloaded as {}", filename);

    Ok(())
}
