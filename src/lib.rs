#[allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    unnecessary_transmutes
)]
#[allow(clippy::all)]
mod bindings;
pub use bindings::*;
mod error;
pub mod urls;

use error::Error;
use urls::DocUrl;

use std::ffi::{CStr, CString};

/// A Wrapper for the `LibreOfficeKit` C API.
#[derive(Clone)]
pub struct Office {
    lok: *mut LibreOfficeKit,
    lok_clz: *mut LibreOfficeKitClass,
}

/// A Wrapper for the `LibreOfficeKitDocument` C API.
pub struct Document {
    doc: *mut LibreOfficeKitDocument,
}

/// Optional features of LibreOfficeKit, in particular callbacks that block
///  LibreOfficeKit until the corresponding reply is received, which would
///  deadlock if the client does not support the feature.
///
///  @see [Office::set_optional_features]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum LibreOfficeKitOptionalFeatures {
    /// Handle `LOK_CALLBACK_DOCUMENT_PASSWORD` by prompting the user for a password.
    ///
    /// @see [Office::set_document_password]
    LOK_FEATURE_DOCUMENT_PASSWORD = (1 << 0),

    /// Handle `LOK_CALLBACK_DOCUMENT_PASSWORD_TO_MODIFY` by prompting the user for a password.
    ///
    /// @see [Office::set_document_password]
    LOK_FEATURE_DOCUMENT_PASSWORD_TO_MODIFY = (1 << 1),

    /// Request to have the part number as an 5th value in the `LOK_CALLBACK_INVALIDATE_TILES` payload.
    LOK_FEATURE_PART_IN_INVALIDATION_CALLBACK = (1 << 2),

    /// Turn off tile rendering for annotations
    LOK_FEATURE_NO_TILED_ANNOTATIONS = (1 << 3),

    /// Enable range based header data
    LOK_FEATURE_RANGE_HEADERS = (1 << 4),

    /// Request to have the active view's Id as the 1st value in the `LOK_CALLBACK_INVALIDATE_VISIBLE_CURSOR` payload.
    LOK_FEATURE_VIEWID_IN_VISCURSOR_INVALIDATION_CALLBACK = (1 << 5),
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
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut office = Office::new("/usr/lib/libreoffice/program")?;
    ///
    /// assert_eq!("", office.get_error());
    /// # Ok(())
    /// # }
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
    ///  * `cb` - the callback to invoke (type, payload)
    ///
    /// # Example
    ///
    /// ```
    /// use libreoffice_rs::{Office, LibreOfficeKitOptionalFeatures};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut office = Office::new("/usr/lib/libreoffice/program")?;
    /// office.set_optional_features(
    ///    [LibreOfficeKitOptionalFeatures::LOK_FEATURE_DOCUMENT_PASSWORD]
    /// )?;
    ///
    /// office.register_callback(Box::new({
    ///     move |_type, _payload| {
    ///         println!("Call set_document_password and/or do something here!");
    ///     }
    /// }))?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn register_callback<
        F: FnMut(std::os::raw::c_int, *const std::os::raw::c_char) + 'static,
    >(
        &mut self,
        cb: F,
    ) -> Result<(), Error> {
        unsafe {
            /// Callback that Libreoffice will invoke. The actual user callback
            /// is provided as the data value to this callback
            ///
            /// LibreOfficeKitCallback typedef (int nType, const char* pPayload, void* pData);
            unsafe extern "C" fn callback_shim(
                ty: std::os::raw::c_int,
                payload: *const std::os::raw::c_char,
                data: *mut std::os::raw::c_void,
            ) {
                // Get the callback function from the data argument
                let callback: *mut Box<
                    dyn FnMut(std::os::raw::c_int, *const std::os::raw::c_char),
                > = data.cast();

                // Catch panics from calling the callback
                _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                    // Invoke the callback
                    unsafe {
                        (**callback)(ty, payload);
                    }
                }));
            }

            // Wrap the user provided callback and convert it into a pointer
            let user_callback: *mut Box<
                dyn FnMut(std::os::raw::c_int, *const std::os::raw::c_char),
            > = Box::into_raw(Box::new(Box::new(cb)));

            let callback: LibreOfficeKitCallback = Some(callback_shim);

            // Get and invoke the register callback
            let register_callback = (*self.lok_clz)
                .registerCallback
                .expect("missing registerCallback function");

            register_callback(self.lok, callback, user_callback.cast());

            let error = self.get_error();
            if !error.is_empty() {
                return Err(Error::new(error));
            }
        }

        Ok(())
    }

    /// Loads a document from a URL.
    ///
    /// # Arguments
    ///  * `url` - The URL to load.
    ///
    /// # Example
    ///
    /// ```
    /// use libreoffice_rs::Office;
    /// use libreoffice_rs::urls;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut office = Office::new("/usr/lib/libreoffice/program")?;
    /// let doc_url = urls::local_into_abs("./test_data/test.odt")?;
    /// office.document_load(doc_url)?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn document_load(&mut self, url: DocUrl) -> Result<Document, Error> {
        let c_url = CString::new(url.to_string()).unwrap();
        unsafe {
            let doc = (*self.lok_clz).documentLoad.unwrap()(self.lok, c_url.as_ptr());
            let error = self.get_error();
            if !error.is_empty() {
                return Err(Error::new(error));
            }
            Ok(Document { doc })
        }
    }

    /// Set bitmask of optional features supported by the client and return the flags set.
    ///
    /// # Arguments
    ///  * `feature_flags` - The feature flags to set.
    ///
    /// @see [LibreOfficeKitOptionalFeatures]
    ///
    /// @since LibreOffice 6.0
    ///
    /// # Example
    ///
    /// ```
    /// use libreoffice_rs::{Office, LibreOfficeKitOptionalFeatures};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut office = Office::new("/usr/lib/libreoffice/program")?;
    /// let feature_flags = [
    ///     LibreOfficeKitOptionalFeatures::LOK_FEATURE_DOCUMENT_PASSWORD,
    ///     LibreOfficeKitOptionalFeatures::LOK_FEATURE_DOCUMENT_PASSWORD_TO_MODIFY,
    /// ];
    /// let flags_set = office.set_optional_features(feature_flags)?;
    ///
    /// // Integration tests assertions
    /// for feature_flag in feature_flags {
    ///   assert!(flags_set & feature_flag as u64 > 0,
    ///     "Failed to set the flag with value: {}", feature_flag as u64
    ///   );
    /// }
    /// assert!(flags_set &
    /// LibreOfficeKitOptionalFeatures::LOK_FEATURE_PART_IN_INVALIDATION_CALLBACK as u64 == 0,
    ///   "LOK_FEATURE_PART_IN_INVALIDATION_CALLBACK feature was wrongly set!"
    /// );
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_optional_features<T>(&mut self, optional_features: T) -> Result<u64, Error>
    where
        T: IntoIterator<Item = LibreOfficeKitOptionalFeatures>,
    {
        let feature_flags: u64 = optional_features
            .into_iter()
            .map(|i| i as u64)
            .fold(0, |acc, item| acc | item);

        unsafe {
            (*self.lok_clz).setOptionalFeatures.unwrap()(self.lok, feature_flags);
            let error = self.get_error();
            if !error.is_empty() {
                return Err(Error::new(error));
            }
        }

        Ok(feature_flags)
    }

    ///
    /// Set password required for loading or editing a document.
    ///
    /// Loading the document is blocked until the password is provided.
    /// This MUST be used in combination of features and within a callback
    ///
    /// # Arguments
    ///  * `url` - the URL of the document, as sent to the callback
    ///  * `password` - the password, nullptr indicates no password
    ///
    /// In response to `LOK_CALLBACK_DOCUMENT_PASSWORD`, a valid password
    /// will continue loading the document, an invalid password will
    /// result in another `LOK_CALLBACK_DOCUMENT_PASSWORD` request,
    /// and a NULL password will abort loading the document.
    ///
    /// In response to `LOK_CALLBACK_DOCUMENT_PASSWORD_TO_MODIFY`, a valid
    /// password will continue loading the document, an invalid password will
    /// result in another `LOK_CALLBACK_DOCUMENT_PASSWORD_TO_MODIFY` request,
    /// and a NULL password will continue loading the document in read-only
    /// mode.
    ///
    /// @since LibreOffice 6.0
    ///
    /// # Example
    ///
    /// ```
    /// use libreoffice_rs::{Office, LibreOfficeKitOptionalFeatures, urls};
    /// use std::sync::atomic::{AtomicBool, Ordering};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let doc_url = urls::local_into_abs("./test_data/test_password.odt")?;
    /// let password = "test";
    /// let password_was_set = AtomicBool::new(false);
    /// let mut office = Office::new("/usr/lib/libreoffice/program")?;
    ///
    /// office.set_optional_features([LibreOfficeKitOptionalFeatures::LOK_FEATURE_DOCUMENT_PASSWORD])?;
    /// office.register_callback({
    ///     let mut office = office.clone();
    ///     let doc_url = doc_url.clone();
    ///     move |_, _| {
    ///         if !password_was_set.load(Ordering::Acquire) {
    ///             let ret = office.set_document_password(doc_url.clone(), &password);
    ///             password_was_set.store(true, Ordering::Release);
    ///         }
    ///     }
    /// })?;
    ///
    /// let mut _doc = office.document_load(doc_url)?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_document_password(&mut self, url: DocUrl, password: &str) -> Result<(), Error> {
        let c_url = CString::new(url.to_string()).unwrap();
        let c_password = CString::new(password).unwrap();
        unsafe {
            (*self.lok_clz).setDocumentPassword.unwrap()(
                self.lok,
                c_url.as_ptr(),
                c_password.as_ptr(),
            );
            let error = self.get_error();
            if !error.is_empty() {
                return Err(Error::new(error));
            }
            Ok(())
        }
    }

    /// This method provides a defense mechanism against infinite loops, upon password entry failures:
    /// * Loading the document is blocked until a valid password is set within callbacks
    /// * A wrong password will result into infinite repeated callback loops
    /// * This method advises `LibreOfficeKit` to stop requesting a password *"as soon as possible"*
    ///
    /// It is safe for this method to be invoked even if the originally provided password was correct:
    /// - `LibreOfficeKit` appears to maintain thread-local values of the password. It will stick to the first password entry value.
    ///   That will translate into a a successfully loaded document.
    /// - `LibreOfficeKit` seems to send an "excessive" number of callbacks (potential internal issues with locks/monitors)
    ///
    /// # Arguments
    ///  * `url` - the URL of the document, as sent to the callback
    ///
    /// # Example
    ///
    /// ```
    /// use libreoffice_rs::{Office, LibreOfficeKitOptionalFeatures, urls};
    /// use std::sync::atomic::{AtomicBool, Ordering};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let doc_url = urls::local_into_abs("./test_data/test_password.odt")?;
    /// let password = "forgotten_invalid_password_which_is_just_test";
    /// let password_was_set = AtomicBool::new(false);
    /// let failed_password_attempt = AtomicBool::new(false);
    /// let mut office = Office::new("/usr/lib/libreoffice/program")?;
    ///
    /// office.set_optional_features([LibreOfficeKitOptionalFeatures::LOK_FEATURE_DOCUMENT_PASSWORD])?;
    /// office.register_callback({
    ///     let mut office = office.clone();
    ///     let doc_url = doc_url.clone();
    ///     move |_, _| {
    ///         if !password_was_set.load(Ordering::Acquire) {
    ///             let ret = office.set_document_password(doc_url.clone(), &password);
    ///             password_was_set.store(true, Ordering::Release);
    ///         } else {
    ///             if !failed_password_attempt.load(Ordering::Acquire) {
    ///                 let ret = office.unset_document_password(doc_url.clone());
    ///                 failed_password_attempt.store(true, Ordering::Release);
    ///             }
    ///         }
    ///     }
    /// })?;
    ///
    /// assert!(office.document_load(doc_url).is_err(),
    ///         "Document loaded successfully with a wrong password!");
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn unset_document_password(&mut self, url: DocUrl) -> Result<(), Error> {
        let c_url = CString::new(url.to_string()).unwrap();
        unsafe {
            (*self.lok_clz).setDocumentPassword.unwrap()(
                self.lok,
                c_url.as_ptr(),
                std::ptr::null(),
            );
            let error = self.get_error();
            if !error.is_empty() {
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
    ///   Another useful FilterOption is "Language=...".  It is consumed
    ///   by the documentLoad() itself, and when provided, LibreOfficeKit
    ///   switches the language accordingly first.
    ///
    /// # Example
    ///
    /// ```
    /// use libreoffice_rs::{Office, urls};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut office = Office::new("/usr/lib/libreoffice/program")?;
    /// let doc_url = urls::local_into_abs("./test_data/test.odt")?;
    /// office.document_load_with(doc_url, "en-US")?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn document_load_with(&mut self, url: DocUrl, options: &str) -> Result<Document, Error> {
        let c_url = CString::new(url.to_string()).unwrap();
        let c_options = CString::new(options).unwrap();
        unsafe {
            let doc = (*self.lok_clz).documentLoadWithOptions.unwrap()(
                self.lok,
                c_url.as_ptr(),
                c_options.as_ptr(),
            );
            let error = self.get_error();
            if !error.is_empty() {
                return Err(Error::new(error));
            }
            Ok(Document { doc })
        }
    }

    /// Runs a macro stored at a specific path (within a document).
    ///
    /// # Arguments
    /// * `path` - The macro path (macro:///Standard.Module1.MyMacro).
    pub fn run_macro(&mut self, path: &str) -> Result<(), Error> {
        let path = CString::new(path).unwrap();
        unsafe {
            let x = (*self.lok_clz).runMacro.unwrap()(self.lok, path.as_ptr());
            if x == 0 {
                let error = self.get_error();
                if !error.is_empty() {
                    return Err(Error::new(error));
                }
            }
            Ok(())
        }
    }
}

impl Drop for Office {
    fn drop(&mut self) {
        unsafe {
            (*self.lok_clz).destroy.unwrap()(self.lok);
        }
    }
}

impl Document {
    /// Stores the document's persistent data to a URL and
    /// continues to be a representation of the old URL.
    ///
    /// If the result is not true, then there's an error (possibly unsupported format or other errors)
    ///
    /// # Arguments
    /// * `url` - the location where to store the document
    /// * `format` - the format to use while exporting, when omitted, then deducted from pURL's extension
    /// * `filter` -  options for the export filter, e.g. SkipImages.Another useful FilterOption is "TakeOwnership".  It is consumed
    ///   by the saveAs() itself, and when provided, the document identity
    ///   changes to the provided pUrl - meaning that '.uno:ModifiedStatus'
    ///   is triggered as with the "Save As..." in the UI.
    ///   "TakeOwnership" mode must not be used when saving to PNG or PDF.
    ///
    /// # Example
    ///
    /// ```
    /// use libreoffice_rs::Office;
    /// use libreoffice_rs::urls;
    ///
    /// # fn  main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut office = Office::new("/usr/lib/libreoffice/program")?;
    /// let doc_url = urls::local_into_abs("./test_data/test.odt")?;
    /// let mut doc = office.document_load(doc_url)?;
    /// let output_path = std::env::temp_dir().join("libreoffice_rs_save_as.png");
    /// let output_location = output_path.display().to_string();
    /// let previously_saved = doc.save_as(&output_location, "png", None);
    /// let _ = std::fs::remove_file(&output_path);
    ///
    /// assert!(previously_saved, "{}", office.get_error());
    ///
    /// #  Ok(())
    /// # }
    /// ```
    pub fn save_as(&mut self, url: &str, format: &str, filter: Option<&str>) -> bool {
        let c_url = CString::new(url).unwrap();
        let c_format: CString = CString::new(format).unwrap();
        let c_filter: CString = CString::new(filter.unwrap_or_default()).unwrap();
        let ret = unsafe {
            (*(*self.doc).pClass).saveAs.unwrap()(
                self.doc,
                c_url.as_ptr(),
                c_format.as_ptr(),
                c_filter.as_ptr(),
            )
        };

        ret != 0
    }
}

impl Drop for Document {
    fn drop(&mut self) {
        unsafe {
            (*(*self.doc).pClass).destroy.unwrap()(self.doc);
        }
    }
}
