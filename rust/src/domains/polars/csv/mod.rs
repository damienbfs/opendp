use std::path::PathBuf;

use polars::prelude::*;

use crate::{
    core::{Domain, MetricSpace},
    error::Fallible,
    transformations::DatasetMetric,
};

#[cfg(feature = "ffi")]
mod ffi;

use super::LazyFrameDomain;

/// # Proof Definition
/// `CsvDomain(F)` is the domain of all CSV files holding data represented by `FrameDomain(F)`.
///
/// # Generics
/// * `F` - LazyFrame or DataFrame
///
/// # Example
/// ```
/// use opendp::domains::{AtomDomain, SeriesDomain, LazyFrameDomain, CsvDomain};
///
/// let lazy_frame_domain = LazyFrameDomain::new(vec![
///             SeriesDomain::new("A", AtomDomain::<i32>::default()),
///             SeriesDomain::new("B", AtomDomain::<f64>::default()),
/// ])?;
///
/// let csv_domain = CsvDomain::new(lazy_frame_domain);
/// # opendp::error::Fallible::Ok(())
/// ```
#[derive(Clone, PartialEq, Debug)]
pub struct CsvDomain {
    pub lazyframe_domain: LazyFrameDomain,
    pub separator: char,
    pub has_header: bool,
    pub skip_rows: usize,
    pub comment_char: Option<char>,
    pub quote_char: Option<char>,
    pub eol_char: char,
    pub null_values: Option<NullValues>,
}

impl CsvDomain {
    pub fn new(lazyframe_domain: LazyFrameDomain) -> Self {
        CsvDomain {
            lazyframe_domain,
            separator: ',',
            has_header: true,
            skip_rows: 0,
            comment_char: None,
            quote_char: Some('"'),
            eol_char: '\n',
            null_values: None,
        }
    }

    /// Set the CSV file's column separator as a byte character
    #[must_use]
    pub fn with_separator(mut self, separator: char) -> Self {
        self.separator = separator;
        self
    }

    /// Set whether the CSV file has headers
    #[must_use]
    pub fn has_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }

    /// Skip the first `n` rows during parsing. The header will be parsed at row `n`.
    #[must_use]
    pub fn with_skip_rows(mut self, skip_rows: usize) -> Self {
        self.skip_rows = skip_rows;
        self
    }

    /// Set the comment character. Lines starting with this character will be ignored.
    #[must_use]
    pub fn with_comment_char(mut self, comment_char: Option<char>) -> Self {
        self.comment_char = comment_char;
        self
    }

    /// Set the `char` used as quote char. The default is `'"'`. If set to `[None]` quoting is disabled.
    #[must_use]
    pub fn with_quote_char(mut self, quote: Option<char>) -> Self {
        self.quote_char = quote;
        self
    }

    /// Set the `char` used as end of line. The default is `'\n'`.
    #[must_use]
    pub fn with_end_of_line_char(mut self, eol_char: char) -> Self {
        self.eol_char = eol_char;
        self
    }

    /// Set values that will be interpreted as missing/ null.
    #[must_use]
    pub fn with_null_values(mut self, null_values: Option<NullValues>) -> Self {
        self.null_values = null_values;
        self
    }

    pub fn new_reader<'a>(&self, path: PathBuf) -> LazyCsvReader<'a> {
        LazyCsvReader::new(path)
            // parsing errors are a side-channel
            .with_ignore_errors(true)
            // fill missing columns with null
            .with_missing_is_null(true)
            .with_schema(Some(Arc::new(self.lazyframe_domain.schema())))
            .with_separator(self.separator as u8)
            .has_header(self.has_header)
            .with_skip_rows(self.skip_rows)
            .with_quote_char(self.quote_char.map(|v| v as u8))
            .with_end_of_line_char(self.eol_char as u8)
            .with_null_values(self.null_values.clone())
    }
}

impl Domain for CsvDomain {
    type Carrier = PathBuf;

    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        self.lazyframe_domain
            .member(&self.new_reader(val.clone()).finish()?)
    }
}

impl<D: DatasetMetric> MetricSpace for (CsvDomain, D)
where
    (LazyFrameDomain, D): MetricSpace,
{
    fn check_space(&self) -> Fallible<()> {
        (self.0.lazyframe_domain.clone(), self.1.clone()).check_space()
    }
}
