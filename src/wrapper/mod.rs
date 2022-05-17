mod bindings;

#[link(name = "wrapper")]
extern "C" {
    fn lok_init_wrapper(
        install_path: *const ::std::os::raw::c_char,
    ) -> *mut bindings::LibreOfficeKit;
}

pub use bindings::*;

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
        unsafe {
            let lok = lok_init_wrapper(CString::new(install_path).unwrap().as_ptr());
            Office {
                lok: lok,
                lok_clz: (*lok).pClass,
            }
        }
    }

    pub fn destroy(&mut self) {
        unsafe {
            (*self.lok_clz).destroy.unwrap()(self.lok);
        }
    }

    pub fn document_load(&self, url: &str) -> Document {
        unsafe {
            let doc = (*self.lok_clz).documentLoad.unwrap()(
                self.lok,
                CString::new(url).unwrap().as_ptr(),
            );
            Document { doc: doc }
        }
    }

    pub fn document_load_with(&self, url: &str, options: &str) {
        unsafe {
            (*self.lok_clz).documentLoadWithOptions.unwrap()(
                self.lok,
                CString::new(url).unwrap().as_ptr(),
                CString::new(options).unwrap().as_ptr(),
            );
        }
    }
}
impl Document {
    pub fn save_as(&self, url: &str, format: &str, filter: Option<&str>) {
        unsafe {
            (*(*self.doc).pClass).saveAs.unwrap()(
                self.doc,
                CString::new(url).unwrap().as_ptr(),
                CString::new(format).unwrap().as_ptr(),
                CString::new(filter.unwrap_or_default()).unwrap().as_ptr(),
            );
        }
    }
}
