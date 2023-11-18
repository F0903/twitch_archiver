use std::{
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn convert_to_file(input: &mut impl Read, out_file: impl AsRef<Path>) -> Result<()> {
    let args = format!(
        "-protocol_whitelist http,https,tls,tcp,file,pipe -y -f hls -i pipe:0 {}",
        out_file.as_ref().display()
    );
    let mut ffmpeg = Command::new("ffmpeg")
        .stdin(Stdio::piped())
        .args(args.split_whitespace())
        .spawn().map_err(|_| "FFmpeg.exe could not be started!\nPlease install it by either downloading and placing the executable in the same directoy as twitch_archiver, or install it globally by adding the FFmpeg directory to your PATH variable (or equivalent for non-windows systems)\nhttps://ffmpeg.org/download.html")?;

    {
        let mut stdin = ffmpeg
            .stdin
            .take()
            .ok_or("Could not pipe input to ffmpeg!")?;

        let mut buf = String::default();
        input.read_to_string(&mut buf)?;
        // Set all streams to be autoselected and default so it wont be ignored by FFMPEG.
        let buf = buf.replace("AUTOSELECT=NO,DEFAULT=NO", "AUTOSELECT=YES,DEFAULT=YES");

        stdin.write_all(buf.as_bytes())?;
        stdin.flush()?;
    } // Required to drop stdin or it will hang.

    let status = ffmpeg.wait()?;
    if !status.success() {
        return Err(
            "FFmpeg did not exit with code 0 (OK). Please check FFmpeg output logs above.".into(),
        );
    }

    Ok(())
}
