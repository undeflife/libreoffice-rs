#![allow(
    dead_code,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals
)]
#![allow(clippy::all)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

mod error;

use error::Error;

use core::ffi::c_void;
use std::ffi::{CStr, CString};
/// A Wrapper for the `LibreOfficeKit` C API.
pub struct Office {
    lok: *mut LibreOfficeKit,
    lok_clz: *mut LibreOfficeKitClass,
}
/// A Wrapper for the `LibreOfficeKitDocument` C API.
pub struct Document {
    doc: *mut LibreOfficeKitDocument,
}

impl Office {
    /// Create a new LibreOfficeKit instance.
    ///
    /// # Arguments
    ///
    ///  * `install_path` - The path to the LibreOffice installation.
    ///
    /// # Example
    ///
    /// ```
    /// use libreoffice_rs::Office;
    /// let office = Office::new("/usr/lib/libreoffice/program");
    /// ```
    pub fn new(install_path: &str) -> Result<Office, Error> {
        let c_install_path = CString::new(install_path).unwrap();
        unsafe {
            let lok = lok_init_wrapper(c_install_path.as_ptr());
            let raw_error = (*(*lok).pClass).getError.unwrap()(lok);
            match *raw_error {
                0 => Ok(Office {
                    lok,
                    lok_clz: (*lok).pClass,
                }),
                _ => Err(Error::new(
                    CStr::from_ptr(raw_error).to_string_lossy().into_owned(),
                )),
            }
        }
    }

    ///
    pub fn destroy(&mut self) {
        unsafe {
            (*self.lok_clz).destroy.unwrap()(self.lok);
        }
    }

    /// Returns the last error as a string
    pub fn get_error(&mut self) -> String {
        unsafe {
            let raw_error = (*self.lok_clz).getError.unwrap()(self.lok);
            CStr::from_ptr(raw_error).to_string_lossy().into_owned()
        }
    }

    /// Registers a callback. LOK will invoke this function when it wants to
    /// inform the client about events.
    ///
    /// # Arguments
    ///
    ///  * `callback` - the callback to invoke
    ///  * `user_data` - the user data, will be passed to the callback on invocation
    ///
    pub fn register_callback(&mut self, callback: LibreOfficeKitCallback, data: *mut c_void) {
        unsafe {
            (*self.lok_clz).registerCallback.unwrap()(self.lok, callback, data);
        }
    }

    /// Loads a document from a URL.
    ///
    /// # Arguments
    ///  * `url` - The URL to load.
    pub fn document_load(&mut self, url: &str) -> Result<Document, Error> {
        let c_url = CString::new(url).unwrap();
        unsafe {
            let doc = (*self.lok_clz).documentLoad.unwrap()(self.lok, c_url.as_ptr());
            let error = self.get_error();
            if error != "" {
                return Err(Error::new(error));
            }
            Ok(Document { doc })
        }
    }
    ///
    /// Set password required for loading or editing a document.
    ///
    /// Loading the document is blocked until the password is provided.
    ///
    ///
    /// # Arguments
    ///  * `url` - the URL of the document, as sent to the callback
    ///  * `password` - the password, nullptr indicates no password
    ///
    /// In response to LOK_CALLBACK_DOCUMENT_PASSWORD, a valid password
    /// will continue loading the document, an invalid password will
    /// result in another LOK_CALLBACK_DOCUMENT_PASSWORD request,
    /// and a NULL password will abort loading the document.
    ///
    /// In response to LOK_CALLBACK_DOCUMENT_PASSWORD_TO_MODIFY, a valid
    /// password will continue loading the document, an invalid password will
    /// result in another LOK_CALLBACK_DOCUMENT_PASSWORD_TO_MODIFY request,
    /// and a NULL password will continue loading the document in read-only
    /// mode.
    ///
    /// @since LibreOffice 6.0

    pub fn set_document_password(&mut self, url: &str, password: &str) -> Result<(), Error> {
        let c_url = CString::new(url).unwrap();
        let c_password = CString::new(password).unwrap();
        unsafe {
            (*self.lok_clz).setDocumentPassword.unwrap()(
                self.lok,
                c_url.as_ptr(),
                c_password.as_ptr(),
            );
            let error = self.get_error();
            if error != "" {
                return Err(Error::new(error));
            }
            Ok(())
        }
    }

    /// Loads a document from a URL with additional options.
    ///
    /// # Arguments
    /// * `url` - The URL to load.
    /// * `options` - options for the import filter, e.g. SkipImages.
    ///               Another useful FilterOption is "Language=...".  It is consumed
    ///               by the documentLoad() itself, and when provided, LibreOfficeKit
    ///               switches the language accordingly first.
    ///
    /// # Example
    ///
    /// ```
    /// use libreoffice_rs::Office;
    /// let office = Office::new("/usr/lib/libreoffice/program");
    /// office.document_load_with("./test.odt", "en-US");
    /// ```
    pub fn document_load_with(&mut self, url: &str, options: &str) -> Result<Document, Error> {
        let c_url = CString::new(url).unwrap();
        let c_options = CString::new(options).unwrap();
        unsafe {
            let doc = (*self.lok_clz).documentLoadWithOptions.unwrap()(
                self.lok,
                c_url.as_ptr(),
                c_options.as_ptr(),
            );
            let error = self.get_error();
            if error != "" {
                return Err(Error::new(error));
            }
            Ok(Document { doc })
        }
    }
}

impl Drop for Office {
    fn drop(&mut self) {
        self.destroy()
    }
}
impl Document {
    /// Stores the document's persistent data to a URL and
    /// continues to be a representation of the old URL.
    ///
    /// # Arguments
    /// * `url` - the location where to store the document
    /// * `format` - the format to use while exporting, when omitted, then deducted from pURL's extension
    /// * `filter` -  options for the export filter, e.g. SkipImages.Another useful FilterOption is "TakeOwnership".  It is consumed
    ///               by the saveAs() itself, and when provided, the document identity
    ///               changes to the provided pUrl - meaning that '.uno:ModifiedStatus'
    ///               is triggered as with the "Save As..." in the UI.
    ///              "TakeOwnership" mode must not be used when saving to PNG or PDF.
    ///
    pub fn save_as(&mut self, url: &str, format: &str, filter: Option<&str>) {
        let c_url = CString::new(url).unwrap();
        let c_format: CString = CString::new(format).unwrap();
        let c_filter: CString = CString::new(filter.unwrap_or_default()).unwrap();
        unsafe {
            (*(*self.doc).pClass).saveAs.unwrap()(
                self.doc,
                c_url.as_ptr(),
                c_format.as_ptr(),
                c_filter.as_ptr(),
            );
        }
    }

    pub fn destroy(&mut self) {
        unsafe {
            (*(*self.doc).pClass).destroy.unwrap()(self.doc);
        }
    }
}

impl Drop for Document {
    fn drop(&mut self) {
        self.destroy()
    }
}

#[test]
fn test_convert() {
    let mut office = Office::new("/usr/lib/libreoffice/program").unwrap();
    let mut doc = office.document_load("/tmp/1.doc").unwrap();
    doc.save_as("/tmp/1.png", "png", None);
    assert_eq!(office.get_error(), "".to_string());
}
