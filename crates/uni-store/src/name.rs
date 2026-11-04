//! A universe's name (specs/003-project-file/research.md, R7).

use crate::StoreError;

/// The most Unicode scalar values a name may have.
pub const MAX_NAME_CHARS: usize = 200;

/// A valid universe name: trimmed, and 1 to 200 Unicode scalar values long.
///
/// Apart from trimming, the text is kept exactly as typed: no case changes or Unicode
/// normalisation.
///
/// ```
/// use uni_store::UniverseName;
///
/// let name = UniverseName::new("  Aster ✶ Ñandú  ").unwrap();
/// assert_eq!(name.as_str(), "Aster ✶ Ñandú");
/// assert!(UniverseName::new("   ").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniverseName(String);

impl UniverseName {
    /// Trims `raw` and checks its length, or returns [`StoreError::InvalidName`].
    pub fn new(raw: &str) -> Result<UniverseName, StoreError> {
        let trimmed = raw.trim();
        let chars = trimmed.chars().count();
        if (1..=MAX_NAME_CHARS).contains(&chars) {
            Ok(UniverseName(trimmed.to_owned()))
        } else {
            Err(StoreError::InvalidName)
        }
    }

    /// The name's text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_blank_and_too_long_names_are_rejected() {
        for raw in [String::new(), "   ".to_owned(), "x".repeat(201)] {
            assert_eq!(
                UniverseName::new(&raw),
                Err(StoreError::InvalidName),
                "{raw:?}"
            );
        }
    }

    #[test]
    fn valid_names_are_trimmed_and_otherwise_kept() {
        let long = "é".repeat(200);
        assert_eq!(UniverseName::new(&long).unwrap().as_str(), long);
        assert_eq!(UniverseName::new("  Aster  ").unwrap().as_str(), "Aster");
        assert_eq!(
            UniverseName::new("Aster ✶ Ñandú").unwrap().as_str(),
            "Aster ✶ Ñandú"
        );
    }
}
