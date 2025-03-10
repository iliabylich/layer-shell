use anyhow::{Context as _, Result, bail};

pub(crate) fn get_weather() -> Result<String> {
    get_weather_response().context("failed to get weather")
}

fn get_weather_response() -> Result<String> {
    let res = minreq::get("http://api.open-meteo.com/v1/forecast")
        .with_param("latitude", "52.2298")
        .with_param("longitude", "21.0118")
        .with_param("current", "temperature_2m,weather_code")
        .with_param("hourly", "temperature_2m,weather_code")
        .with_param(
            "daily",
            "temperature_2m_min,temperature_2m_max,weather_code",
        )
        .with_param("timezone", "Europe/Warsaw")
        .send()
        .context("failed to send weather req")?;

    let status = res.status_code;
    let body = String::from_utf8(res.into_bytes()).context("non-utf8 response")?;

    if status != 200 {
        bail!("Failed to get weather, received non-200 response: {status}\n{body}");
    }

    Ok(body)
}
