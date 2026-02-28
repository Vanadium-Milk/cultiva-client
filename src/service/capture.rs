use image::imageops::FilterType;
use image::{ImageFormat, ImageReader};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::RequestedFormatType::AbsoluteHighestResolution;
use nokhwa::utils::{CameraIndex, RequestedFormat};
use nokhwa::{Camera, NokhwaError};
use rust_socketio::{Payload, RawClient};
use serde_json::json;
use std::error::Error;
use std::fs::{read, read_dir};
use std::io::Cursor;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

fn get_cam() -> Result<Camera, NokhwaError> {
    // first camera in system
    let index = CameraIndex::Index(0);
    // request the absolute highest resolution CameraFormat that can be decoded to RGB.
    let requested = RequestedFormat::new::<RgbFormat>(AbsoluteHighestResolution);
    // make the camera
    let mut camera = Camera::new(index, requested)?;

    camera.open_stream()?;

    //Force camera to initiate stream and wait two seconds so the first photo isn't messy
    //This is a hacky solution to support some cameras but works perfectly
    camera.frame()?;
    sleep(Duration::from_secs(2));

    Ok(camera)
}

fn save_frame() -> Result<(), Box<dyn Error>> {
    let frame = get_cam()?.frame()?;

    //Resize the frame into a more portable size
    let src_image = ImageReader::new(Cursor::new(frame.buffer()))
        .with_guessed_format()?
        .decode()?;
    let resized = src_image.resize(1024, 576, FilterType::Lanczos3);

    let img_name = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    resized.save_with_format(
        format!("/var/lib/cultiva/captures/{}.jpg", img_name.as_secs()),
        ImageFormat::Jpeg,
    )?;

    println!("{}", t!("capture.success"));
    Ok(())
}

pub(super) fn poll_cam() {
    loop {
        if let Err(e) = save_frame() {
            eprintln!("{}. {}", t!("capture.failed", error = e), t!("retry"));
            //Implement cleanup here (Not right now, I need the photos for the presentation)
            sleep(Duration::from_mins(1))
        } else {
            sleep(Duration::from_hours(3));
        }
    }
}

pub(super) fn send_frame(payload: Payload, client: RawClient) {
    //Save frame when image is requested
    if let Err(e) = save_frame() {
        eprintln!("{}. {}", t!("capture.failed", error = e), t!("retry"));
    }

    if let Payload::Text(text) = &payload
        && !text.is_empty()
        && let Some(response_id) = text[0].as_str()
    {
        let paths = read_dir("/var/lib/cultiva/captures/").unwrap();
        let last = paths.last().unwrap().unwrap().path();
        dbg!(&last);
        let img = read(last).unwrap();

        let send = json!({
        "id": response_id,
        "data": {
            "buffer": img
        }});
        client.emit("response", send).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::service::capture::{poll_cam, save_frame};
    use nokhwa::query;
    use nokhwa::utils::ApiBackend::Auto;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn query_cams() {
        dbg!(query(Auto).unwrap());
    }
    #[test]
    fn take_photo() {
        save_frame().unwrap();
    }
    #[test]
    fn test_polling() {
        loop {
            poll_cam();
            println!("Photo taken!");
            sleep(Duration::from_secs(5));
        }
    }
}
