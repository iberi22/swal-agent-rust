//! Web Git tools. Real implementation using isomorphic-git via JS interop.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
fn map_js_err(err: wasm_bindgen::JsValue) -> String {
    if let Some(s) = err.as_string() {
        s
    } else if let Some(err_obj) = err.dyn_ref::<js_sys::Error>() {
        err_obj.message().into()
    } else {
        format!("{:?}", err)
    }
}

#[cfg(target_arch = "wasm32")]
fn get_isomorphic_git() -> Result<wasm_bindgen::JsValue, String> {
    let window = web_sys::window().ok_or_else(|| "No global window object found".to_string())?;

    // Check window.__isomorphicGit
    let iso_git_key = js_sys::JsString::from("__isomorphicGit");
    let mut git_obj = js_sys::Reflect::get(&window, &iso_git_key)
        .map_err(|e| format!("Failed to read __isomorphicGit from window: {:?}", e))?;

    if git_obj.is_undefined() || git_obj.is_null() {
        // Fallback to window.git
        let git_key = js_sys::JsString::from("git");
        git_obj = js_sys::Reflect::get(&window, &git_key)
            .map_err(|e| format!("Failed to read git from window: {:?}", e))?;
    }

    if git_obj.is_undefined() || git_obj.is_null() {
        return Err("isomorphic-git unavailable (neither window.__isomorphicGit nor window.git was found)".to_string());
    }

    Ok(git_obj)
}

#[cfg(target_arch = "wasm32")]
fn get_fs_plugin() -> Option<wasm_bindgen::JsValue> {
    let window = web_sys::window()?;

    // Check window.fs or window.__fs
    if let Ok(fs) = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("fs")) {
        if !fs.is_undefined() && !fs.is_null() {
            return Some(fs);
        }
    }
    if let Ok(fs) = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("__fs")) {
        if !fs.is_undefined() && !fs.is_null() {
            return Some(fs);
        }
    }
    None
}

#[cfg(target_arch = "wasm32")]
fn get_http_plugin() -> Option<wasm_bindgen::JsValue> {
    let window = web_sys::window()?;

    // Check window.http, window.__http, window.GitHttp, or window.isomorphicGitHttp
    for key in &["http", "__http", "GitHttp", "isomorphicGitHttp"] {
        if let Ok(http) = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str(key)) {
            if !http.is_undefined() && !http.is_null() {
                return Some(http);
            }
        }
    }
    None
}

#[cfg(target_arch = "wasm32")]
fn create_options_with_fs() -> Result<js_sys::Object, String> {
    let options = js_sys::Object::new();
    if let Some(fs) = get_fs_plugin() {
        js_sys::Reflect::set(&options, &wasm_bindgen::JsValue::from_str("fs"), &fs)
            .map_err(|e| format!("Failed to set fs: {:?}", e))?;
    }
    Ok(options)
}

/// Clones a git repository into the specified directory.
#[cfg(target_arch = "wasm32")]
pub async fn clone_repo(url: &str, dir: &str) -> Result<(), String> {
    let git_obj = get_isomorphic_git()?;
    let clone_key = wasm_bindgen::JsValue::from_str("clone");
    let clone_fn = js_sys::Reflect::get(&git_obj, &clone_key)
        .map_err(|e| format!("Failed to find clone on isomorphic-git: {:?}", e))?;

    if !clone_fn.is_function() {
        return Err("clone is not a function in isomorphic-git".to_string());
    }
    let clone_fn = clone_fn.dyn_into::<js_sys::Function>()
        .map_err(|e| format!("Failed to convert clone to Function: {:?}", e))?;

    let options = create_options_with_fs()?;
    js_sys::Reflect::set(&options, &wasm_bindgen::JsValue::from_str("url"), &wasm_bindgen::JsValue::from_str(url))
        .map_err(|e| format!("Failed to set url: {:?}", e))?;
    js_sys::Reflect::set(&options, &wasm_bindgen::JsValue::from_str("dir"), &wasm_bindgen::JsValue::from_str(dir))
        .map_err(|e| format!("Failed to set dir: {:?}", e))?;

    if let Some(http) = get_http_plugin() {
        js_sys::Reflect::set(&options, &wasm_bindgen::JsValue::from_str("http"), &http)
            .map_err(|e| format!("Failed to set http: {:?}", e))?;
    }

    let promise = clone_fn.call1(&wasm_bindgen::JsValue::UNDEFINED, &options)
        .map_err(|e| format!("Error calling clone: {:?}", e))?;

    let promise = promise.dyn_into::<js_sys::Promise>()
        .map_err(|e| format!("clone did not return a Promise: {:?}", e))?;

    wasm_bindgen_futures::JsFuture::from(promise).await
        .map_err(|e| format!("Clone failed: {}", map_js_err(e)))?;

    Ok(())
}

