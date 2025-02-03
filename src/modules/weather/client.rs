use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    time::Duration,
};

use anyhow::{bail, Context, Result};

pub(crate) fn get_weather() -> Result<Response> {
    let json = get_weather_response().context("failed to get weather")?;

    miniserde::json::from_str(&json).context("failed to parse response body")
}

fn get_weather_response() -> Result<String> {
    const HOST: &str = "api.open-meteo.com";
    const PATH: &str = concat!(
        "/v1/forecast",
        "?",
        "latitude=52.2298",
        "&",
        "longitude=21.0118",
        "&",
        "current=temperature_2m,weather_code",
        "&",
        "hourly=temperature_2m,weather_code",
        "&",
        "daily=temperature_2m_min,temperature_2m_max,weather_code",
        "&",
        "timezone=Europe/Warsaw"
    );

    let addr = format!("{HOST}:80")
        .to_socket_addrs()
        .context("invalid host:port")?
        .find(|addr| matches!(addr, SocketAddr::V4(_)))
        .context("failed to resolve ipv4 of DNS name")?;

    let mut socket = TcpStream::connect_timeout(&addr, Duration::from_secs(2))
        .context("failed to open TCP stream")?;
    socket
        .set_write_timeout(Some(Duration::from_secs(2)))
        .context("failed to set write timeout")?;
    socket
        .set_read_timeout(Some(Duration::from_secs(2)))
        .context("failed to set read timeout")?;

    let request = format!("GET {PATH} HTTP/1.0\r\nHost: {HOST}\r\nConnection: close\r\n\r\n");
    socket
        .write_all(request.as_bytes())
        .context("failed to write")?;

    let mut response = vec![];
    socket
        .read_to_end(&mut response)
        .context("failed to read from socket")?;
    let response = String::from_utf8(response).context("non-utf8 response")?;
    let (headers, body) = response
        .split_once("\r\n\r\n")
        .context("malformed response")?;
    let (line1, headers) = headers.split_once("\r\n").context("malformed response")?;
    let status = line1
        .strip_prefix("HTTP/1.1 ")
        .context("malformed response")?;

    if status != "200 OK" {
        bail!("Failed to get weather, received non-200 response: {status}\n{headers}\n{response}");
    }

    Ok(body.to_string())
}

#[derive(miniserde::Deserialize, Debug)]
pub(crate) struct Response {
    pub(crate) current: CurrentResponse,
    pub(crate) hourly: HourlyResponse,
    pub(crate) daily: DailyResponse,
}

#[derive(miniserde::Deserialize, Debug)]
pub(crate) struct CurrentResponse {
    pub(crate) temperature_2m: f32,
    pub(crate) weather_code: u32,
}

#[derive(miniserde::Deserialize, Debug)]
pub(crate) struct HourlyResponse {
    pub(crate) time: Vec<String>,
    pub(crate) temperature_2m: Vec<f32>,
    pub(crate) weather_code: Vec<u32>,
}

#[derive(miniserde::Deserialize, Debug)]
pub(crate) struct DailyResponse {
    pub(crate) time: Vec<String>,
    pub(crate) temperature_2m_min: Vec<f32>,
    pub(crate) temperature_2m_max: Vec<f32>,
    pub(crate) weather_code: Vec<u32>,
}
