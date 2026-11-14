#![cfg(target_os = "android")]

use pomme_client::{AndroidApp, LAUNCH_FILE, LaunchArgs, Platform};

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    unsafe {
        libc::prctl(55, 0, 0, 0, 0);
    }
    pomme_client::install_panic_hook();
    let files = app
        .internal_data_path()
        .expect("the app has no internal files directory");
    pomme_client::run(
        LaunchArgs::from_file(&files.join(LAUNCH_FILE)),
        Platform {
            data_root: Some(files.to_path_buf()),
            android_app: app,
        },
    );
}
