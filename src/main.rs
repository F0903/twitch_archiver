mod convert;

use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::blocking::Client;
use std::{env::args, error::Error, io::stdin, path::Path};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

struct Authentication {
    signature: String,
    auth_value: String,
}

const TWITCH_GQL: &str = "https://gql.twitch.tv/gql";
const TWITCH_GQL_TOKEN_REQ_PAYLOAD_TEMPLATE: &str = r#"{"operationName":"PlaybackAccessToken_Template","query":"query PlaybackAccessToken_Template($login: String!, $isLive: Boolean!, $vodID: ID!, $isVod: Boolean!, $playerType: String!) {  streamPlaybackAccessToken(channelName: $login, params: {platform: \"web\", playerBackend: \"mediaplayer\", playerType: $playerType}) @include(if: $isLive) {    value    signature    __typename  }  videoPlaybackAccessToken(id: $vodID, params: {platform: \"web\", playerBackend: \"mediaplayer\", playerType: $playerType}) @include(if: $isVod) {    value    signature    __typename  }}","variables":{"isLive":false,"login":"","isVod":true,"vodID":"{}","playerType":"site"}}"#;
const TWITCH_VOD: &str = "https://usher.ttvnw.net/vod/";

const TWITCH_VOD_URL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?:http|https):\/\/(?:www.)?twitch.tv\/videos\/(\d+)"#).unwrap());

fn make_string_url_friendly(input: String) -> String {
    let mut strbuf = String::with_capacity(input.capacity());
    for ch in input.chars() {
        match ch {
            '!' => strbuf.push_str("%21"),
            '"' => strbuf.push_str("%22"),
            '$' => strbuf.push_str("%23"),
            '\'' => strbuf.push_str("%27"),
            '(' => strbuf.push_str("%28"),
            ')' => strbuf.push_str("%29"),
            '*' => strbuf.push_str("%2A"),
            '+' => strbuf.push_str("%2B"),
            ',' => strbuf.push_str("%2C"),
            '-' => strbuf.push_str("%2D"),
            '.' => strbuf.push_str("%2E"),
            '/' => strbuf.push_str("%2F"),
            ':' => strbuf.push_str("%3A"),
            ';' => strbuf.push_str("%3B"),
            '@' => strbuf.push_str("%40"),
            '[' => strbuf.push_str("%5B"),
            '\\' => strbuf.push_str("%5C"),
            ']' => strbuf.push_str("%5D"),
            '{' => strbuf.push_str("%7B"),
            '}' => strbuf.push_str("%7D"),
            _ => strbuf.push(ch),
        }
    }
    strbuf
}

fn get_auth(vod_id: &str, client: &Client) -> Result<Authentication> {
    let payload = TWITCH_GQL_TOKEN_REQ_PAYLOAD_TEMPLATE.replace("{}", vod_id);
    let token_req_res = client
        .post(TWITCH_GQL)
        .header("Client-ID", "kimne78kx3ncx6brgo4mv6wki5h1ko")
        .header("Host", "gql.twitch.tv")
        .body(payload)
        .send()?;
    token_req_res.error_for_status_ref()?;
    let auth_json = token_req_res.json::<serde_json::Value>()?;
    let auth_access = &auth_json["data"]["videoPlaybackAccessToken"];
    let auth_sig = auth_access["signature"]
        .as_str()
        .ok_or("Auth signature was not present in response!")?;
    let auth_value = auth_access["value"]
        .as_str()
        .ok_or("Auth value was not present in response!")?;
    Ok(Authentication {
        signature: auth_sig.to_owned(),
        auth_value: auth_value.to_owned(),
    })
}

fn parse_id_from_url<'a>(url: &'a str) -> Result<&'a str> {
    let captures = match TWITCH_VOD_URL_REGEX.captures(url) {
        Some(x) => x,
        None => match url.parse::<u64>() {
            Ok(_) => return Ok(url),
            _ => return Err("Please enter a valid url or vod ID.".into()),
        },
    };

    let id = captures.get(1).ok_or("Could not parse ID from url!")?;
    let id_str = id.as_str();
    Ok(id_str)
}

fn download<'a>(client: &Client, mut args: impl Iterator<Item = &'a str>) -> Result<()> {
    let url = args.next().ok_or("No URL or VOD ID provided.")?;
    let id = parse_id_from_url(url)?;
    let auth = get_auth(id, &client)?;
    let req_url = format!(
        "{}{}.m3u8?sig={}&token={}",
        TWITCH_VOD,
        id,
        auth.signature,
        make_string_url_friendly(auth.auth_value)
    );
    let mut result = client.get(req_url).send()?;
    result.error_for_status_ref()?;
    let out_path = match args.next() {
        Some(x) => Path::new(x),
        None => {
            println!("No output path provided... Will use default ttv_vod.mp4");
            Path::new("ttv_vod.mp4")
        }
    };
    convert::convert_to_file(&mut result, out_path)?;
    Ok(())
}

fn parse_cmd(input: &str, client: &Client) -> Result<()> {
    let mut words = input.split_whitespace();
    let cmd_word = words.next().ok_or("No command specified")?;
    match cmd_word {
        "get" => download(client, words),
        _ => Err(format!("Unknown command. \"{}\"", cmd_word).into()),
    }?;
    println!("Success!");
    Ok(())
}

fn parse_commands(client: &Client) -> Result<()> {
    let stdin = stdin();
    loop {
        let mut stdin_buf = String::default();
        let count = stdin.read_line(&mut stdin_buf)?;
        println!();
        let input = &stdin_buf[..count];
        if let Err(err) = parse_cmd(input, client) {
            println!("Error:\n{}", err);
        }
    }
}

fn run_interactive(client: &Client) -> Result<()> {
    parse_commands(&client)
}

fn run_once(args: impl Iterator<Item = String>, client: &Client) -> Result<()> {
    let mut strbuf = String::with_capacity(20);
    for arg in args {
        strbuf.push_str(&arg);
        strbuf.push(' ');
    }
    parse_cmd(&strbuf, client)
}

fn main() -> Result<()> {
    let client = Client::builder().build()?;
    let mut args = args();
    if args.len() > 1 {
        args.next(); // Consume the first arg, as it is the program path.
        run_once(args, &client)
    } else {
        run_interactive(&client)
    }
}
