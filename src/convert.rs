use std::{fs::File, io, process::Command};

use crate::{libgen::Extension, libreads::BookInfo};

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
) -> Result<String, String> {
    let title = sanitise_title(book.title.as_str());

    let in_filename = format!("{}.{}", title, book.extension);
    if let Err(err) = download(book.download_link.as_str(), &in_filename).await {
        return Err(err.to_string());
    };

    if book.extension == wanted_extension {
        return Ok(in_filename);
    }

    let out_filename = format!("{}.{}", title, wanted_extension);
    println!("Converting book to {:?}...", wanted_extension);
    println!("[Debug] in: {}, out: {}", in_filename, out_filename);

    println!(
        "{}",
        String::from_utf8_lossy(&Command::new("ls").output().unwrap().stdout)
    );

    let output = Command::new(EBOOK_CONVERT_EXECUTABLE)
        .arg(&in_filename)
        .arg(&out_filename)
        .output()
        .unwrap();

    std::fs::remove_file(&in_filename).expect("Delete input file");

    let output = String::from_utf8_lossy(&output.stdout);
    if output.contains("Cannot read from") {
        return Err("File not found".to_string());
    }

    if !output.contains("Output saved to") {
        // Something probably went wrong.
        // We return the full command output as an error.
        return Err(String::from_utf8_lossy(output.as_bytes()).to_string());
    }

    Ok(out_filename)
}

#[tokio::test]
#[ignore = "This does a real HTTP call to a 3rd party server. TODO: mock that server."]
async fn convert() {
    let book = InputBookInfo {
        title: "Governing the Commons".to_string(),
        extension: Extension::Djvu,
        download_link: "https://cloudflare-ipfs.com/ipfs/bafykbzacedqn6erurfdw45jy4xbwldyh3ihqykr2kp3sx7knm6lslzcj66m76?filename=%28Political%20Economy%20of%20Institutions%20and%20Decisions%29%20Elinor%20Ostrom%20-%20Governing%20the%20Commons_%20The%20Evolution%20of%20Institutions%20for%20Collective%20Action%20%28Political%20Economy%20of%20Institutions%20and%20Decisions%29-Cambridge.djvu".to_string(),
    };

    let output_filename = download_as(book, Extension::Mobi).await.unwrap();
    std::fs::remove_file(output_filename).expect("Delete output file");
}

async fn download(url: &str, filename: &str) -> Result<(), reqwest::Error> {
    println!("Downloading {}...", &filename);

    let resp = reqwest::get(url).await?;
    let mut out = File::create(filename).expect("failed to create file");
    io::copy(&mut resp.bytes().await?.as_ref(), &mut out).expect("failed to copy content");

    Ok(())
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
