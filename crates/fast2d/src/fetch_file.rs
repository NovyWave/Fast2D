pub async fn fetch_file(url: &str) -> Result<Vec<u8>, String> {
    use wasm_bindgen_futures::JsFuture;
    use web_sys::wasm_bindgen::JsCast;
    use web_sys::{window, Response};
    use web_sys::js_sys::Uint8Array;

    let win = window().ok_or("No window")?;
    let resp_value = JsFuture::from(win.fetch_with_str(url))
        .await
        .map_err(|_| "fetch failed".to_string())?;
    let resp: Response = resp_value.dyn_into().map_err(|_| "response cast failed".to_string())?;
    let buffer_promise = resp.array_buffer().map_err(|_| "array_buffer failed".to_string())?;
    let buffer_value = JsFuture::from(buffer_promise)
        .await
        .map_err(|_| "array_buffer promise failed".to_string())?;
    let buffer = buffer_value.dyn_into::<web_sys::js_sys::ArrayBuffer>().map_err(|_| "buffer cast failed".to_string())?;
    let u8arr = Uint8Array::new(&buffer);
    let mut bytes = vec![0u8; u8arr.length() as usize];
    u8arr.copy_to(&mut bytes[..]);
    Ok(bytes)
}
