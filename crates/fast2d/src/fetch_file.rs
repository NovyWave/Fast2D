use wasm_bindgen_futures::JsFuture;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{window, Response};
use web_sys::js_sys::Uint8Array;

/// Errors that can occur when fetching a file using [`fetch_file`] function.
#[derive(Debug)]
pub enum FetchFileError {
    EmptyUrl,
    NoWindow { url: String },
    FetchFailed { url: String, error: String },
    ResponseCastFailed { url: String, error: String },
    ArrayBufferFailed { url: String, error: String },
    ArrayBufferPromiseFailed { url: String, error: String },
    BufferCastFailed { url: String, error: String },
}

impl std::fmt::Display for FetchFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchFileError::EmptyUrl => write!(f, "URL is empty"),
            FetchFileError::NoWindow { url } => write!(f, "No window available for URL: {}", url),
            FetchFileError::FetchFailed { url, error } => write!(f, "Failed to fetch '{}': {}", url, error),
            FetchFileError::ResponseCastFailed { url, error } => write!(f, "Failed to cast response for '{}': {}", url, error),
            FetchFileError::ArrayBufferFailed { url, error } => write!(f, "Failed to get ArrayBuffer for '{}': {}", url, error),
            FetchFileError::ArrayBufferPromiseFailed { url, error } => write!(f, "Failed to resolve ArrayBuffer promise for '{}': {}", url, error),
            FetchFileError::BufferCastFailed { url, error } => write!(f, "Failed to cast buffer for '{}': {}", url, error),
        }
    }
}

impl std::error::Error for FetchFileError {}

/// Fetches a file from the given URL using the browser's Fetch API and returns its contents as a `Vec<u8>`.
/// 
/// This function is intended for use in WebAssembly (wasm32) environments where access to browser APIs is available.
/// It performs an HTTP(S) request to the specified URL, reads the response as an ArrayBuffer, and converts it to a byte vector.
/// 
/// # Arguments
/// * `url` - The URL of the file to fetch. Must not be empty.
/// 
/// # Returns
/// * `Ok(Vec<u8>)` containing the file's bytes if the fetch and conversion succeed.
/// * `Err(FetchFileError)` if any step fails (e.g., empty URL, fetch error, conversion error).
/// 
/// # Errors
/// Returns a [`FetchFileError`] variant describing the failure reason if the operation does not succeed.
/// 
/// # Example
/// ```no_run
/// let bytes = fetch_file("/assets/image.png").await?;
/// ```
pub async fn fetch_file(url: &str) -> Result<Vec<u8>, FetchFileError> {
    if url.is_empty() {
        return Err(FetchFileError::EmptyUrl);
    }
    // Get the browser window
    let browser_window = window().ok_or(FetchFileError::NoWindow { url: url.to_string() })?;

    // Start the fetch and await the response
    let fetch_promise = browser_window.fetch_with_str(url);
    let fetch_response_jsvalue = JsFuture::from(fetch_promise)
        .await
        .map_err(|error| FetchFileError::FetchFailed { url: url.to_string(), error: format!("{error:?}") })?;
    let response: Response = fetch_response_jsvalue
        .dyn_into()
        .map_err(|error| FetchFileError::ResponseCastFailed { url: url.to_string(), error: format!("{error:?}") })?;

    // Get the ArrayBuffer from the response
    let array_buffer_promise = response
        .array_buffer()
        .map_err(|error| FetchFileError::ArrayBufferFailed { url: url.to_string(), error: format!("{error:?}") })?;
    let array_buffer_jsvalue = JsFuture::from(array_buffer_promise)
        .await
        .map_err(|error| FetchFileError::ArrayBufferPromiseFailed { url: url.to_string(), error: format!("{error:?}") })?;
    let array_buffer = array_buffer_jsvalue
        .dyn_into::<web_sys::js_sys::ArrayBuffer>()
        .map_err(|error| FetchFileError::BufferCastFailed { url: url.to_string(), error: format!("{error:?}") })?;

    // Convert ArrayBuffer to Vec<u8>
    let uint8_array = Uint8Array::new(&array_buffer);
    let mut bytes = vec![0u8; uint8_array.length() as usize];
    uint8_array.copy_to(&mut bytes);
    Ok(bytes)
}
