use chrono::prelude::*;
use execute::Execute;
use image::Luma;
use qrcode::QrCode;
use s3::{creds::Credentials, Bucket, BucketConfiguration, Region};
use std::process::Command;
use std::{env, error::Error};

const ENDPOINT_URL: &str = "ENDPOINT_URL";
const BUCKET_NAME: &str = "BUCKET_NAME";
const ACCESS_KEY_ID: &str = "ACCESS_KEY_ID";
const SECRET_ACCESS_KEY: &str = "SECRET_ACCESS_KEY";
const REGION: &str = "REGION";
const SOURCE_PATH: &str = "SOURCE_PATH";
const TARGET_PATH: &str = "TARGET_PATH";
const FFMPEG_PATH: &str = "FFMPEG_PATH";
const CDN_URL: &str = "CDN_URL";
const MP4_COMPRESSED_FOLDER_NAME: &str = "MP4_COMPRESSED_FOLDER_NAME";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let local: DateTime<Local> = Local::now();
    mp4_compressed();
    upload_mp4(local).await?;
    generate_qrcode(local);
    Ok(())
}

fn mp4_compressed() {
    println!(
        "{}",
        console::style("==========压缩mp4大小==========")
            .blue()
            .bright()
    );
    let source_path = env::var(SOURCE_PATH).expect("missing source path");
    let target_path = env::var(TARGET_PATH).expect("missing target path");
    let mp4_compressed_folder_name =
        env::var(MP4_COMPRESSED_FOLDER_NAME).expect("missing mp4_compressed_folder_name");
    if let Err(err) =
        std::fs::create_dir_all(target_path + "/" + mp4_compressed_folder_name.as_str())
    {
        eprintln!("无法创建目录: {}", err);
    } else {
        println!("目录创建成功！");
    }
    let mut id = 0;
    let entries = std::fs::read_dir(source_path.clone()).unwrap();
    for entry in entries {
        if let Ok(entry) = entry {
            if let Some(file_name) = entry.file_name().to_str() {
                id += 1;
                println!(
                    "{}{}{} {} 开始压缩...",
                    console::style("Task[").blue(),
                    console::style(id).green().bright(),
                    console::style("]:").blue(),
                    file_name
                );
                let ffmpeg = env::var(FFMPEG_PATH).expect("missing ffmpeg");
                let mut compress_command = Command::new(ffmpeg);
                let source_path = env::var(SOURCE_PATH).expect("missing source path");
                let target_path = env::var(TARGET_PATH).expect("missing target path");
                let mp4_compressed_folder_name = env::var(MP4_COMPRESSED_FOLDER_NAME)
                    .expect("missing mp4_compressed_folder_name");
                compress_command.arg("-i");
                compress_command.arg(source_path + "/" + file_name);
                compress_command.arg("-codec:v");
                compress_command.arg("libx264");
                compress_command.arg("-crf");
                compress_command.arg("23");
                compress_command.arg("-preset");
                compress_command.arg("medium");
                compress_command.arg("-codec:a");
                compress_command.arg("aac");
                compress_command.arg("-b:a");
                compress_command.arg("128k");
                compress_command.arg("-stats");
                compress_command.arg("-hide_banner");
                compress_command.arg("-y");
                compress_command
                    .arg(target_path + "/" + mp4_compressed_folder_name.as_str() + "/" + file_name);
                if compress_command.execute_check_exit_status_code(0).is_err() {
                    eprintln!(
                        "\t{} {} {}",
                        console::style("-->").white(),
                        console::style(file_name).yellow(),
                        console::style("压缩失败").red()
                    );
                } else {
                    println!(
                        "\t{} {} {}",
                        console::style("-->").white().bright(),
                        console::style(file_name).yellow(),
                        console::style("压缩成功").green()
                    );
                }
            }
        }
    }
}

