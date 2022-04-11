//! Crate find_isbn can find ISBN numbers (10 and 13) in a Goodreads HTML page
//! for a book.

use scraper::{Html, Selector};

pub fn find_isbn_10(fragment: &Html) -> Option<String> {
    let selector = Selector::parse(r#"span[itemprop="isbn"]"#).ok()?;
    let span = fragment.select(&selector).next()?;
    let div = span.parent()?.parent()?;

    let content = div.first_child()?.value().as_text()?;
    Some(content.trim().to_string())
}

pub fn find_isbn_13(fragment: &Html) -> Option<String> {
    let selector = Selector::parse(r#"span[itemprop="isbn"]"#).ok()?;
    let span = fragment.select(&selector).next()?;
    Some(span.text().collect())
}

#[cfg(test)]
mod test_find_isbn_10 {
    use super::*;

    #[test]
    fn test_ok() {
        let fragment = r#"<div class="clearFloats">
            <div class="infoBoxRowTitle">ISBN</div>
            <div class="infoBoxRowItem">
                0521405998
                <span class="greyText">(ISBN13: <span itemprop='isbn'>9780521405997</span>)</span>
            </div>
        </div>"#;
        let fragment = Html::parse_fragment(&fragment);

        assert_eq!(Some("0521405998".to_string()), find_isbn_10(&fragment));
    }

    #[test]
    fn test_missing() {
        let fragment = r#"<div class="clearFloats">
            <div class="infoBoxRowTitle">ISBN</div>
            <div class="infoBoxRowItem">
                0521405998
                <span class="greyText">(ISBN13: <span itemprop='something_random'>9780521405997</span>)</span>
            </div>
        </div>"#;
        let fragment = Html::parse_fragment(&fragment);

        assert_eq!(None, find_isbn_10(&fragment));
    }
}

#[cfg(test)]
mod test_find_isbn_13 {
    use super::*;

    #[test]
    fn test_ok() {
        let fragment = r#"<div class="clearFloats">
            <div class="infoBoxRowTitle">ISBN</div>
            <div class="infoBoxRowItem">
                0521405998
                <span class="greyText">(ISBN13: <span itemprop='isbn'>9780521405997</span>)</span>
            </div>
        </div>"#;
        let fragment = Html::parse_fragment(&fragment);

        assert_eq!(Some("9780521405997".to_string()), find_isbn_13(&fragment));
    }

    #[test]
    fn test_missing() {
        let fragment = r#"<div class="clearFloats">
            <div class="infoBoxRowTitle">ISBN</div>
            <div class="infoBoxRowItem">
                0521405998
                <span class="greyText">(ISBN13: <span itemprop='something_random'>9780521405997</span>)</span>
            </div>
        </div>"#;
        let fragment = Html::parse_fragment(&fragment);

        assert_eq!(None, find_isbn_13(&fragment));
    }
}
