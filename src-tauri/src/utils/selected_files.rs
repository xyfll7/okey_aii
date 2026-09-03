#[cfg(target_os = "macos")]
use std::process::Command;

/// 哨兵值：Finder 不在前台时 AppleScript 返回它，Rust 侧据此过滤为空。
/// 用同一常量拼进脚本，避免字面量在两处维护发生漂移。
#[cfg(target_os = "macos")]
const NOT_FRONTMOST: &str = "__OKEY_NOT_FRONTMOST__";

/// 获取 Finder（含桌面图标）当前选中的文件 / 文件夹路径列表。
///
/// macOS 没有公开 API 读取 Finder 的选中项，这里通过 AppleScript
/// (osascript) 控制 Finder 读取其 `selection` 属性。
/// 首次调用时系统会弹出「自动化」授权：允许本 App 控制 Finder。
///
/// 注意：
/// - 只对 Finder 窗口 / 桌面中选中的文件有效，其他 App 的“选中”仍是文本；
/// - Finder 的 `selection` 不会因为切到别的 App 而清空。为避免在其它 App 里
///   选中文本时读到 Finder 残留的旧文件选择，这里把「判断 Finder 是否前台」
///   和「读取选中项」合并进同一次 osascript 调用：`frontmost` 是 Finder 自带
///   的 Application 属性，判断与读取原子完成，无竞态，也不依赖 `lsappinfo`
///   这类未公开工具，更不需要 System Events 的辅助功能权限。
#[cfg(target_os = "macos")]
pub fn get_selected_file_paths() -> Vec<String> {
    let script = format!(
        r#"
        tell application "Finder"
            if not frontmost then return "{NOT_FRONTMOST}"
            with timeout of 2 seconds
                set theSelection to selection
                set output to ""
                repeat with itemRef in theSelection
                    set output to output & POSIX path of (itemRef as alias) & linefeed
                end repeat
                return output
            end timeout
        end tell
        "#
    );

    match run_osascript(&script) {
        Some(paths) => paths
            .into_iter()
            .filter(|line| line != NOT_FRONTMOST)
            .collect(),
        // 脚本执行失败（如未授权自动化）时按“没有文件选中”处理
        None => Vec::new(),
    }
}

#[cfg(not(target_os = "macos"))]
pub fn get_selected_file_paths() -> Vec<String> {
    Vec::new()
}

#[cfg(target_os = "macos")]
fn run_osascript(script: &str) -> Option<Vec<String>> {
    match Command::new("osascript").arg("-e").arg(script).output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let paths: Vec<String> = stdout
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect();
            Some(paths)
        }
        Ok(output) => {
            // -1743：自动化权限被拒绝 / 用户未授权控制 Finder。
            // 这是预期内且可恢复的情况，用 warn 级别提示并给出引导，而不是当普通错误。
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("-1743") {
                log::warn!(
                    "读取 Finder 选中文件失败（可能未授权自动化）：{stderr} \
                     可在 系统设置 → 隐私与安全性 → 自动化 中允许本 App 控制 Finder"
                );
            } else {
                log::warn!("osascript failed: {stderr}");
            }
            None
        }
        Err(err) => {
            log::warn!("failed to run osascript: {err}");
            None
        }
    }
}
