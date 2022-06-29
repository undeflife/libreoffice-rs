use std::fmt;
use std::path::Path;
use url::Url;

use crate::error::Error;

/// Type-safe URL "container" for LibreOffice documents
#[derive(Debug, Clone)]
pub struct DocUrl(String);

impl fmt::Display for DocUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Construct a type-safe `DocUrl` instance for a given path
/// - This method **does check** if the file actually exists, which you may not want
/// - If the provided file path is relative, then it'll be converted to an absolute path
/// - Once the absolute path is obtained, this delegates to [local_as_abs]
///
/// # Arguments
/// * `path` - An relative or absolute path that for an existing local file
///
/// ```
/// use libreoffice_rs::urls;
///
/// # fn  main() -> Result<(), Box<dyn std::error::Error>> {
/// let valid_url_ret = urls::local_into_abs("./test_data/test.odt");
/// assert!(valid_url_ret.is_ok(), "{}", valid_url_ret.err().unwrap());
///
/// let invalid_url_ret = urls::local_into_abs("does_not_exist.odt");
/// assert!(invalid_url_ret.is_err(), "Computed an absolute path URL for a nonexistent file!");
///
/// #  Ok(())
/// # }
/// ```
pub fn local_into_abs<S: Into<String>>(path: S) -> Result<DocUrl, Error> {
    let doc_path = path.into();

    match std::fs::canonicalize(&doc_path) {
        Ok(doc_abspath) => local_as_abs(doc_abspath.display().to_string()),
        Err(ex) => {
            let msg = format!("Does the file exist at {}? {}", doc_path, ex.to_string());
            Err(Error::new(msg))
        }
    }
}

/// Construct a type-safe `DocUrl` instance for a given absolute local path
/// - This method doesn't check if the file actually exists yet
/// - The provided file path must be an absolute location, per LibreOffice expectations
///
/// # Arguments
/// * `path` - An absolute path on the local filesystem
///
/// ```
/// use libreoffice_rs::urls;
///
/// # fn  main() -> Result<(), Box<dyn std::error::Error>> {
/// let relative_path_ret = urls::local_as_abs("./test_data/test.odt");
/// assert!(relative_path_ret.is_err(), "{}", relative_path_ret.err().unwrap());
///
/// #  Ok(())
/// # }
/// ```
pub fn local_as_abs<S: Into<String>>(path: S) -> Result<DocUrl, Error> {
    let uri_location = path.into();
    let p = Path::new(&uri_location);

    if !p.is_absolute() {
        return Err(Error::new(format!("The file path {} must be absolute!", &uri_location)));
    }

    let url_ret = Url::from_file_path(&uri_location);

    match url_ret {
        Ok(url_value) => {
            Ok(DocUrl(url_value.as_str().to_owned()))
        },
        Err(ex) => {
            return Err(Error::new(format!("Failed to parse as URL {}! {:?}", uri_location, ex)));
        }
    }
}

/// Construct a type-safe DocUrl instance if the document remote URI is valid
///
/// # Arguments
/// * `uri` - A document URI
///
/// # Example
///
/// ```
/// use libreoffice_rs::urls;
///
/// # fn  main() -> Result<(), Box<dyn std::error::Error>> {
/// let valid_url_ret = urls::remote("http://google.com");
///
/// assert!(valid_url_ret.is_ok(), "{:?}", valid_url_ret.err());
///
/// #  Ok(())
/// # }
/// ```
pub fn remote<S: Into<String>>(uri: S) -> Result<DocUrl, Error> {
    let uri_location = uri.into();
    let uri_location_str = uri_location.as_str();

    if let Err(ex) = Url::parse(uri_location_str) {
        return Err(Error::new(format!("Failed to parse URI {}! {}", uri_location, ex.to_string())));
    }

    Ok(DocUrl(uri_location))
}
