[![Rust Stable](https://github.com/F0903/twitch_archiver/actions/workflows/rust.yml/badge.svg)](https://github.com/F0903/twitch_archiver/actions/workflows/rust.yml)

# twitch_archiver

A fast and tiny downloader for Twitch VODs (also sub-only).

Functionality is currently extremely barebones, with only hls stream download and conversion support. No VOD metadata is provided as of yet.

For the CLI program see the [twitch_archiver_cli repo.](https://github.com/F0903/twitch_archiver_cli)

## Usage

```rust
    use twitch_archiver::{
        convert::{convert_hls_to_file},
        Twitch,
    };

    let twitch = Twitch::new(**your_client_id**, **auth_token_if_needed**);
    let hls = twitch.get_hls_manifest("url")?;
    convert_hls_to_file(hls, **your_output_path.mp4**, **optional_ffmpeg_input_args**,**optional_ffmpeg_output_args**)?;
    // Done!
```

### Getting your Client ID

1. Open any Twitch VOD
2. Press F12 to open the dev tools and go to the Network tab. It should now be recording all network requests to and from Twitch
3. Press CTRL+R to reload the page and start from scratch.
4. When the page starts loading, wait until the video starts playing. Then press the red button to stop recording, and scroll to the top of the list.
5. Scroll down through the list until you find a request called "gql".
6. Open this request and find the "Client-Id" header under "Request Headers". Copy the value.

### Downloading sub-only VODs

To download sub only VODs, you need to provide an OAuth token.

To get your OAuth token, follow the method on getting your client id above, but copy the value of the "Authorization" header instead, without the first "OAuth" part.

### Hardware Acceleration

For faster conversion, you can use hardware acceleration arguments provided to FFmpeg when using **convert_hls_to_file**.
For Nvidia GPUs, you can use the following arguments:

> input_args = "-hwaccel cuda"  
> output_args = "-c:v h264_nvenc"
