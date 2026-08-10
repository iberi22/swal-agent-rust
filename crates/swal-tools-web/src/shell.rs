//! Web Shell tools via WebContainers (real impl).
//!
//! This module provides a sandboxed shell bridge to the WebContainers JS API
//! for the wasm32 target, and clean fallbacks for native targets.
//!
//! Note: A native subprocess shell is an explicit non-goal of this crate,
//! as WebContainers represents a sandboxed browser-only shell.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(inline_js = r#"
let webcontainerInstance = null;

export function check_webcontainer_exists() {
    return typeof window !== 'undefined' && typeof window.WebContainer !== 'undefined';
}

export async function run_webcontainer_cmd(cmd, workdir) {
    if (!check_webcontainer_exists()) {
        throw new Error("WebContainer is not available on window");
    }
    if (!webcontainerInstance) {
        webcontainerInstance = await window.WebContainer.boot();
    }
    const spawnOptions = {};
    if (workdir) {
        spawnOptions.cwd = workdir;
    }
    const process = await webcontainerInstance.spawn("bash", ["-c", cmd], spawnOptions);

    let output = "";
    const writable = new WritableStream({
        write(chunk) {
            if (typeof chunk === 'string') {
                output += chunk;
            } else if (chunk instanceof Uint8Array) {
                output += new TextDecoder().decode(chunk);
            } else {
                output += String(chunk);
            }
        }
    });

    await process.output.pipeTo(writable);
    const exitCode = await process.exit;
    if (exitCode !== 0) {
        throw new Error(`Command failed with exit code ${exitCode}. Output: ${output}`);
    }
    return output;
}
"#)]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn run_webcontainer_cmd(cmd: &str, workdir: Option<&str>) -> Result<JsValue, JsValue>;

    fn check_webcontainer_exists() -> bool;
}

#[cfg(target_arch = "wasm32")]
fn js_val_to_string(val: JsValue) -> String {
    if let Some(s) = val.as_string() {
        s
    } else if let Some(err) = val.dyn_ref::<js_sys::Error>() {
        err.message().into()
    } else {
        match js_sys::JSON::stringify(&val) {
            Ok(js_str) => js_str.into(),
            Err(_) => format!("{:?}", val),
        }
    }
}

/// Checks if the WebContainers environment is available.
pub fn is_available() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        check_webcontainer_exists()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

/// Runs a command in the WebContainers shell environment.
pub async fn run_cmd(cmd: &str) -> Result<String, String> {
    #[cfg(target_arch = "wasm32")]
    {
        match run_webcontainer_cmd(cmd, None).await {
            Ok(val) => Ok(js_val_to_string(val)),
            Err(err) => Err(js_val_to_string(err)),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = cmd;
        Err("WebContainers unavailable on native".to_string())
    }
}

/// Runs a command in the WebContainers shell environment with a specific working directory.
pub async fn run_cmd_in(workdir: &str, cmd: &str) -> Result<String, String> {
    #[cfg(target_arch = "wasm32")]
    {
        match run_webcontainer_cmd(cmd, Some(workdir)).await {
            Ok(val) => Ok(js_val_to_string(val)),
            Err(err) => Err(js_val_to_string(err)),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = cmd;
        let _ = workdir;
        Err("WebContainers unavailable on native".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block_on_immediate<F: std::future::Future>(mut f: F) -> F::Output {
        let mut f = unsafe { std::pin::Pin::new_unchecked(&mut f) };
        let waker = unsafe {
            std::task::Waker::from_raw(std::task::RawWaker::new(
                std::ptr::null(),
                &std::task::RawWakerVTable::new(
                    |_| std::task::RawWaker::new(std::ptr::null(), &VTABLE),
                    |_| {},
                    |_| {},
                    |_| {},
                ),
            ))
        };
        static VTABLE: std::task::RawWakerVTable = std::task::RawWakerVTable::new(
            |_| std::task::RawWaker::new(std::ptr::null(), &VTABLE),
            |_| {},
            |_| {},
            |_| {},
        );
        let mut cx = std::task::Context::from_waker(&waker);
        match f.as_mut().poll(&mut cx) {
            std::task::Poll::Ready(val) => val,
            std::task::Poll::Pending => panic!("Future was pending!"),
        }
    }

    #[test]
    fn test_shell_native() {
        assert_eq!(is_available(), false);
        let res = block_on_immediate(run_cmd("echo 123"));
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "WebContainers unavailable on native");

        let res_in = block_on_immediate(run_cmd_in("dir", "echo 123"));
        assert!(res_in.is_err());
        assert_eq!(res_in.unwrap_err(), "WebContainers unavailable on native");
    }
}
