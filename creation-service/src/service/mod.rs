pub mod diagram;
pub mod entity;
pub mod kinship_derivation;
pub mod person;
pub mod relationship;
pub mod transaction;
pub mod tree_path;
pub mod user;

pub(crate) fn normalize_name(name: &str, max_chars: usize) -> Option<String> {
    let trimmed = name.trim();

    if trimmed.is_empty() || trimmed.chars().count() > max_chars {
        return None;
    }

    Some(trimmed.to_string())
}

pub(crate) fn normalize_optional_text(text: Option<String>) -> Option<String> {
    text.and_then(|value| {
        let trimmed = value.trim();

        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub(crate) fn normalize_optional_text_with_max_chars(
    text: Option<String>,
    max_chars: usize,
) -> Option<Option<String>> {
    match normalize_optional_text(text) {
        Some(value) if value.chars().count() > max_chars => None,
        Some(value) => Some(Some(value)),
        None => Some(None),
    }
}

macro_rules! map_service_result {
    ($future:expr, $error:path) => {{
        match $future.await {
            Err(err) => Err($error(err)),
            Ok(val) => Ok(val),
        }
    }};
}

macro_rules! map_service_result_unit {
    ($future:expr, $error:path) => {{
        match $future.await {
            Err(err) => Err($error(err)),
            Ok(_) => Ok(()),
        }
    }};
}

pub(crate) use map_service_result;
pub(crate) use map_service_result_unit;

#[cfg(test)]
mod tests {
    use super::{map_service_result, map_service_result_unit};

    #[derive(Debug, PartialEq)]
    enum WrappedError {
        Inner(u8),
    }

    #[test]
    fn map_service_result_maps_ok_and_err() {
        let ok = futures::executor::block_on(async {
            map_service_result!(async { Ok::<u16, u8>(42) }, WrappedError::Inner)
        });
        assert_eq!(ok, Ok(42));

        let err = futures::executor::block_on(async {
            map_service_result!(async { Err::<u16, u8>(7) }, WrappedError::Inner)
        });
        assert_eq!(err, Err(WrappedError::Inner(7)));
    }

    #[test]
    fn map_service_result_unit_maps_ok_and_err() {
        let ok = futures::executor::block_on(async {
            map_service_result_unit!(async { Ok::<usize, u8>(1) }, WrappedError::Inner)
        });
        assert_eq!(ok, Ok(()));

        let err = futures::executor::block_on(async {
            map_service_result_unit!(async { Err::<usize, u8>(3) }, WrappedError::Inner)
        });
        assert_eq!(err, Err(WrappedError::Inner(3)));
    }

    #[test]
    fn normalize_name_trims_and_validates_length() {
        assert_eq!(
            super::normalize_name("  sample  ", 10),
            Some("sample".to_string())
        );
        assert_eq!(super::normalize_name("   ", 10), None);
        assert_eq!(super::normalize_name("abcdef", 5), None);
    }

    #[test]
    fn normalize_optional_text_trims_and_empty_becomes_none() {
        assert_eq!(
            super::normalize_optional_text(Some("  value  ".to_string())),
            Some("value".to_string())
        );
        assert_eq!(
            super::normalize_optional_text(Some("   ".to_string())),
            None
        );
        assert_eq!(super::normalize_optional_text(None), None);
    }

    #[test]
    fn normalize_optional_text_with_max_chars_validates_length() {
        assert_eq!(
            super::normalize_optional_text_with_max_chars(Some("  value  ".to_string()), 10),
            Some(Some("value".to_string()))
        );
        assert_eq!(
            super::normalize_optional_text_with_max_chars(Some("   ".to_string()), 10),
            Some(None)
        );
        assert_eq!(
            super::normalize_optional_text_with_max_chars(Some("abcdef".to_string()), 5),
            None
        );
    }
}
