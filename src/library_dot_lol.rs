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

pub struct LibraryDotLol {
    pub base_url: String,
}

#[async_trait]
impl DownloadLinksStore for LibraryDotLol {
    async fn get_download_links(&self, id: &str) -> Result<DownloadLinks, reqwest::Error> {
        let page_url = format!("{base_url}/{id}", base_url = self.base_url, id = id);
        let body = reqwest::get(page_url).await?.text().await?;
        let document = Html::parse_document(&body);

        Ok(extract_links(&document))
    }
}

fn extract_links(fragment: &Html) -> DownloadLinks {
    let links: Vec<String> = fragment
        .select(&Selector::parse(r#"div[id="download"] a"#).unwrap())
        .map(|element| element.value().attr("href").unwrap().to_string())
        .collect();

    // TODO: return a HashMap of ["name" => "link"] instead of hardcoding sources?
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

impl Default for LibraryDotLol {
    fn default() -> Self {
        Self {
            base_url: BASE_URL.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_download_links() {
        use httpmock::{Method::GET, MockServer};

        let mock_server = MockServer::start();
        let lib_dot_lol = LibraryDotLol {
            base_url: mock_server.base_url(),
        };

        let endpoint_mock = mock_server.mock(|when, then| {
            when.method(GET).path("/AB13556B96D473C8DFAD7165C4704526");
            then.status(200)
                .header("content-type", "text/html")
                .body(include_str!("../tests/testdata/library.lol_book_page.html"));
        });
        let got = lib_dot_lol
            .get_download_links("AB13556B96D473C8DFAD7165C4704526")
            .await;

        endpoint_mock.assert();
        assert!(got.is_ok());
        assert_eq!(
            DownloadLinks {
                cloudflare: "https://cloudflare-ipfs.com/ipfs/example.pdf".to_string(),
                ipfs_dot_io: "https://ipfs.io/ipfs/example.pdf".to_string(),
                infura: "https://ipfs.infura.io/ipfs/example.pdf".to_string(),
                pinata: "https://gateway.pinata.cloud/ipfs/example.pdf".to_string(),
                http: "http://12.34.45.67/main/316000/example.pdf".to_string(),
            },
            got.unwrap(),
        );
    }
}
