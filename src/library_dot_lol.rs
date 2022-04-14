//! Module download_registry can find download links for a book.
//!
//! In the current implementation, it takes a book MD5 hash from LibGen,
//! and finds the download links in http://library.lol

use async_trait::async_trait;
use scraper::{Html, Selector};

const BASE_URL: &str = "http://library.lol/main";

#[derive(PartialEq, Debug)]
pub struct DownloadLinks {
    pub cloudflare: String,
    pub ipfs_dot_io: String,
    pub infura: String,
    pub pinata: String,
    pub http: String,
}

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait DownloadLinksStore {
    async fn get_download_links(&self, id: &str) -> Result<DownloadLinks, reqwest::Error>;
}

#[derive(Default)]
pub struct LibraryDotLol {}

#[async_trait]
impl DownloadLinksStore for LibraryDotLol {
    async fn get_download_links(&self, id: &str) -> Result<DownloadLinks, reqwest::Error> {
        let page_url = format!("{base_url}/{id}", base_url = BASE_URL, id = id);
        let body = reqwest::get(page_url).await?.text().await?;
        let document = Html::parse_document(&body);

        Ok(extract_links(&document))
    }
}

#[tokio::test]
#[ignore = "It calls the webpage, don't run it by default"]
async fn integration_test_get_download_links() {
    let got = LibraryDotLol::default()
        .get_download_links("AB13556B96D473C8DFAD7165C4704526")
        .await;

    assert!(got.is_ok());
    assert_eq!(
        DownloadLinks {
            cloudflare: "https://cloudflare-ipfs.com/ipfs/bafykbzacedotjioda7arles2s7gyc74hppa3owagm4hmohz4ye574omtalsoc?filename=Jane%20Austen%20-%20Pride%20and%20Prejudice-CIDEB%20%282000%29.pdf".to_string(),
            ipfs_dot_io: "https://ipfs.io/ipfs/bafykbzacedotjioda7arles2s7gyc74hppa3owagm4hmohz4ye574omtalsoc?filename=Jane%20Austen%20-%20Pride%20and%20Prejudice-CIDEB%20%282000%29.pdf".to_string(),
            infura: "https://ipfs.infura.io/ipfs/bafykbzacedotjioda7arles2s7gyc74hppa3owagm4hmohz4ye574omtalsoc?filename=Jane%20Austen%20-%20Pride%20and%20Prejudice-CIDEB%20%282000%29.pdf".to_string(),
            pinata: "https://gateway.pinata.cloud/ipfs/bafykbzacedotjioda7arles2s7gyc74hppa3owagm4hmohz4ye574omtalsoc?filename=Jane%20Austen%20-%20Pride%20and%20Prejudice-CIDEB%20%282000%29.pdf".to_string(),
            http: "http://31.42.184.140/main/316000/ab13556b96d473c8dfad7165c4704526/Jane%20Austen%20-%20Pride%20and%20Prejudice-CIDEB%20%282000%29.pdf".to_string(),
        },
        got.unwrap(),
    )
}

fn extract_links(fragment: &Html) -> DownloadLinks {
    let links: Vec<String> = fragment
        .select(&Selector::parse(r#"div[id="download"] a"#).unwrap())
        .map(|element| element.value().attr("href").unwrap().to_string())
        .collect();

    // TODO: return a HashMap of ["name" => "link"] instead?
    DownloadLinks {
        http: links.get(0).unwrap().to_owned(),
        cloudflare: links.get(1).unwrap().to_owned(),
        ipfs_dot_io: links.get(2).unwrap().to_owned(),
        infura: links.get(3).unwrap().to_owned(),
        pinata: links.get(4).unwrap().to_owned(),
    }
}

#[test]
fn test_extract_links() {
    let download_html = r#"
<div id="download">
    <h2><a href="http://some_ip_address/main/316000/some_path/example_filename.pdf">GET</a></h2>
            <div><em>FASTER</em> Download from an IPFS distributed storage, choose any gateway:</div>
    <ul>
        <li><a href="https://cloudflare-ipfs.com/ipfs/example?filename=example_filename.pdf">Cloudflare</a>
        </li><li><a href="https://ipfs.io/ipfs/example?filename=example_filename.pdf">IPFS.io</a>
        </li><li><a href="https://ipfs.infura.io/ipfs/example?filename=example_filename.pdf">Infura</a></li>
        <li><a href="https://gateway.pinata.cloud/ipfs/example?filename=example_filename.pdf">Pinata</a></li>
    </ul>
</div>
"#;

    let fragment = Html::parse_fragment(&download_html);
    let got = extract_links(&fragment);

    assert_eq!(
        "https://cloudflare-ipfs.com/ipfs/example?filename=example_filename.pdf",
        got.cloudflare,
    );
    assert_eq!(
        "https://ipfs.io/ipfs/example?filename=example_filename.pdf",
        got.ipfs_dot_io
    );
    assert_eq!(
        "https://ipfs.infura.io/ipfs/example?filename=example_filename.pdf",
        got.infura
    );
    assert_eq!(
        "https://gateway.pinata.cloud/ipfs/example?filename=example_filename.pdf",
        got.pinata
    );
    assert_eq!(
        "http://some_ip_address/main/316000/some_path/example_filename.pdf",
        got.http
    );
}
