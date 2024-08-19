use std::sync::atomic::{AtomicBool, Ordering};

use libreoffice_rs::{urls, LibreOfficeKitOptionalFeatures, Office};

#[test]
#[ignore = "requires libreoffice to run this test"]
fn test_password_callback() {
    let doc_url = urls::local_into_abs("./test_data/test_password.odt").unwrap();
    let password = "test";
    let password_was_set = AtomicBool::new(false);
    let mut office = Office::new("/usr/lib/libreoffice/program").unwrap();
    office
        .set_optional_features([LibreOfficeKitOptionalFeatures::LOK_FEATURE_DOCUMENT_PASSWORD])
        .unwrap();

    const LOK_CALLBACK_DOCUMENT_PASSWORD: i32 = 20;

    office
        .register_callback({
            let mut office = office.clone();
            let doc_url = doc_url.clone();
            move |ty, _| {
                if ty == LOK_CALLBACK_DOCUMENT_PASSWORD && !password_was_set.load(Ordering::Acquire)
                {
                    office
                        .set_document_password(doc_url.clone(), password)
                        .unwrap();
                    password_was_set.store(true, Ordering::Release);
                }
            }
        })
        .unwrap();
    let mut _doc = office.document_load(doc_url).unwrap();
}
