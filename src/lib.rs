#![allow(
    dead_code,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals
)]
#![allow(clippy::all)]
mod bindings;
mod error;

pub use bindings::*;
pub use error::Error;

use core::ffi::c_void;
use std::ffi::CString;

pub struct Office {
    lok: *mut LibreOfficeKit,
    lok_clz: *mut LibreOfficeKitClass,
}

pub struct Document {
    doc: *mut LibreOfficeKitDocument,
}

impl Office {
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
                    CString::from_raw(raw_error).into_string().unwrap(),
                )),
            }
        }
    }

    pub fn destroy(&mut self) {
        unsafe {
            (*self.lok_clz).destroy.unwrap()(self.lok);
        }
    }

    pub fn get_error(&mut self) -> String {
        unsafe {
            let raw_error = (*self.lok_clz).getError.unwrap()(self.lok);
            CString::from_raw(raw_error).into_string().unwrap()
        }
    }

    pub fn registerCallback(&mut self, callback: LibreOfficeKitCallback, data: *mut c_void) {
        unsafe {
            (*self.lok_clz).registerCallback.unwrap()(self.lok, callback, data);
        }
    }

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
        println!("drop office");
        self.destroy()
    }
}
impl Document {
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
        println!("drop document");
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
