// use std::fs;
// use std::path::PathBuf;
// use lazy_static::lazy_static;
// use serde::{Deserialize, Serialize};
//
// const IMG_CDN_JSON_UPLOAD: &str = "https://www.picgo.net/api/1/upload";
// const API_KEY: &str = "chv_1F6W_b949fd23c8b1af8f3bf366c0c486b18056ecc07f315276c2dadf1a0d02bf458645708fce502bf04e15855fc1599ad009b178e72580a96621dd848af39990e0f6";
// const PROGRAM_NAME: &str = "pic_uploader";
// const LOGS_DIR: &str = "logs";
// const IMG_RECORDED_DIR: &str = "img_recorded";
//
// fn get_app_path() -> Option<PathBuf> {
//     let user_data_dir = match dirs::data_dir() {
//         Some(path) => path,
//         None => {
//             println!("无法获取用户数据目录");
//             return None;
//         }
//     };
//     Some(user_data_dir.join(PROGRAM_NAME))
// }
//
// fn get_log_file_path() -> Option<PathBuf> {
//     let app_data_dir = get_app_path().unwrap();
//     let logs_dir = app_data_dir.join(LOGS_DIR);
//     if !logs_dir.exists() {
//         if let Err(err) = fs::create_dir_all(&logs_dir) {
//             println!("无法创建日志目录: {}", err);
//             return None;
//         }
//     }
//     Some(logs_dir.join(format!("{}.log", PROGRAM_NAME)))
// }
//
// fn get_img_recorded_path() -> Option<PathBuf> {
//     let app_data_dir = get_app_path().unwrap();
//     let img_recorded_path = app_data_dir.join(IMG_RECORDED_DIR);
//     if !img_recorded_path.exists() {
//         if let Err(err) = fs::create_dir_all(&img_recorded_path) {
//             println!("无法创建零时文件目录: {}", err);
//             return None;
//         }
//     }
//     Some(img_recorded_path)
// }
//
// #[derive(Serialize)]
// pub struct UploadPara {
//     // 本地路径
//     img_selected_path: PathBuf,
//     // 上传成功后移到的路径
//     img_uploaded_path: PathBuf,
//     // 上传返回的网络地址
//     img_url: String,
// }
//
// #[derive(Deserialize)]
// struct UploadResponse {
//     status_code: u32,
//     status_txt: String,
//     image: Option<ImageData>,
// }
//
// #[derive(Deserialize)]
// struct ImageData {
//     url: String,
//     size: u32,
// }
//
// impl UploadPara {
//     pub fn new() -> UploadPara {
//         UploadPara {
//             img_selected_path: Default::default(),
//             img_uploaded_path: Default::default(),
//             img_url: "".to_string(),
//         }
//     }
//
//     fn move_img(&mut self){
//
//     }
//     pub fn upload(&self) {
//         // 把图片上传到已上传的路径
//         let temp_file_path = PathBuf::from(&args[1]);
//         // let temp_file_path = temp_file_path.to_string_lossy().to_string();
//
//         let form = reqwest::blocking::multipart::Form::new()
//             .text("key", API_KEY)
//             // .file("source", &temp_file_path).unwrap();
//             .file("source", "test_data/image-20230804112820402.png").unwrap();
//         let res = client.post(IMG_CDN_JSON_UPLOAD)
//             .multipart(form)
//             .send()
//             .expect("Failed to send request");
//
//         let upload_result: UploadResponse = res.json().expect("Failed to parse response JSON");
//         if upload_result.status_code == StatusCode::OK.as_u16() as u32 {
//             if let Some(image) = upload_result.image {
//                 println!("Image uploaded successfully. URL: {}", image.url);
//             } else {
//                 println!("Image upload successful, but no image data returned.");
//             }
//         } else {
//             println!("Image upload failed. Error: {}", upload_result.status_txt);
//         }
//         // std::fs::remove_file(&temp_file_path).expect("Failed to remove temporary file");
//     }
// }
