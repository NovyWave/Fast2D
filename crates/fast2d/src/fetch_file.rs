use wasm_bindgen_futures::JsFuture;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{window, Response};
use web_sys::js_sys::Uint8Array;

/// Fetch a file from the given URL and return its bytes.
pub async fn fetch_file(url: &str) -> Result<Vec<u8>, String> {
    if url.is_empty() {
        return Err("URL is empty".to_string());
    }
    // Get the browser window
    let browser_window = window().ok_or("No window")?;

    // Start the fetch and await the response
    let fetch_promise = browser_window.fetch_with_str(url);
    let fetch_response_jsvalue = JsFuture::from(fetch_promise)
        .await
        .map_err(|error| format!("fetch failed for '{url}': {error:?}"))?;
    let response: Response = fetch_response_jsvalue
        .dyn_into()
        .map_err(|error| format!("response cast failed for '{url}': {error:?}"))?;

    // Get the ArrayBuffer from the response
    let array_buffer_promise = response
        .array_buffer()
        .map_err(|error| format!("array_buffer failed for '{url}': {error:?}"))?;
    let array_buffer_jsvalue = JsFuture::from(array_buffer_promise)
        .await
        .map_err(|error| format!("array_buffer promise failed for '{url}': {error:?}"))?;
    let array_buffer = array_buffer_jsvalue
        .dyn_into::<web_sys::js_sys::ArrayBuffer>()
        .map_err(|error| format!("buffer cast failed for '{url}': {error:?}"))?;

    // Convert ArrayBuffer to Vec<u8>
    let uint8_array = Uint8Array::new(&array_buffer);
    let mut bytes = vec![0u8; uint8_array.length() as usize];
    uint8_array.copy_to(&mut bytes);
    Ok(bytes)
}
