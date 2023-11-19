use std::{
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
};

/// Converts hls stream to a user specified format by output pathfile extension.
/// input_args and output_args are passed directly to FFmpeg.
/// Requires FFmpeg in system PATH or working directory to work.
pub fn convert_hls_to_file(
    input: &mut impl Read,
    out_file: impl AsRef<Path>,
    input_args: Option<&str>,
    output_args: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let args = format!(
        "-y -loglevel info -protocol_whitelist http,https,tls,tcp,file,pipe -f hls {} -i pipe:0 {} {}",
        input_args.unwrap_or(""),
        output_args.unwrap_or(""),
        out_file.as_ref().display()
    );
    println!("\nConstructed the following FFmpeg args:\n{}\n", args);
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

        println!("\nHLS stream manifest:\n{}\n\n", buf);

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
