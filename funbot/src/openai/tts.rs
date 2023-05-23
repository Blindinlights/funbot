use std::path::{Path, PathBuf};
use anyhow::Result;
use async_openai::{types::CreateTranscriptionRequestArgs, Client};
use std::process::Command;
use uuid::Uuid;
pub fn convert_audio_format(input_path: &PathBuf, output_path: &PathBuf) -> Result<()> {
    let status = Command::new("ffmpeg")
        .arg("-i")
        .arg(input_path)
        .arg(output_path)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("ffmpeg failed"))
    }
}

pub async fn transcribe_audio(file_path: &Path) -> Result<String> {
    let client = Client::new();
    let request = CreateTranscriptionRequestArgs::default()
        .file(file_path)
        .model("wisper-1")
        .prompt("这是一个中国大陆的用户")
        .build()?;
    let res = client.audio().transcribe(request).await?;
    Ok(res.text)
}

pub async fn text_to_speech(text: &str) -> Result<PathBuf> {
    info!("text_to_speech: {}", text);
    let region = std::env::var("SPEECH_REGION").expect("SPEECH_REGION not set");
    let sub_key = std::env::var("SPEECH_KEY").expect("SPEECH_KEY not set");
    let url = format!("https://{region}.tts.speech.microsoft.com/cognitiveservices/v1");
    let ssml = format!(
        r#"<speak version="1.0" xmlns="http://www.w3.org/2001/10/synthesis" xml:lang="zh-CN"><voice name="zh-CN-XiaoxiaoNeural">{}</voice></speak>"#,
        text
    );

    let res = reqwest::Client::new()
        .post(&url)
        .bearer_auth(sub_key)
        .body(ssml)
        .header("User-Agent", "curl")
        .header("Content-Type", "application/ssml+xml")
        .header(
            "X-Microsoft-OutputFormat",
            "audio-16khz-32kbitrate-mono-mp3",
        )
        .send()
        .await?;
    let output = res.bytes().await?;
    let file_name = format!("tts_output_{}.wav", Uuid::new_v4());
    let file_path = Path::new("/tmp").join(file_name);
    std::fs::write(&file_path, output)?;
    Ok(file_path)
}
