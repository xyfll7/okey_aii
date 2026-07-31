icon:
	pnpm tauri icon  # ./app-icon.png

test_stream_ask:
	cd src-tauri && cargo test -- test_stream_ask --nocapture