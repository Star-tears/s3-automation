<div align="center">
<h1 style="border-bottom: none">s3-automation<br></h1>
<p>s3对象云存储服务自动化<br /></p>
</div>

---

<div align="center">
<em>mp4视频自动压缩，文件上传，链接二维码生成</em>
</div>

## 简介

一键mp4视频压缩，文件上传，链接二维码生成。可用于支持S3（Simple Storage Service）对象存储云服务，如可自部署的minio等云存储。

由于涉及mp4视频压缩，需要配置好`ffmpeg`环境，可通过命令`ffmpeg -version`检查ffmpeg是否已配置好。

## 使用说明

若想直接调试运行，可在`.cargo`目录下新建`config.toml`配置相应环境变量再通过命令`cargo run`运行

```toml
[env]
ENDPOINT_URL = "https://minio-api.example.com"
BUCKET_NAME = "your-bucket-name"
ACCESS_KEY_ID = "xxxxxxxxxxxxxxxx"
SECRET_ACCESS_KEY = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
REGION = "us-west-rack2"
SOURCE_PATH = "source_folder"
TARGET_PATH = "target_folder"
FFMPEG_PATH = "ffmpeg"
CDN_URL = "https://oss.example.com"
MP4_COMPRESSED_FOLDER_NAME = "mp4_compressed"
```



### 环境变量

| 变量名                     | 作用                                                        |
| -------------------------- | ----------------------------------------------------------- |
| ENDPOINT_URL               | 对象云存储api入口url                                        |
| BUCKET_NAME                | 存储桶名                                                    |
| ACCESS_KEY_ID              | 密钥ID                                                      |
| SECRET_ACCESS_KEY          | 密钥                                                        |
| REGION                     | 区域                                                        |
| SOURCE_PATH                | 原文件目录路径，将需要上传的文件或文件夹放到该目录          |
| TARGET_PATH                | 目标目录路径，存放程序生成的文件，如链接二维码和压缩后的mp4 |
| FFMPEG_PATH                | ffmpeg路径                                                  |
| CDN_URL                    | cdn的url（用于文件链接生成最终对应二维码）                  |
| MP4_COMPRESSED_FOLDER_NAME | mp4压缩后存放的文件夹名（该文件夹在TARGET_PATH目录下）      |

