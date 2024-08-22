use crate::login_mode::login_mode;
use chrono::Utc;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::{Builder, Runtime};

use jni::objects::{JClass, JString};
use jni::sys::jint;
use jni::JNIEnv;
use once_cell::sync::OnceCell;
use reqwest::Client;
use tokio::time::sleep;

// #[no_mangle]
// #[used]
// pub static mut RUNNING: i8 = 0;

pub static STATUS: OnceCell<Arc<Mutex<i8>>> = OnceCell::new();

pub fn get_status() -> i8 {
    let value = STATUS.get_or_init(|| Arc::new(Mutex::new(0)));
    *value.lock().unwrap()
}

pub fn set_status(status: i8) {
    let value = STATUS.get_or_init(|| Arc::new(Mutex::new(0)));
    let mut val = value.lock().unwrap();
    *val = status;
}

pub fn create_current_thread_runtime() -> Arc<Runtime> {
    let runtime = Arc::new(
        Builder::new_current_thread()
            .thread_name("blockmesh-cli")
            .enable_all()
            .build()
            .unwrap(),
    );
    runtime
}

/// # Safety
/// This method should be called by any external program that want to use BlockMesh Network CLI
/// cbindgen:ignore
#[no_mangle]
pub unsafe extern "C" fn Java_xyz_blockmesh_runLib(
    mut env: JNIEnv,
    _class: JClass,
    url: JString,
    email: JString,
    password: JString,
) -> jint {
    let runtime = create_current_thread_runtime();
    let url: String = match env.get_string(&url) {
        Ok(s) => s.into(),
        Err(e) => {
            eprintln!("Failed to load url {}", e);
            return -1;
        }
    };

    let email: String = match env.get_string(&email) {
        Ok(s) => s.into(),
        Err(e) => {
            eprintln!("Failed to load email {}", e);
            return -1;
        }
    };

    let password: String = match env.get_string(&password) {
        Ok(s) => s.into(),
        Err(e) => {
            eprintln!("Failed to load password {}", e);
            return -1;
        }
    };

    runtime.block_on(async {
        let _ = login_mode(&url, &email, &password, None).await;
    });
    -1
}

/// # Safety
/// This method should be called by any external program that want to use BlockMesh Network CLI
#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn stop_lib() -> i8 {
    let runtime = create_current_thread_runtime();
    runtime.block_on(async {
        let _ = Client::new()
            .get(format!(
                "{}/health_check?RUNNING={}&stop_lib=before",
                "http://localhost:8000",
                get_status()
            ))
            .send()
            .await;
        set_status(-1);
        let _ = Client::new()
            .get(format!(
                "{}/health_check?RUNNING={}&stop_lib=after",
                "http://localhost:8000",
                get_status()
            ))
            .send()
            .await;
    });
    get_status()
}

/// # Safety
/// This method should be called by any external program that want to use BlockMesh Network CLI
#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn run_lib(
    url: *const c_char,
    email: *const c_char,
    password: *const c_char,
) -> i8 {
    if get_status() == 1 {
        return 0;
    }
    let url = match unsafe { CStr::from_ptr(url) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load url {}", e);
            return -1;
        }
    };
    let _email = match unsafe { CStr::from_ptr(email) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load email {}", e);
            return -1;
        }
    };
    let _password = match unsafe { CStr::from_ptr(password) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load password {}", e);
            return -1;
        }
    };

    let runtime = create_current_thread_runtime();
    runtime.block_on(async {
        set_status(1);
        loop {
            if get_status() == -1 {
                break;
            }
            let v = get_status();
            set_status(v + 1);
            let now = Utc::now();
            let _ = Client::new()
                .get(format!(
                    "{}/health_check?time={}&RUNNING={}",
                    url,
                    now,
                    get_status()
                ))
                .send()
                .await;
            sleep(Duration::from_secs(5)).await
        }
        // let _ = login_mode(url, email, password).await;
    });
    set_status(0);
    -1
}
