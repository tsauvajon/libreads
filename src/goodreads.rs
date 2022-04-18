//! Module goodreads can find ISBN numbers (10 and 13) in a Goodreads HTML page
//! for a book.

use async_trait::async_trait;
use regex::Regex;
use scraper::{Html, Selector};

#[derive(Debug, PartialEq)]
pub struct BookIdentification {
    pub isbn10: Option<String>,
    pub isbn13: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
}

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait BookIdentificationGetter {
    async fn get_identification(
        &self,
        page_url: &str,
    ) -> Result<BookIdentification, reqwest::Error>;
}

#[derive(Default)]
pub struct Goodreads {}

impl Goodreads {
    fn find_isbn_10(&self, fragment: &Html) -> Option<String> {
        let selector = Selector::parse(r#"span[itemprop="isbn"]"#).ok()?;
        let span = fragment.select(&selector).next()?;
        let div = span.parent()?.parent()?;

        let content = div.first_child()?.value().as_text()?;
        Some(content.trim().to_string())
    }

    fn find_isbn_13(&self, fragment: &Html) -> Option<String> {
        let selector = Selector::parse(r#"span[itemprop="isbn"]"#).ok()?;
        let span = fragment.select(&selector).next()?;
        Some(span.text().collect())
    }

    fn find_title(&self, fragment: &Html) -> Option<String> {
        let selector =
            Selector::parse(r#"h1[data-testid="bookTitle"], h1[id="bookTitle"]"#).ok()?;
        let span = fragment.select(&selector).next()?;
        Some(span.text().collect::<String>().trim().to_string())
    }

    fn find_author(&self, fragment: &Html) -> Option<String> {
        let selector =
            Selector::parse(r#"div[class="ContributorLinksList"] span[data-testid="name"], a[class="authorName"] span[itemprop="name"]"#)
                .ok()?;
        let span = fragment.select(&selector).next()?;

        let raw_author: String = span.text().collect();
        let re = Regex::new(r"\s+").unwrap();
        let author = re.replace_all(raw_author.as_str(), " ");

        Some(author.to_string())
    }
}

#[async_trait]
impl BookIdentificationGetter for Goodreads {
    async fn get_identification(
        &self,
        page_url: &str,
    ) -> Result<BookIdentification, reqwest::Error> {
        let body = reqwest::get(page_url).await?.text().await?;

        let document = Html::parse_document(&body);
        let isbn10 = self.find_isbn_10(&document);
        let isbn13 = self.find_isbn_13(&document);
        let title = self.find_title(&document);
        let author = self.find_author(&document);

        Ok(BookIdentification {
            isbn10,
            isbn13,
            title,
            author,
        })
    }
}

#[cfg(test)]
mod test_find_isbn_10 {
    use super::*;

    #[test]
    fn test_ok() {
        let fragment = r#"
        <div class="clearFloats">
            <div class="infoBoxRowTitle">ISBN</div>
            <div class="infoBoxRowItem">
                0521405998
                <span class="greyText">(ISBN13: <span itemprop='isbn'>9780521405997</span>)</span>
            </div>
        </div>"#;
        let fragment = Html::parse_fragment(&fragment);

        assert_eq!(
            Some("0521405998".to_string()),
            Goodreads::default().find_isbn_10(&fragment)
        );
    }

    #[test]
    fn test_missing() {
        let fragment = r#"
        <div class="clearFloats">
            <div class="infoBoxRowTitle">ISBN</div>
            <div class="infoBoxRowItem">
                0521405998
                <span class="greyText">(ISBN13: <span itemprop='something_random'>9780521405997</span>)</span>
            </div>
        </div>"#;
        let fragment = Html::parse_fragment(&fragment);

        assert_eq!(None, Goodreads::default().find_isbn_10(&fragment));
    }
}

#[cfg(test)]
mod test_find_isbn_13 {
    use super::*;

    #[test]
    fn test_ok() {
        let fragment = r#"
        <div class="clearFloats">
            <div class="infoBoxRowTitle">ISBN</div>
            <div class="infoBoxRowItem">
                0521405998
                <span class="greyText">(ISBN13: <span itemprop='isbn'>9780521405997</span>)</span>
            </div>
        </div>"#;
        let fragment = Html::parse_fragment(&fragment);

        assert_eq!(
            Some("9780521405997".to_string()),
            Goodreads::default().find_isbn_13(&fragment)
        );
    }

    #[test]
    fn test_missing() {
        let fragment = r#"
        <div class="clearFloats">
            <div class="infoBoxRowTitle">ISBN</div>
            <div class="infoBoxRowItem">
                0521405998
                <span class="greyText">(ISBN13: <span itemprop='something_random'>9780521405997</span>)</span>
            </div>
        </div>"#;
        let fragment = Html::parse_fragment(&fragment);

        assert_eq!(None, Goodreads::default().find_isbn_13(&fragment));
    }
}

#[cfg(test)]
mod test_find_title {
    use super::*;

    #[test]
    fn test_ok() {
        let fragment = Html::parse_fragment(include_str!(
            "../tests/testdata/goodreads_1984_book_page.html"
        ));

        assert_eq!(
            Some("1984".to_string()),
            Goodreads::default().find_title(&fragment)
        );
    }

    #[test]
    fn test_ok_alternative_layout() {
        let fragment = Html::parse_fragment(include_str!(
            "../tests/testdata/goodreads_origin_of_species_curl_page.html"
        ));

        assert_eq!(
            Some("The Origin of Species".to_string()),
            Goodreads::default().find_title(&fragment)
        );
    }

    #[test]
    fn test_missing() {
        let fragment = r#"
        <div class="BookPageTitleSection">
            <div class="BookPageTitleSection__title">
                <h1 class="Text Text__title1" data-testid="bookTitle_unexpectedId" aria-label="Book title: 1984">1984
                </h1>
            </div>
        </div>"#;
        let fragment = Html::parse_fragment(&fragment);

        assert_eq!(None, Goodreads::default().find_title(&fragment));
    }
}

#[cfg(test)]
mod test_find_author {
    use super::*;

    #[test]
    fn test_ok() {
        let fragment = Html::parse_fragment(include_str!(
            "../tests/testdata/goodreads_1984_book_page.html"
        ));

        assert_eq!(
            Some("George Orwell".to_string()),
            Goodreads::default().find_author(&fragment)
        );
    }

    #[test]
    fn test_ok_alternative_layout() {
        let fragment = Html::parse_fragment(include_str!(
            "../tests/testdata/goodreads_origin_of_species_curl_page.html"
        ));

        assert_eq!(
            Some("Charles Darwin".to_string()),
            Goodreads::default().find_author(&fragment)
        );
    }

    #[test]
    fn test_missing() {
        let fragment = r#" <div class="BookPageMetadataSection__contributor">
        <h3 class="Text Text__title3 Text__regular" aria-label="List of contributors">
            <div class="ContributorLinksList">
                <div class="Button__container"><button type="button"
                        class="Button Button--inline Button--small"
                        aria-label="Show all contributors"><span
                            class="Button__labelItem">...more</span></button></div>
            </div>
        </h3>
    </div>"#;
        let fragment = Html::parse_fragment(&fragment);

        assert_eq!(None, Goodreads::default().find_author(&fragment));
    }
}
