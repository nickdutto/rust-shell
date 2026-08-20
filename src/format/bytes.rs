pub fn convert_bytes(bytes: u64, unit: &str) -> f64 {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    const TB: f64 = 1024.0 * 1024.0 * 1024.0 * 1024.0;

    // TODO: loss of precision possible
    let bytes_f = bytes as f64;

    match unit {
        "KB" => bytes_f / KB,
        "MB" => bytes_f / MB,
        "GB" => bytes_f / GB,
        "TB" => bytes_f / TB,
        _ => bytes_f,
    }
}
