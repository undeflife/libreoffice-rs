#![allow(
    dead_code,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals
)]
#![allow(clippy::all)]
mod bindings;

pub use bindings::*;

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
    pub fn new(install_path: &str) -> Office {
        let c_install_path = CString::new(install_path).unwrap();
        unsafe {
            let lok = lok_init_wrapper(c_install_path.as_ptr());
            Office {
                lok,
                lok_clz: (*lok).pClass,
            }
        }
    }

    pub fn destroy(&mut self) {
        unsafe {
            (*self.lok_clz).destroy.unwrap()(self.lok);
        }
    }

    pub fn registerCallback(&mut self, callback: LibreOfficeKitCallback, data: *mut c_void) {
        unsafe {
            (*self.lok_clz).registerCallback.unwrap()(self.lok, callback, data);
        }
    }

    pub fn document_load(&mut self, url: &str) -> Document {
        let c_url = CString::new(url).unwrap();
        unsafe {
            let doc = (*self.lok_clz).documentLoad.unwrap()(self.lok, c_url.as_ptr());
            Document { doc }
        }
    }

    pub fn document_load_with(&mut self, url: &str, options: &str) {
        let c_url = CString::new(url).unwrap();
        let c_options = CString::new(options).unwrap();
        unsafe {
            (*self.lok_clz).documentLoadWithOptions.unwrap()(
                self.lok,
                c_url.as_ptr(),
                c_options.as_ptr(),
            );
        }
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
}

#[test]
fn test_convert() {
    let office = Office::new("/opt/libreoffice/instdir/program");
    let doc = office.document_load("/tmp/1.doc");
    doc.save_as("/tmp/1.png", "png", None);
}
