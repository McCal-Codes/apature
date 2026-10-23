//! Start the server on a temporary folder, log in with the session password,
//! upload a file, and find it on disk with the same bytes.
use std::io::Cursor;

#[test]
fn upload_lands_in_the_folder() {
    let dir = std::env::temp_dir().join(format!("apature-receive-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let port = 2190 + (std::process::id() % 50) as u16;
    let rx = apature_receive::Receiver::start(&dir, port).expect("server starts");
    std::thread::sleep(std::time::Duration::from_millis(300));

    let mut ftp = suppaftp::FtpStream::connect(("127.0.0.1", port)).expect("connect");
    ftp.login(apature_receive::USER, &rx.password)
        .expect("login");
    let bytes = vec![7u8; 10_000];
    ftp.put_file("DSC_0001.JPG", &mut Cursor::new(bytes.clone()))
        .expect("put");
    ftp.quit().ok();

    let got = std::fs::read(dir.join("DSC_0001.JPG")).expect("file on disk");
    assert_eq!(got, bytes);
    assert_eq!(rx.logins(), 1);

    let mut bad = suppaftp::FtpStream::connect(("127.0.0.1", port)).unwrap();
    assert!(bad.login(apature_receive::USER, "000000").is_err() || rx.password == "000000");
    drop(rx);
    let _ = std::fs::remove_dir_all(&dir);
}
