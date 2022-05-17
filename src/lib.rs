mod wrapper;

#[test]
fn test_convert() {
    use crate::wrapper::Office;
    let office = Office::new("/opt/libreoffice/instdir/program");
    let doc = office.document_load("/tmp/1.doc");
    doc.save_as("/tmp/1.png", "png", None);
}
