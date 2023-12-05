extern crate ffmpeg_next;

use ffmpeg_next::{codec, encoder, format, media, Dictionary};
use std::path::Path;

fn main() {
    ffmpeg_next::init().unwrap();

    let url = "test2.mp4";

    if !Path::new(url).exists() {
        panic!("File does not exist");
    }

    run(url);
}

fn run(url: &str) {
    let dict = Dictionary::new();

    let context = match format::input_with_dictionary(&url, dict) {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error opening input file: {:?}", e);
            return;
        }
    };

    let stream = match context
        .streams()
        .best(media::Type::Video)
        .ok_or_else(|| ffmpeg_next::Error::StreamNotFound)
    {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error finding best video stream: {:?}", e);
            return;
        }
    };

    let ctx = codec::context::Context::from_parameters(stream.parameters()).unwrap();

    let mut decoder = ctx.decoder().video().unwrap();

    let codec = encoder::find(codec::Id::H264)
        .expect("failed to find encoder")
        .video()
        .unwrap();

    let mut octx = format::output(&"output.mp4").unwrap();

    let global = octx
        .format()
        .flags()
        .contains(format::flag::Flags::GLOBAL_HEADER);

    decoder.set_parameters(stream.parameters()).unwrap();

    let mut output = octx.add_stream(codec).unwrap();

    let ctx = unsafe {
        codec::context::Context::wrap(
            ffmpeg_next::ffi::avcodec_alloc_context3(codec.as_ptr()),
            None,
        )
    };

    let mut encoder = ctx.encoder().video().unwrap();

    if global {
        encoder.set_flags(codec::flag::Flags::GLOBAL_HEADER);
    }

    let pixel_format = decoder.format();

    encoder.set_format(pixel_format);
    encoder.set_height(decoder.height());
    encoder.set_width(decoder.width());

    println!("Input pixel format: {:?}", decoder.format());

    encoder.set_bit_rate(decoder.bit_rate().min(320000));
    encoder.set_max_bit_rate(decoder.max_bit_rate().min(320000));

    encoder.set_time_base((1, 200));
    output.set_time_base((1, 200));

    let encoder = encoder.open_as(codec).unwrap();

    output.set_parameters(&encoder);
}

fn get_video_info(url: &str) {
    let dict = Dictionary::new();

    let context = format::input_with_dictionary(&url, dict).unwrap();

    let stream = context
        .streams()
        .best(media::Type::Video)
        .ok_or_else(|| ffmpeg_next::Error::StreamNotFound)
        .unwrap();

    let codec = codec::context::Context::from_parameters(stream.parameters()).unwrap();

    if let Ok(video) = codec.decoder().video() {
        let mut bitrate = video.bit_rate();

        if bitrate == 0 {
            bitrate = context.bit_rate() as usize;
        }

        let mut frames = stream.frames() as usize;
        if frames == 0 {
            frames = (stream.duration() as f64
                * f64::from(stream.time_base())
                * f64::from(stream.rate())) as usize;
        }

        println!(
            "duration_ms: {}",
            stream.duration() as f64 * f64::from(stream.time_base()) * 1000.0
        );
        println!("frame_count: {}", frames);
        println!("fps: {}", f64::from(stream.rate()));
        println!("bitrate: {}", bitrate as f64 / 1024.0 / 1024.0);
        println!("video_width: {}", video.width());
    }
}
