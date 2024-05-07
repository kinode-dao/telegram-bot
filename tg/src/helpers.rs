pub fn request_no_wait<T1: serde::ser::Serialize>(
    api_url: &str,
    method: &str,
    params: Option<T1>,
) -> Result<(), anyhow::Error> {
    let url = format!("{}/{method}", api_url);
    let url = url::Url::from_str(&url)?;

    let headers: HashMap<String, String> =
        HashMap::from_iter([("Content-Type".into(), "application/json".into())]);

    let body = if let Some(ref params) = params {
        serde_json::to_vec(params)?
    } else {
        Vec::new()
    };
    send_request(Method::GET, url, Some(headers), Some(20), body);
    Ok(())
}