use serde::Deserialize;

// Request:
// http://libgen.rs/json.php?isbn=9788853001351&fields=Title,Author,Year,Extension,MD5
//
// Response:
// [{"title":"Pride and Prejudice","author":"Jane Austen","year":"2000","extension":"pdf","md5":"ab13556b96d473c8dfad7165c4704526"}]

const BASE_URL: &str = "http://libgen.rs/json.php";

#[derive(Deserialize, Clone)]
pub struct LibgenMetadata {
    pub title: String,
    pub author: String,
    pub year: String,
    pub extension: String, // TODO: enum
    pub md5: String,
}

pub enum _Extension {
    Mobi,
    Epub,
}

pub async fn get_metadata(isbn: &str) -> Result<Option<LibgenMetadata>, reqwest::Error> {
    let url = format!(
        "{base_url}?isbn={isbn}&fields=Title,Author,Year,Extension,MD5",
        base_url = BASE_URL,
        isbn = isbn,
    );

    let mut response: Vec<LibgenMetadata> = reqwest::get(url).await?.json().await?;
    if response.is_empty() {
        return Ok(None);
    }

    response.sort_by(|a, b| a.extension.cmp(&b.extension));

    Ok(Some(response[0].clone()))
}

#[tokio::test]
async fn test_ok() {
    // /!\ This test calls the LibGen API!!
    // TODO: mock instead

    let got = get_metadata("9788853001351")
        .await
        .expect("The call to LibGen should succeed");
    assert!(got.is_some());
    let got = got.unwrap();

    assert_eq!("Pride and Prejudice", got.title.as_str());
    assert_eq!("Jane Austen", got.author.as_str());
}
