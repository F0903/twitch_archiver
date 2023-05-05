# twitch_archiver

A blazingly fast, small, and simple downloader for Twitch VODs.
Simply pass in a url, and give it an optional output path with an extension of your choosing, which the video will then be converted to.

## Usage

Either run the executable normally as an interactive shell, or start the exe with parameters.

### Subscriber-only VODs

To download subscriber-only VODs, you need to provide an OAuth token either by setting it through **auth token set** or by passing it to the get command with the --auth switch.

To get your OAuth token, open any Twitch VOD, then press F12 to open the dev tools, then go to the Network tab. It should now be recording all network requests to and from Twitch. Then press CTRL+R to reload the page and start from scratch.
When the page starts loading, wait until the video starts playing. Then press the red button to stop recording, and scroll to the top of the list. Now, scroll down through the list until you find a request called "gql". Open this request, and find the "Authorization" header under "Request Headers", and copy the value except for the "OAuth" part.

## Commands

get **url** _optional_auth_token_ _optional_output_path_

> Downloads specified VOD.  
> Example: **get https://www.twitch.tv/videos/1199379108 vod.mp4**

auth token set **auth_token**

> Sets the auth token to use in requests. Saved in settings.json.

auth token get

> Gets the current auth token.
