use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::blocking::Client;
use std::{error::Error, fmt::Display, io::Read};

#[derive(Debug)]
struct TwitchError {
    message: String,
}

impl TwitchError {
    pub fn with_msg(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for TwitchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Encountered the following Twitch error:\n{}",
            self.message
        ))
    }
}

impl Error for TwitchError {}

type TwitchResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct VODAuthentication {
    signature: String,
    auth_token: String,
}

const TWITCH_GQL: &str = "https://gql.twitch.tv/gql";
const TWITCH_GQL_TOKEN_REQ_PAYLOAD_TEMPLATE: &str = r#"{"operationName":"PlaybackAccessToken_Template","query":"query PlaybackAccessToken_Template($login: String!, $isLive: Boolean!, $vodID: ID!, $isVod: Boolean!, $playerType: String!) {  streamPlaybackAccessToken(channelName: $login, params: {platform: \"web\", playerBackend: \"mediaplayer\", playerType: $playerType}) @include(if: $isLive) {    value    signature    __typename  }  videoPlaybackAccessToken(id: $vodID, params: {platform: \"web\", playerBackend: \"mediaplayer\", playerType: $playerType}) @include(if: $isVod) {    value    signature    __typename  }}","variables":{"isLive":false,"login":"","isVod":true,"vodID":"{}","playerType":"site"}}"#;
const TWITCH_VOD: &str = "https://usher.ttvnw.net/vod/";

pub const TWITCH_VOD_URL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?:http|https):\/\/(?:www.)?twitch.tv\/videos\/(\d+)"#).unwrap());

pub struct Twitch {
    client: Client,
    client_id: String,
    auth_token: Option<String>,
}

impl Twitch {
    pub fn new(client_id: impl Into<String>, auth_token: Option<impl Into<String>>) -> Self {
        Self {
            client: Client::new(),
            client_id: client_id.into(),
            auth_token: auth_token.map(|x| x.into()),
        }
    }

    pub fn set_auth_token(&mut self, auth_token: Option<impl Into<String>>) {
        self.auth_token = auth_token.map(|x| x.into())
    }

    fn get_auth(&self, vod_id: &str) -> TwitchResult<VODAuthentication> {
        let auth_token = match self.auth_token.as_deref() {
            Some(x) => "OAuth ".to_owned() + &x,
            None => "undefined".to_owned(),
        };
        let payload = TWITCH_GQL_TOKEN_REQ_PAYLOAD_TEMPLATE.replace("{}", vod_id);
        let token_req_res = self
            .client
            .post(TWITCH_GQL)
            .header("Authorization", auth_token)
            .header("Client-ID", &self.client_id)
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
        Ok(VODAuthentication {
            signature: auth_sig.to_owned(),
            auth_token: auth_value.to_owned(),
        })
    }

    pub fn parse_id_from_url<'a>(url: &'a str) -> TwitchResult<&'a str> {
        let captures = match TWITCH_VOD_URL_REGEX.captures(url) {
            Some(x) => x,
            None => match url.parse::<u64>() {
                Ok(_) => return Ok(url),
                _ => {
                    return Err(TwitchError::with_msg("Please enter a valid url or vod ID.").into())
                }
            },
        };

        let id = captures.get(1).ok_or("Could not parse ID from url!")?;
        let id_str = id.as_str();
        Ok(id_str)
    }

    pub fn get_hls_manifest<'a>(&mut self, url: &'a str) -> TwitchResult<impl Read> {
        let id = Self::parse_id_from_url(url)?;
        let auth = self.get_auth(id)?;
        let req_url = format!(
            "{}{}.m3u8?allow_source=true&sig={}&token={}",
            TWITCH_VOD,
            id,
            auth.signature,
            crate::util::make_string_url_friendly(&auth.auth_token)
        );

        let result = self.client.get(req_url).send()?;

        if let Err(err) = result.error_for_status_ref() {
            return Err(
                TwitchError::with_msg(format!("Could not download VOD. Error:\n{}", err)).into(),
            );
        }

        Ok(result)
    }
}
