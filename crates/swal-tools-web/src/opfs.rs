//! OPFS tools implementation. OPFS is backed by navigator.storage.getDirectory() on wasm32.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(inline_js = r#"
export async function opfs_get_root() {
    if (!globalThis.navigator || !globalThis.navigator.storage) {
        throw new Error("OPFS is not supported: navigator.storage is undefined");
    }
    return await globalThis.navigator.storage.getDirectory();
}

export async function opfs_get_directory_handle(parent, name, create) {
    if (!parent || typeof parent.getDirectoryHandle !== "function") {
        throw new Error("Invalid parent directory handle");
    }
    return await parent.getDirectoryHandle(name, { create });
}

export async function opfs_get_file_handle(parent, name, create) {
    if (!parent || typeof parent.getFileHandle !== "function") {
        throw new Error("Invalid parent directory handle");
    }
    return await parent.getFileHandle(name, { create });
}

export async function opfs_write_file(file_handle, content) {
    if (!file_handle || typeof file_handle.createWritable !== "function") {
        throw new Error("Invalid file handle");
    }
    const writable = await file_handle.createWritable();
    await writable.write(content);
    await writable.close();
}

export async function opfs_read_file(file_handle) {
    if (!file_handle || typeof file_handle.getFile !== "function") {
        throw new Error("Invalid file handle");
    }
    const file = await file_handle.getFile();
    return await file.text();
}

export async function opfs_list_dir(dir_handle) {
    if (!dir_handle || typeof dir_handle.keys !== "function") {
        throw new Error("Invalid directory handle");
    }
    const names = [];
    for await (const name of dir_handle.keys()) {
        names.push(name);
    }
    return names;
}

