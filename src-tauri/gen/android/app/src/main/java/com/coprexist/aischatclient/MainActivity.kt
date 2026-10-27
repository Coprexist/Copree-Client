package com.coprexist.aischatclient

import android.os.Bundle
import android.webkit.WebView
import androidx.activity.OnBackPressedCallback
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
  private var webView: WebView? = null

  // 启动页由 App 自己托管在这个源的根路径，实例页则是任意外部地址。
  private val pickerUrl = "http://tauri.localhost/"

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
  }

  /**
   * 返回键固定「实例页 → 启动页」，再按一次才退出。
   *
   * 不依赖 WebView.canGoBack()：实例页是从本地页跨源跳转过去的，实测在真机上
   * canGoBack() 返回 false，WryActivity 于是走退出分支直接结束 Activity，用户就
   * 再也回不到选择实例的入口。这里改用当前 URL 判断，与历史状态无关。
   */
  override fun onWebViewCreate(webView: WebView) {
    this.webView = webView
    // 比 WryActivity 的回调注册得晚，因此分发时先执行（后注册先执行）
    onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
      override fun handleOnBackPressed() {
        val current = this@MainActivity.webView?.url
        if (!current.isNullOrEmpty() && !current.startsWith(pickerUrl)) {
          this@MainActivity.webView?.loadUrl(pickerUrl)
          return
        }
        // 已经在启动页：退到后台，与微信/QQ 在根页按返回的一致。
        // 不用 finish()：Activity 结束时 Tauri 会直接结束进程，而 Chromium 的
        // Chrome_InProcGp 线程还没收尾，每次退出都会留下一条 SIGABRT。
        moveTaskToBack(true)
      }
    })
  }
}
