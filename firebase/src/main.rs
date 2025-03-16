use reqwest::Client;
use std::fs;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // 讀取要上傳的圖片
    let image_path = "image.jpg";
    let image_data = fs::read(image_path)?;

    // Firebase Storage 上傳 URL，替換為你的 bucket 和檔案名稱
    let firebase_storage_url = "https://firebasestorage.googleapis.com/v0/b/rust-law-web-frdata.appspot.com/o?name=images/image.jpg";

    // 設置身份驗證 Token
    let firebase_token = "YOUR_FIREBASE_TOKEN";  // 需替換成你的 Firebase Token

    // 發送上傳請求
    let response = client
        .post(firebase_storage_url)
        .header("Content-Type", "image/jpeg")
        .body(image_data)
        .send()
        .await?;

    if response.status().is_success() {
        let response_text = response.text().await?;
        println!("圖片上傳成功！回應內容: {}", response_text);
    } else {
        println!("圖片上傳失敗: {}", response.status());
    }

    Ok(())
}