export async function opfs_delete_file(parent, name) {
    if (!parent || typeof parent.removeEntry !== "function") {
        throw new Error("Invalid parent directory handle");
    }
    await parent.removeEntry(name);
}
"#)]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn opfs_get_root() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn opfs_get_directory_handle(
        parent: &JsValue,
        name: &str,
        create: bool,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn opfs_get_file_handle(
        parent: &JsValue,
        name: &str,
        create: bool,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn opfs_write_file(file_handle: &JsValue, content: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn opfs_read_file(file_handle: &JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn opfs_list_dir(dir_handle: &JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn opfs_delete_file(parent: &JsValue, name: &str) -> Result<JsValue, JsValue>;
}

#[cfg(target_arch = "wasm32")]
fn js_err_to_str(err: JsValue) -> String {
    if let Some(s) = err.as_string() {
        s
    } else if let Some(obj) = err.dyn_ref::<js_sys::Error>() {
        obj.message().into()
    } else {
        format!("{:?}", err)
    }
}

#[cfg(target_arch = "wasm32")]
async fn traverse_to_parent_dir(path: &str, create: bool) -> Result<(JsValue, String), String> {
    let parts = path
        .split('/')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && *s != "." && *s != "..")
        .collect::<Vec<_>>();

    if parts.is_empty() {
        return Err("Path is empty or invalid".to_string());
    }

    let root = opfs_get_root().await.map_err(js_err_to_str)?;
    let mut current_dir = root;

    let (dirs, file_name) = parts.split_at(parts.len() - 1);
    let file_name = file_name[0].to_string();

    for segment in dirs {
        current_dir = opfs_get_directory_handle(&current_dir, segment, create)
            .await
            .map_err(js_err_to_str)?;
    }

    Ok((current_dir, file_name))
}

#[cfg(target_arch = "wasm32")]
async fn traverse_to_dir(path: &str) -> Result<JsValue, String> {
    let parts = path
        .split('/')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && *s != "." && *s != "..")
        .collect::<Vec<_>>();

    let root = opfs_get_root().await.map_err(js_err_to_str)?;
    let mut current_dir = root;

    for segment in parts {
        current_dir = opfs_get_directory_handle(&current_dir, segment, false)
            .await
            .map_err(js_err_to_str)?;
    }

    Ok(current_dir)
}

/// Reads a file from the Origin Private File System (OPFS).
#[cfg(target_arch = "wasm32")]
pub async fn read_file(path: &str) -> Result<String, String> {
    let (parent_dir, file_name) = traverse_to_parent_dir(path, false).await?;
    let file_handle = opfs_get_file_handle(&parent_dir, &file_name, false)
        .await
        .map_err(js_err_to_str)?;
    let content_val = opfs_read_file(&file_handle).await.map_err(js_err_to_str)?;
    content_val
        .as_string()
        .ok_or_else(|| "Failed to convert read content to string".to_string())
}

/// Writes content to a file in the Origin Private File System (OPFS).
#[cfg(target_arch = "wasm32")]
pub async fn write_file(path: &str, content: &str) -> Result<(), String> {
    let (parent_dir, file_name) = traverse_to_parent_dir(path, true).await?;
    let file_handle = opfs_get_file_handle(&parent_dir, &file_name, true)
        .await
        .map_err(js_err_to_str)?;
    opfs_write_file(&file_handle, content)
        .await
        .map_err(js_err_to_str)?;
    Ok(())
}

/// Lists all files/directories at the given directory path in OPFS.
#[cfg(target_arch = "wasm32")]
pub async fn list_dir(path: &str) -> Result<Vec<String>, String> {
    let dir_handle = traverse_to_dir(path).await?;
    let names_val = opfs_list_dir(&dir_handle).await.map_err(js_err_to_str)?;
    let array = names_val
        .dyn_into::<js_sys::Array>()
        .map_err(|_| "Failed to cast list result to Array".to_string())?;
    let mut vec = Vec::with_capacity(array.length() as usize);
    for i in 0..array.length() {
        let val = array.get(i);
        if let Some(s) = val.as_string() {
            vec.push(s);
        }
    }
    Ok(vec)
}

/// Deletes a file in OPFS.
#[cfg(target_arch = "wasm32")]
pub async fn delete_file(path: &str) -> Result<(), String> {
    let (parent_dir, file_name) = traverse_to_parent_dir(path, false).await?;
    opfs_delete_file(&parent_dir, &file_name)
        .await
        .map_err(js_err_to_str)?;
    Ok(())
}

/// Reads a file from the Origin Private File System (OPFS).
#[cfg(not(target_arch = "wasm32"))]
pub async fn read_file(_path: &str) -> Result<String, String> {
    Err("OPFS unavailable on native".to_string())
}

/// Writes content to a file in the Origin Private File System (OPFS).
#[cfg(not(target_arch = "wasm32"))]
pub async fn write_file(_path: &str, _content: &str) -> Result<(), String> {
    Err("OPFS unavailable on native".to_string())
}

/// Lists all files/directories at the given directory path in OPFS.
#[cfg(not(target_arch = "wasm32"))]
pub async fn list_dir(_path: &str) -> Result<Vec<String>, String> {
    Err("OPFS unavailable on native".to_string())
}

/// Deletes a file in OPFS.
#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_file(_path: &str) -> Result<(), String> {
    Err("OPFS unavailable on native".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    fn block_on<F: std::future::Future>(mut f: F) -> F::Output {
        use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

        unsafe fn noop_clone(_: *const ()) -> RawWaker {
            noop_raw_waker()
        }
        unsafe fn noop_wake(_: *const ()) {}
        unsafe fn noop_wake_by_ref(_: *const ()) {}
        unsafe fn noop_drop(_: *const ()) {}

        static VTABLE: RawWakerVTable =
            RawWakerVTable::new(noop_clone, noop_wake, noop_wake_by_ref, noop_drop);

        fn noop_raw_waker() -> RawWaker {
            RawWaker::new(std::ptr::null(), &VTABLE)
        }

        let raw_waker = noop_raw_waker();
        let waker = unsafe { Waker::from_raw(raw_waker) };
        let mut cx = Context::from_waker(&waker);

        let mut f = unsafe { std::pin::Pin::new_unchecked(&mut f) };
        match f.as_mut().poll(&mut cx) {
            Poll::Ready(res) => res,
            Poll::Pending => panic!("Future pending indefinitely on native fallback!"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_native_fallback() {
        let read_res = block_on(read_file("foo.txt"));
        assert_eq!(read_res, Err("OPFS unavailable on native".to_string()));

        let write_res = block_on(write_file("foo.txt", "content"));
        assert_eq!(write_res, Err("OPFS unavailable on native".to_string()));

        let list_res = block_on(list_dir("foo/bar"));
        assert_eq!(list_res, Err("OPFS unavailable on native".to_string()));

        let delete_res = block_on(delete_file("foo.txt"));
        assert_eq!(delete_res, Err("OPFS unavailable on native".to_string()));
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_wasm_dummy() {
        assert!(true);
    }
}
