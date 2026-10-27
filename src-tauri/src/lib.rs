use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Serialize, Deserialize)]
struct IOBridgeRequest {
  target: serde_json::Value,
  method: String,
  args: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SafeAreaInsets {
  top: f64,
  bottom: f64,
  left: f64,
  right: f64,
}

/// 探测实例的超时：比浏览器默认等待更长，慢速网络下也能给出明确结论
const PROBE_TIMEOUT: Duration = Duration::from_secs(9);
const PROBE_CONNECT_TIMEOUT: Duration = Duration::from_secs(6);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProbeResult {
  ok: bool,
  status: u16,
  version: Option<String>,
  distribution: Option<String>,
  degraded: bool,
  latency_ms: u64,
  error: Option<String>,
}

impl ProbeResult {
  fn failure(message: impl Into<String>, latency_ms: u64) -> Self {
    Self {
      ok: false,
      status: 0,
      version: None,
      distribution: None,
      degraded: false,
      latency_ms,
      error: Some(message.into()),
    }
  }
}

/// 通用桥接命令：按 method 分发到不同逻辑
#[tauri::command]
fn io_bridge_call(request: IOBridgeRequest) -> Result<serde_json::Value, String> {
  let method = request.method;
  let _args = request.args.unwrap_or_default();

  log::info!("IO Bridge call: {}", method);

  match method.as_str() {
    "getPlatform" => {
      #[cfg(target_os = "android")]
      return Ok(serde_json::json!({ "platform": "android" }));
      #[cfg(target_os = "ios")]
      return Ok(serde_json::json!({ "platform": "ios" }));
      #[cfg(not(any(target_os = "android", target_os = "ios")))]
      return Ok(serde_json::json!({ "platform": "desktop" }));
    }
    "getAppVersion" => {
      Ok(serde_json::json!({ "version": env!("CARGO_PKG_VERSION") }))
    }
    _ => {
      log::warn!("IO Bridge: unknown method {}", method);
      Ok(serde_json::json!({
        "result": "ok",
        "method": method,
        "args": _args,
        "note": "unimplemented method"
      }))
    }
  }
}

/// 获取状态栏高度（Android 通过 CSS env(safe-area-inset-top) 更可靠）
#[tauri::command]
fn get_status_bar_height() -> f64 {
  0.0
}

/// 获取安全区域插值
#[tauri::command]
fn get_safe_area_insets() -> Result<SafeAreaInsets, String> {
  Ok(SafeAreaInsets {
    top: 0.0,
    bottom: 0.0,
    left: 0.0,
    right: 0.0,
  })
}

/// 探测实例：依次请求 `{url}/api/health` 与 `{url}/health`，返回状态码、版本号与失败原因。
///
/// 放在 Rust 侧而不是页面里 fetch，是因为起始页与实例不同源，而实例默认不启用 CORS：
/// 浏览器请求只会在控制台留下跨域错误，拿不到真实状态，也分不清「连不上」和「路径不对」。
///
/// 两个候选路径是两种常见的部署形态：站点由前端容器代理时健康检查在 /api/health，
/// 后端直接托管静态文件时在 /health。只认 JSON 里的 status/version，避免把 SPA 的
/// 兜底 HTML（200）误判成连接成功。
#[tauri::command]
async fn probe_instance(url: String) -> ProbeResult {
  let started = Instant::now();
  let base = url.trim().trim_end_matches('/');
  if !(base.starts_with("http://") || base.starts_with("https://")) {
    return ProbeResult::failure("实例地址必须以 http:// 或 https:// 开头", 0);
  }

  let client = match reqwest::Client::builder()
    .timeout(PROBE_TIMEOUT)
    .connect_timeout(PROBE_CONNECT_TIMEOUT)
    .user_agent(concat!("AIsChat-Client/", env!("CARGO_PKG_VERSION")))
    .build()
  {
    Ok(client) => client,
    Err(err) => return ProbeResult::failure(format!("初始化 HTTP 客户端失败：{err}"), 0),
  };

  let mut last_status = 0u16;
  for path in ["/api/health", "/health"] {
    match client.get(format!("{base}{path}")).send().await {
      Ok(response) => {
        let status = response.status();
        last_status = status.as_u16();
        let body = response.json::<serde_json::Value>().await.ok();
        let field = |key: &str| {
          body.as_ref()
            .and_then(|value| value.get(key))
            .and_then(|value| value.as_str())
            .map(str::to_owned)
        };
        let version = field("version");
        let state = field("status");
        if version.is_none() && state.is_none() {
          // 该地址返回的是网页而不是健康检查，换下一个候选
          continue;
        }
        return ProbeResult {
          ok: status.is_success(),
          status: status.as_u16(),
          version,
          distribution: field("distribution"),
          degraded: state.as_deref() == Some("degraded"),
          latency_ms: started.elapsed().as_millis() as u64,
          error: None,
        };
      }
      Err(err) => {
        let reason = if err.is_timeout() {
          format!("请求超时（{} 秒）", PROBE_TIMEOUT.as_secs())
        } else if err.is_connect() {
          "无法建立连接：域名解析失败、端口未开放或被网络阻断".to_string()
        } else if err.is_request() {
          format!("请求无法发送：{err}")
        } else {
          err.to_string()
        };
        return ProbeResult::failure(reason, started.elapsed().as_millis() as u64);
      }
    }
  }

  // 两个候选都返回了网页：说明地址可达，但不等同于一个 AIsChat 实例
  let message = if last_status == 0 {
    "该地址没有响应健康检查".to_string()
  } else {
    format!("{base} 可达（HTTP {last_status}），但没有返回 AIsChat 的健康检查数据")
  };
  ProbeResult::failure(message, started.elapsed().as_millis() as u64)
}

/// 移动端注入脚本：只补 viewport-fit，不改动页面自身的缩放策略。
///
/// 网页里没写 viewport-fit=cover 时，env(safe-area-inset-*) 恒为 0，
/// 主应用依赖这几个变量给底部导航留出系统手势条的位置。
const VIEWPORT_SCRIPT: &str = r#"(function(){
function apply(){
  var meta=document.querySelector('meta[name=viewport]');
  if(!meta){
    meta=document.createElement('meta');
    meta.name='viewport';
    meta.content='width=device-width, initial-scale=1.0, viewport-fit=cover';
    (document.head||document.documentElement).appendChild(meta);
    return;
  }
  if(meta.content.indexOf('viewport-fit')===-1){
    meta.content=meta.content.replace(/s*$/,'')+', viewport-fit=cover';
  }
}
if(document.head){apply();}else{document.addEventListener('DOMContentLoaded',apply);}
})();"#;

// Tauri 2.0: 移动端入口点
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      log::info!("AIsChat Client starting...");
      let _window = tauri::WebviewWindowBuilder::new(
        app,
        "main",
        tauri::WebviewUrl::App("index.html".into()),
      )
      .title("AIsChat")
      .inner_size(1200.0, 800.0)
      .min_inner_size(800.0, 600.0)
      .resizable(true)
      .initialization_script(VIEWPORT_SCRIPT)
      .on_navigation(|url| {
        // 实例地址由用户填写，可能是任意域名或局域网 IP，只按协议放行
        let allowed = url.scheme() == "https"
          || url.scheme() == "http"
          || url.scheme() == "tauri";
        if !allowed {
          log::warn!("Blocked navigation to: {}", url);
        }
        allowed
      })
      .build()?;
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      io_bridge_call,
      get_status_bar_height,
      get_safe_area_insets,
      probe_instance,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}