use std::{
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn convert_to_file(mut input: impl Read, out_file: impl AsRef<Path>) -> Result<()> {
    let mut ffmpeg = Command::new("ffmpeg")
        .stdin(Stdio::piped())
        .arg("-protocol_whitelist")
        .arg("http,https,tls,tcp,file,pipe")
        .arg("-i")
        .arg("pipe:0")
        .arg(out_file.as_ref())
        .spawn().map_err(|_| "FFmpeg.exe could not be started!\nPlease install it by either downloading and placing the executable in the same directoy as twitch_archiver, or install it globally by adding the FFmpeg directory to your PATH variable (or equivalent for non-windows systems)\nhttps://ffmpeg.org/download.html")?;
    let mut stdin = ffmpeg
        .stdin
        .take()
        .ok_or("Could not pipe input to ffmpeg!")?;
    let mut buf = [0; 12 * 1024];
    let mut count;
    while {
        count = input.read(&mut buf)?;
        count > 0
    } {
        stdin.write(&buf[..count])?;
    }
    stdin.flush()?;
    drop(stdin); // Required or it will hang
    ffmpeg.wait()?;
    Ok(())
}