/// Commits all current changes with the specified commit message.
#[cfg(target_arch = "wasm32")]
pub async fn commit_all(msg: &str) -> Result<(), String> {
    let git_obj = get_isomorphic_git()?;

    // 1. isomorphicGit.add({ fs, dir: ".", filepath: "." })
    let add_key = wasm_bindgen::JsValue::from_str("add");
    let add_fn = js_sys::Reflect::get(&git_obj, &add_key)
        .map_err(|e| format!("Failed to find add on isomorphic-git: {:?}", e))?;

    if !add_fn.is_function() {
        return Err("add is not a function in isomorphic-git".to_string());
    }
    let add_fn = add_fn.dyn_into::<js_sys::Function>()
        .map_err(|e| format!("Failed to convert add to Function: {:?}", e))?;

    let add_options = create_options_with_fs()?;
    js_sys::Reflect::set(&add_options, &wasm_bindgen::JsValue::from_str("dir"), &wasm_bindgen::JsValue::from_str("."))
        .map_err(|e| format!("Failed to set dir: {:?}", e))?;
    js_sys::Reflect::set(&add_options, &wasm_bindgen::JsValue::from_str("filepath"), &wasm_bindgen::JsValue::from_str("."))
        .map_err(|e| format!("Failed to set filepath: {:?}", e))?;

    let add_promise = add_fn.call1(&wasm_bindgen::JsValue::UNDEFINED, &add_options)
        .map_err(|e| format!("Error calling add: {:?}", e))?;

    let add_promise = add_promise.dyn_into::<js_sys::Promise>()
        .map_err(|e| format!("add did not return a Promise: {:?}", e))?;

    wasm_bindgen_futures::JsFuture::from(add_promise).await
        .map_err(|e| format!("Add failed: {}", map_js_err(e)))?;

    // 2. isomorphicGit.commit({ fs, dir: ".", message: msg })
    let commit_key = wasm_bindgen::JsValue::from_str("commit");
    let commit_fn = js_sys::Reflect::get(&git_obj, &commit_key)
        .map_err(|e| format!("Failed to find commit on isomorphic-git: {:?}", e))?;

    if !commit_fn.is_function() {
        return Err("commit is not a function in isomorphic-git".to_string());
    }
    let commit_fn = commit_fn.dyn_into::<js_sys::Function>()
        .map_err(|e| format!("Failed to convert commit to Function: {:?}", e))?;

    let commit_options = create_options_with_fs()?;
    js_sys::Reflect::set(&commit_options, &wasm_bindgen::JsValue::from_str("dir"), &wasm_bindgen::JsValue::from_str("."))
        .map_err(|e| format!("Failed to set dir: {:?}", e))?;
    js_sys::Reflect::set(&commit_options, &wasm_bindgen::JsValue::from_str("message"), &wasm_bindgen::JsValue::from_str(msg))
        .map_err(|e| format!("Failed to set message: {:?}", e))?;

    let author_obj = js_sys::Object::new();
    js_sys::Reflect::set(&author_obj, &wasm_bindgen::JsValue::from_str("name"), &wasm_bindgen::JsValue::from_str("Agent"))
        .map_err(|e| format!("Failed to set author name: {:?}", e))?;
    js_sys::Reflect::set(&author_obj, &wasm_bindgen::JsValue::from_str("email"), &wasm_bindgen::JsValue::from_str("agent@swal.local"))
        .map_err(|e| format!("Failed to set author email: {:?}", e))?;
    js_sys::Reflect::set(&commit_options, &wasm_bindgen::JsValue::from_str("author"), &author_obj)
        .map_err(|e| format!("Failed to set author: {:?}", e))?;

    let commit_promise = commit_fn.call1(&wasm_bindgen::JsValue::UNDEFINED, &commit_options)
        .map_err(|e| format!("Error calling commit: {:?}", e))?;

    let commit_promise = commit_promise.dyn_into::<js_sys::Promise>()
        .map_err(|e| format!("commit did not return a Promise: {:?}", e))?;

    wasm_bindgen_futures::JsFuture::from(commit_promise).await
        .map_err(|e| format!("Commit failed: {}", map_js_err(e)))?;

    Ok(())
}

