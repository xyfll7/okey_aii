ROOT_DIR := $(shell pwd)
ADMIN_DIR := $(ROOT_DIR)

.PHONY: tab icon test_stream_ask

## 在现有 Terminal 窗口中新建一个 tab 标签，并 cd 到 youshu-admin 目录
tab:
	@osascript \
	    -e 'tell application "Terminal"' \
	    -e 'activate' \
	    -e 'tell application "System Events" to keystroke "t" using command down' \
	    -e 'delay 0.5' \
	    -e 'do script "cd $(ADMIN_DIR) && pnpm start " in (selected tab of front window)' \
	    -e 'end tell'

icon:
	pnpm tauri icon  # ./app-icon.png

test_stream_ask:
	cd src-tauri && cargo test -- test_stream_ask --nocapture