async fn upload_mp4(local: DateTime<Local>) -> Result<(), Box<dyn Error>> {
    println!(
        "{}",
        console::style("==========上传mp4至对象存储服务==========")
            .blue()
            .bright()
    );
    let endpoiont = env::var(ENDPOINT_URL).expect("missing endpoint");
    let bucket_name = env::var(BUCKET_NAME).expect("missing bucket name");
    let region = env::var(REGION).expect("missing region");
    let access_key = env::var(ACCESS_KEY_ID).expect("mising access key");
    let secret_key = env::var(SECRET_ACCESS_KEY).expect("missing secret key");
    let bucket: Bucket = create_bucket(
        endpoiont.as_str(),
        bucket_name.as_str(),
        region.as_str(),
        access_key.as_str(),
        secret_key.as_str(),
    )
    .await?;
    let year = format!("{}", local.year());
    let day = format!("{:02}{:02}", local.month(), local.day());
    let target_path = env::var(TARGET_PATH).expect("missing target path");
    let mp4_compressed_folder_name =
        env::var(MP4_COMPRESSED_FOLDER_NAME).expect("missing mp4 compressed folder name");
    let mut entries =
        tokio::fs::read_dir(target_path.clone() + "/" + mp4_compressed_folder_name.as_str())
            .await?;
    let mut id = 0;
    while let Some(entry) = entries.next_entry().await? {
        id += 1;
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();
        println!(
            "{}{}{} {} 开始上传...",
            console::style("Task[").blue(),
            console::style(id).green().bright(),
            console::style("]:").blue(),
            file_name_str
        );
        let file = tokio::fs::read(
            target_path.clone()
                + "/"
                + mp4_compressed_folder_name.as_str()
                + "/"
                + file_name_str.to_string().as_str(),
        )
        .await?;
        let remote_path =
            year.clone() + "/" + day.as_str() + "/" + file_name_str.to_string().as_str();
        println!(
            "\t{} 远程目标路径: {}",
            console::style("-->").white().bright(),
            console::style(remote_path.clone()).yellow()
        );
        let (_, code) = bucket
            .put_object_with_content_type(remote_path, &file, "video/mp4")
            .await?;
        println!(
            "\t{} 上传状态code: {}",
            console::style("-->").white().bright(),
            console::style(code).green()
        );
    }
    Ok(())
}

fn generate_qrcode(local: DateTime<Local>) {
    println!(
        "{}",
        console::style("==========生成二维码==========")
            .blue()
            .bright()
    );
    let year = format!("{}", local.year());
    let day = format!("{:02}{:02}", local.month(), local.day());
    let source_path = env::var(SOURCE_PATH).expect("missing source path");
    let target_path = env::var(TARGET_PATH).expect("missing target path");
    let qrcode_folder_name = "qrcode";
    if let Err(err) = std::fs::create_dir_all(target_path.clone() + "/" + qrcode_folder_name) {
        eprintln!("无法创建目录: {}", err);
    } else {
        println!("qrcode 目录创建成功！");
    }
    let mut id = 0;
    let entries = std::fs::read_dir(source_path.clone()).unwrap();
    for entry in entries {
        if let Ok(entry) = entry {
            if let Some(file_name) = entry.file_name().to_str() {
                id += 1;
                println!(
                    "{}{}{} {} 生成二维码...",
                    console::style("Task[").blue(),
                    console::style(id).green().bright(),
                    console::style("]:").blue(),
                    file_name
                );
                let cdn_url = env::var(CDN_URL).expect("missing cdn url");
                let bucket_name = env::var(BUCKET_NAME).expect("missing bucket name");
                let qrcode_content = cdn_url
                    + "/"
                    + bucket_name.as_str()
                    + "/"
                    + year.as_str()
                    + "/"
                    + day.as_str()
                    + "/"
                    + file_name;
                let code = QrCode::new(qrcode_content).unwrap();
                let image = code.render::<Luma<u8>>().build();
                image
                    .save(
                        target_path.clone()
                            + "/"
                            + qrcode_folder_name
                            + "/"
                            + &file_name[..file_name.len() - 4]
                            + ".png",
                    )
                    .unwrap();
                println!(
                    "\t{} {} 二维码生成完毕",
                    console::style("-->").white().bright(),
                    console::style(
                        target_path.clone()
                            + "/"
                            + qrcode_folder_name
                            + "/"
                            + &file_name[..file_name.len() - 4]
                            + ".png"
                    )
                    .yellow()
                );
            }
        }
    }
}

async fn create_bucket(
    endpoint: &str,
    bucket_name: &str,
    region: &str,
    access_key: &str,
    secret_key: &str,
) -> Result<Bucket, Box<dyn Error>> {
    let bucket = Bucket::new_with_path_style(
        bucket_name,
        Region::Custom {
            region: region.to_string(),
            endpoint: endpoint.to_string(),
        },
        Credentials::new(Some(access_key), Some(secret_key), None, None, None)?,
    )?;
    // println!("{:?}", bucket);
    let (_, code) = bucket.head_object("/").await?;
    println!("code: {}", code);
    if code == 404 {
        let create_result = Bucket::create_with_path_style(
            bucket.name.as_str(),
            bucket.region.clone(),
            bucket.credentials.clone(),
            BucketConfiguration::default(),
        )
        .await?;
        println!(
            "res: {} - {} - {}",
            bucket_name, create_result.response_code, create_result.response_text
        )
    }
    Ok(bucket)
}