/// Returns the status of the files in the repository.
#[cfg(target_arch = "wasm32")]
pub async fn status() -> Result<Vec<String>, String> {
    let git_obj = get_isomorphic_git()?;
    let status_key = wasm_bindgen::JsValue::from_str("statusMatrix");
    let status_fn = js_sys::Reflect::get(&git_obj, &status_key)
        .map_err(|e| format!("Failed to find statusMatrix on isomorphic-git: {:?}", e))?;

    if !status_fn.is_function() {
        return Err("statusMatrix is not a function in isomorphic-git".to_string());
    }
    let status_fn = status_fn.dyn_into::<js_sys::Function>()
        .map_err(|e| format!("Failed to convert statusMatrix to Function: {:?}", e))?;

    let options = create_options_with_fs()?;
    js_sys::Reflect::set(&options, &wasm_bindgen::JsValue::from_str("dir"), &wasm_bindgen::JsValue::from_str("."))
        .map_err(|e| format!("Failed to set dir: {:?}", e))?;

    let promise = status_fn.call1(&wasm_bindgen::JsValue::UNDEFINED, &options)
        .map_err(|e| format!("Error calling statusMatrix: {:?}", e))?;

    let promise = promise.dyn_into::<js_sys::Promise>()
        .map_err(|e| format!("statusMatrix did not return a Promise: {:?}", e))?;

    let result_val = wasm_bindgen_futures::JsFuture::from(promise).await
        .map_err(|e| format!("StatusMatrix failed: {}", map_js_err(e)))?;

    let matrix_arr = result_val.dyn_into::<js_sys::Array>()
        .map_err(|_| "statusMatrix result is not an Array".to_string())?;

    let mut results = Vec::new();
    for i in 0..matrix_arr.length() {
        let row_val = matrix_arr.get(i);
        if let Ok(row) = row_val.dyn_into::<js_sys::Array>() {
            if row.length() >= 4 {
                let filepath_val = row.get(0);
                let head_val = row.get(1);
                let workdir_val = row.get(2);
                let stage_val = row.get(3);

                let filepath = filepath_val.as_string().unwrap_or_default();
                let head = head_val.as_f64().unwrap_or(0.0) as i32;
                let workdir = workdir_val.as_f64().unwrap_or(0.0) as i32;
                let stage = stage_val.as_f64().unwrap_or(0.0) as i32;

                if head == 1 && workdir == 1 && stage == 1 {
                    continue;
                }

                let status_str = if head == 0 && workdir == 2 && stage == 0 {
                    format!("? {}", filepath)
                } else if head == 0 && workdir == 2 && stage == 2 {
                    format!("A {}", filepath)
                } else if head == 1 && workdir == 2 && stage == 1 {
                    format!("M {}", filepath)
                } else if head == 1 && workdir == 2 && stage == 2 {
                    format!("M {}", filepath)
                } else if head == 1 && workdir == 0 && stage == 1 {
                    format!("D {}", filepath)
                } else if head == 1 && workdir == 0 && stage == 0 {
                    format!("D {}", filepath)
                } else {
                    format!("{} ({}, {}, {})", filepath, head, workdir, stage)
                };
                results.push(status_str);
            }
        }
    }

    Ok(results)
}

