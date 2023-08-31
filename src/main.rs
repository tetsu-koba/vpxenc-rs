use libivf_rs as ivf;
use std::error::Error;
use std::fs::File;
use std::io::ErrorKind;
use std::io::Read;
use std::process::ExitCode;
use std::time::Instant;

fn encode(
    input_file: &str,
    output_file: &str,
    width: u32,
    height: u32,
    framerate: u32,
    bitrate: u32,
    keyframe_interval: u32,
) -> Result<(), Box<dyn Error>> {
    let mut yuv_file = File::open(input_file)?;
    let mut yuv = vec![0u8; (width * height * 3 / 2) as _];
    let ivf_header = ivf::IvfHeader {
        signature: *ivf::IVF_SIGNATURE,
        version: 0,
        header_size: 32,
        fourcc: *b"VP80",
        width: width as _,
        height: height as _,
        framerate_num: framerate,
        framerate_den: 1,
        num_frames: 0,
        unused: 0,
    };
    let mut ivf_writer = ivf::IvfWriter::init(File::create(output_file)?, &ivf_header)?;
    _ = keyframe_interval;

    let mut vpx = vpx_encode::Encoder::new(vpx_encode::Config {
        width,
        height,
        timebase: [1, 1000],
        bitrate,
        codec: vpx_encode::VideoCodecId::VP8,
    })
    .unwrap();

    let start = Instant::now();
    let mut frame_count: u32 = 0;
    loop {
        let now = Instant::now();
        let time = now - start;

        match yuv_file.read_exact(&mut yuv) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{e:?}");
                break;
            }
        }
        let ms = time.as_secs() * 1000 + time.subsec_millis() as u64;
        for frame in vpx.encode(ms as i64, &yuv).unwrap() {
            //vt.add_frame(frame.data, frame.pts as u64 * 1_000_000, frame.key);
            ivf_writer.write_ivf_frame(frame.data, frame_count.into())?;
        }
        frame_count += 1;
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 6 {
        eprintln!(
            "Usage: {} input_file output_file width height framerate bitrate keyframe_interval",
            args[0]
        );
        return ExitCode::FAILURE;
    }

    let input_file = &args[1];
    let output_file = &args[2];
    let width: u32 = args[3].parse().expect("Invalid width");
    let height: u32 = args[4].parse().expect("Invalid height");
    let framerate: u32 = args[5].parse().expect("Invalid framerate");
    let bitrate: u32 = args[5].parse().expect("Invalid bitrate");
    let keyframe_interval: u32 = args[5].parse().expect("Invalid keyframe interval");

    if let Err(e) = encode(
        input_file,
        output_file,
        width,
        height,
        framerate,
        bitrate,
        keyframe_interval,
    ) {
        if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
            match io_err.kind() {
                ErrorKind::UnexpectedEof | ErrorKind::BrokenPipe => return ExitCode::SUCCESS,
                _ => {}
            }
        }
        eprintln!("{e:?}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
