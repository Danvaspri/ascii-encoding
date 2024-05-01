extern crate ffmpeg_next as ffmpeg;

use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg::util::frame::video::Video;
use std::env;

fn main() -> Result<(), ffmpeg::Error> {
    ffmpeg::init().unwrap();
    if let Ok(mut ictx) = input(&env::args().nth(1).expect("Cannot open file.")) {
        let input = ictx
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg::Error::StreamNotFound)?;
        let video_stream_index = input.index();
        let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;
        let mut decoder = context_decoder.decoder().video()?;
        let mut scaler = Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::GRAY8,
            256,
            144,
            Flags::BILINEAR,
        )?;
        let mut frame_index = 0;
        let mut receive_and_process_decoded_frames =
            |decoder: &mut ffmpeg::decoder::Video| -> Result<(), ffmpeg::Error> {
                let mut decoded = Video::empty();
                while decoder.receive_frame(&mut decoded).is_ok() {
                    let mut rgb_frame = Video::empty();
                    scaler.run(&decoded, &mut rgb_frame)?;
                    render_frame(&rgb_frame, frame_index).unwrap();
                    frame_index += 1;
                }
                Ok(())
            };
        for (stream, packet) in ictx.packets() {
            if stream.index() == video_stream_index {
                decoder.send_packet(&packet)?;
                receive_and_process_decoded_frames(&mut decoder)?;
            }
        }
        decoder.send_eof()?;
        receive_and_process_decoded_frames(&mut decoder)?;
    }
    Ok(())
}

fn render_frame(frame: &Video, index: usize) -> std::result::Result<(), std::io::Error> {
    let to_ascii:Vec<String> =  frame.data(0).iter().map(|each| brightness_to_ascii(*each).to_string() ).collect();
    let mut str  =String::new();
    for chunk in to_ascii.chunks(256) {
        str.push_str(chunk.join(" ").as_str());
        str.push_str("     EOF \n");
    }
    println!("{}",str);
    print!("FRAME NUMBER {}", index);  
    Ok(())
}
fn brightness_to_ascii(brightness: u8) -> char{ 
    let ascii_chars = " .:+=*#@";
    let bin_size = 256.0 / 8.0;  
    let index = (brightness as f32 / bin_size).floor() as usize;
    ascii_chars.chars().nth(index).unwrap_or(' ')
}