/// Pushes commits to the specified remote repository.
#[cfg(target_arch = "wasm32")]
pub async fn push(remote: &str) -> Result<(), String> {
    let git_obj = get_isomorphic_git()?;
    let push_key = wasm_bindgen::JsValue::from_str("push");
    let push_fn = js_sys::Reflect::get(&git_obj, &push_key)
        .map_err(|e| format!("Failed to find push on isomorphic-git: {:?}", e))?;

    if !push_fn.is_function() {
        return Err("push is not a function in isomorphic-git".to_string());
    }
    let push_fn = push_fn.dyn_into::<js_sys::Function>()
        .map_err(|e| format!("Failed to convert push to Function: {:?}", e))?;

    let options = create_options_with_fs()?;
    js_sys::Reflect::set(&options, &wasm_bindgen::JsValue::from_str("dir"), &wasm_bindgen::JsValue::from_str("."))
        .map_err(|e| format!("Failed to set dir: {:?}", e))?;
    js_sys::Reflect::set(&options, &wasm_bindgen::JsValue::from_str("remote"), &wasm_bindgen::JsValue::from_str(remote))
        .map_err(|e| format!("Failed to set remote: {:?}", e))?;

    if let Some(http) = get_http_plugin() {
        js_sys::Reflect::set(&options, &wasm_bindgen::JsValue::from_str("http"), &http)
            .map_err(|e| format!("Failed to set http: {:?}", e))?;
    }

    let promise = push_fn.call1(&wasm_bindgen::JsValue::UNDEFINED, &options)
        .map_err(|e| format!("Error calling push: {:?}", e))?;

    let promise = promise.dyn_into::<js_sys::Promise>()
        .map_err(|e| format!("push did not return a Promise: {:?}", e))?;

    wasm_bindgen_futures::JsFuture::from(promise).await
        .map_err(|e| format!("Push failed: {}", map_js_err(e)))?;

    Ok(())
}

/// Clones a git repository into the specified directory (native fallback).
#[cfg(not(target_arch = "wasm32"))]
pub async fn clone_repo(_url: &str, _dir: &str) -> Result<(), String> {
    Err("isomorphic-git unavailable on native".into())
}

/// Commits all current changes with the specified commit message (native fallback).
#[cfg(not(target_arch = "wasm32"))]
pub async fn commit_all(_msg: &str) -> Result<(), String> {
    Err("isomorphic-git unavailable on native".into())
}

/// Returns the status of the files in the repository (native fallback).
#[cfg(not(target_arch = "wasm32"))]
pub async fn status() -> Result<Vec<String>, String> {
    Err("isomorphic-git unavailable on native".into())
}

/// Pushes commits to the specified remote repository (native fallback).
#[cfg(not(target_arch = "wasm32"))]
pub async fn push(_remote: &str) -> Result<(), String> {
    Err("isomorphic-git unavailable on native".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;
    use std::task::{Context, Poll, Waker, RawWaker, RawWakerVTable};

    fn block_on<F: Future>(mut future: F) -> F::Output {
        let mut future = unsafe { std::pin::Pin::new_unchecked(&mut future) };
        unsafe fn dummy_clone(_: *const ()) -> RawWaker { RawWaker::new(std::ptr::null(), &VTABLE) }
        unsafe fn dummy_wake(_: *const ()) {}
        unsafe fn dummy_wake_by_ref(_: *const ()) {}
        unsafe fn dummy_drop(_: *const ()) {}
        static VTABLE: RawWakerVTable = RawWakerVTable::new(dummy_clone, dummy_wake, dummy_wake_by_ref, dummy_drop);
        let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
        let mut cx = Context::from_waker(&waker);
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(val) => val,
            Poll::Pending => panic!("Future pending"),
        }
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_native_fallback() {
        assert_eq!(
            block_on(clone_repo("https://github.com/example/repo.git", "dir")),
            Err("isomorphic-git unavailable on native".to_string())
        );
        assert_eq!(
            block_on(commit_all("commit message")),
            Err("isomorphic-git unavailable on native".to_string())
        );
        assert_eq!(
            block_on(status()),
            Err("isomorphic-git unavailable on native".to_string())
        );
        assert_eq!(
            block_on(push("origin")),
            Err("isomorphic-git unavailable on native".to_string())
        );
    }
}
