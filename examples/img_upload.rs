use std::fs::copy;
use reqwest::header::CONTENT_TYPE;
use reqwest::blocking::multipart;
use reqwest::StatusCode;
use serde::Deserialize;

// const IMG_CDN_JSON_UPLOAD: &str = "https://cdnjson.com/api/1/upload";
const IMG_CDN_JSON_UPLOAD: &str = "https://www.picgo.net/api/1/upload";
const API_KEY: &str = "chv_1F6W_b949fd23c8b1af8f3bf366c0c486b18056ecc07f315276c2dadf1a0d02bf458645708fce502bf04e15855fc1599ad009b178e72580a96621dd848af39990e0f6";


#[derive(Deserialize)]
struct UploadResponse {
    status_code: u32,
    status_txt: String,
    image: Option<ImageData>,
}

#[derive(Deserialize)]
struct ImageData {
    url: String,
    size: u32,
}


fn main() {
    let client = reqwest::blocking::Client::new();
    let original_image_path = "./test_data/1.jpg";
    let new_filename = "temp.jpg";
    // 创建临时文件的路径
    let temp_file_path = format!("./test_data/{}", new_filename);
    copy(original_image_path, &temp_file_path).expect("Failed to copy image to temporary file");

    let form = reqwest::blocking::multipart::Form::new()
        .text("key", API_KEY)
        .file("source", &temp_file_path).unwrap();
    let res = client.post(IMG_CDN_JSON_UPLOAD)
        .multipart(form)
        .send()
        .expect("Failed to send request");

    let upload_result: UploadResponse = res.json().expect("Failed to parse response JSON");
    if upload_result.status_code == StatusCode::OK.as_u16() as u32 {
        if let Some(image) = upload_result.image {
            println!("Image uploaded successfully. URL: {}", image.url);
        } else {
            println!("Image upload successful, but no image data returned.");
        }
    } else {
        println!("Image upload failed. Error: {}", upload_result.status_txt);
    }
    std::fs::remove_file(&temp_file_path).expect("Failed to remove temporary file");
}