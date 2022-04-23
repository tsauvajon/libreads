//! Module extension provides representation, deserialisation and sorting for
//! ebook extensions.

use serde::{de, Deserialize, Deserializer};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Extension {
    Mobi,
    Epub,
    Azw3,
    Djvu,
    Pdf,
    Doc,
    Other(String),
}

impl std::fmt::Display for Extension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Extension::Mobi => "mobi",
                Extension::Epub => "epub",
                Extension::Azw3 => "azw3",
                Extension::Djvu => "djvu",
                Extension::Pdf => "pdf",
                Extension::Doc => "doc",
                Extension::Other(ext) => ext.as_str(),
            }
        )
    }
}

#[test]
fn test_display_extension() {
    for (ext, want) in vec![
        (Extension::Mobi, "mobi"),
        (Extension::Epub, "epub"),
        (Extension::Azw3, "azw3"),
        (Extension::Djvu, "djvu"),
        (Extension::Pdf, "pdf"),
        (Extension::Doc, "doc"),
        (Extension::Other("hello".to_string()), "hello"),
        (Extension::Other("asdsfdsfds".to_string()), "asdsfdsfds"),
        (Extension::Other("".to_string()), ""),
    ] {
        use std::io::Write;

        let mut out = Vec::<u8>::new();
        write!(out, "{}", ext).unwrap();
        let got = String::from_utf8(out).unwrap();
        assert_eq!(want, got);
    }
}

impl<'de> Deserialize<'de> for Extension {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Value::deserialize(deserializer)?;
        let ext: Option<&str> = Option::deserialize(&v["extension"]).map_err(de::Error::custom)?;
        Ok(match ext {
            Some(ext) => match ext.to_lowercase().as_str() {
                "mobi" => Self::Mobi,
                "epub" => Self::Epub,
                "azw3" => Self::Azw3,
                "djvu" => Self::Djvu,
                "pdf" => Self::Pdf,
                "doc" => Self::Doc,
                ext => Self::Other(ext.to_string()),
            },
            None => Self::Other(String::new()),
        })
    }
}

#[test]
fn test_deserialise_extension() {
    for (data, want) in vec![
        ("pdf", Extension::Pdf),
        ("PDF", Extension::Pdf),
        ("Pdf", Extension::Pdf),
        ("pdF", Extension::Pdf),
        ("PdF", Extension::Pdf),
        ("mobi", Extension::Mobi),
        ("epub", Extension::Epub),
        ("djvu", Extension::Djvu),
        ("azw3", Extension::Azw3),
        (
            "randomextension",
            Extension::Other("randomextension".to_string()),
        ),
        (
            "RANDOMEXTENSION",
            Extension::Other("randomextension".to_string()),
        ),
        ("", Extension::Other(String::new())),
    ] {
        let got: Extension =
            serde_json::from_str(format!(r#"{{ "extension": "{data}" }}"#, data = data).as_str())
                .expect("Should deserialise valid data");
        assert_eq!(want, got);
    }
}

#[test]
fn test_deserialise_missing_extension() {
    let got: Extension = serde_json::from_str("{}").expect("Should deserialise valid data");
    assert_eq!(Extension::Other(String::new()), got)
}

impl Ord for Extension {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        fn val(ext: &Extension) -> u8 {
            match ext {
                Extension::Mobi => 1,
                Extension::Epub => 2,
                Extension::Azw3 => 3,
                Extension::Djvu => 4,
                Extension::Pdf => 90,
                Extension::Doc => 91,
                Extension::Other(_) => 92,
            }
        }

        val(self).cmp(&val(other))
    }
}

impl PartialOrd for Extension {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[test]
fn test_sort_extensions() {
    let mut extensions = vec![
        Extension::Pdf,
        Extension::Other("whatever".to_string()),
        Extension::Mobi,
        Extension::Pdf,
        Extension::Djvu,
        Extension::Epub,
        Extension::Azw3,
        Extension::Doc,
        Extension::Pdf,
        Extension::Mobi,
        Extension::Epub,
        Extension::Doc,
        Extension::Mobi,
        Extension::Pdf,
    ];

    extensions.sort();

    assert_eq!(
        vec![
            Extension::Mobi,
            Extension::Mobi,
            Extension::Mobi,
            Extension::Epub,
            Extension::Epub,
            Extension::Azw3,
            Extension::Djvu,
            Extension::Pdf,
            Extension::Pdf,
            Extension::Pdf,
            Extension::Pdf,
            Extension::Doc,
            Extension::Doc,
            Extension::Other("whatever".to_string()),
        ],
        extensions
    );
}
