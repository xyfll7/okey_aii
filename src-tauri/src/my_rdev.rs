use rdev::{listen, Button, Event, EventType, Key};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::time::Instant;
use tauri::AppHandle;

struct GlobalState<FD: Fn(&AppHandle) + Clone + Send, FC: Fn(&AppHandle, i32, i32)> {
    ime_handler: InputMethodEditorHandler,
    translate_bubble_handler: TranslateBubbleHandler<FD>,
    click_outside_handler: ClickOutsideHandler<FC>,
}

struct InputMethodEditorHandler {
    was_pressed: bool,
    press_start_time: Option<Instant>,
}

impl InputMethodEditorHandler {
    fn new() -> Self {
        Self {
            was_pressed: false,
            press_start_time: None,
        }
    }

    fn handle(&mut self, is_pressed: bool, _app: &AppHandle) {
        if is_pressed && !self.was_pressed {
            self.press_start_time = Some(Instant::now());
        } else if !is_pressed && self.was_pressed {
            self.press_start_time = None;
            // TODO: my_windows::window_input_method_editor_hide(app); 备注：隐藏输入法窗口
        } else if is_pressed && self.was_pressed {
            if let Some(start_time) = self.press_start_time {
                if start_time.elapsed() >= Duration::from_millis(800) {
                    // TODO: my_windows::window_input_method_editor_show(app); 备注：长按显示输入法窗口
                    self.press_start_time = None;
                }
            }
        }

        self.was_pressed = is_pressed;
    }
}

struct TranslateBubbleHandler<FD: Fn(&AppHandle) + Clone + Send> {
    click_count: u32,
    last_release_time: Option<Instant>,
    click_timeout: u128,
    on_double_click: FD,
}

impl<FD: Fn(&AppHandle) + Clone + Send + 'static> TranslateBubbleHandler<FD> {
    fn new(on_double_click: FD) -> Self {
        Self {
            click_count: 0,
            last_release_time: None,
            click_timeout: 400,
            on_double_click,
        }
    }

    fn handle_release(&mut self) {
        let now = Instant::now();

        if let Some(last_release) = self.last_release_time {
            if now.duration_since(last_release).as_millis() < self.click_timeout {
                self.click_count += 1;
            } else {
                self.click_count = 1;
            }
        } else {
            self.click_count = 1;
        }

        self.last_release_time = Some(now);
    }

    // 在时间窗口结束后根据点击次数分发动作（用于区分双击与三连击）
    fn check_timeout(&mut self, app: &AppHandle) {
        if self.click_count == 0 {
            return;
        }

        if let Some(last_release) = self.last_release_time {
            if last_release.elapsed().as_millis() >= self.click_timeout {
                match self.click_count {
                    2 => self.trigger_double_click(app),
                    3 => self.trigger_triple_click(app),
                    _ => {}
                }
                self.click_count = 0;
                self.last_release_time = None;
            }
        }
    }

    fn trigger_double_click(&self, app: &AppHandle) {
        let app_clone = app.clone();
        let callback = self.on_double_click.clone();
        // 在独立线程中执行回调，避免在持有 global_state 锁的状态下
        // 同步调用窗口创建/显示等阻塞操作，导致键盘/鼠标事件处理卡死。
        std::thread::spawn(move || {
            callback(&app_clone);
        });
    }

    fn trigger_triple_click(&self, _app: &AppHandle) {
        // TODO: 三连击的具体逻辑待实现
    }
}

struct ClickOutsideHandler<FC: Fn(&AppHandle, i32, i32)> {
    mouse_x: i32,
    mouse_y: i32,
    on_click_outside: FC,
}

impl<FC: Fn(&AppHandle, i32, i32)> ClickOutsideHandler<FC> {
    fn new(on_click_outside: FC) -> Self {
        Self {
            mouse_x: 0,
            mouse_y: 0,
            on_click_outside,
        }
    }

    fn update_mouse_position(&mut self, x: f64, y: f64) {
        self.mouse_x = x as i32;
        self.mouse_y = y as i32;
    }

    fn handle_click(&mut self, app: &AppHandle) {
        let app_clone = app.clone();
        let (x, y) = (self.mouse_x, self.mouse_y);
        (self.on_click_outside)(&app_clone, x, y);
    }
}

pub fn init_global_input_listener<FD, FC>(app: &AppHandle, on_double_click: FD, on_click_outside: FC)
where
    FD: Fn(&AppHandle) + Clone + Send + 'static,
    FC: Fn(&AppHandle, i32, i32) + Send + 'static,
{
    let app_clone = app.clone();

    let global_state = Arc::new(Mutex::new(GlobalState {
        ime_handler: InputMethodEditorHandler::new(),
        translate_bubble_handler: TranslateBubbleHandler::new(on_double_click),
        click_outside_handler: ClickOutsideHandler::new(on_click_outside),
    }));

    let state_clone = global_state.clone();

    thread::spawn(move || {
        let app = app_clone;

        let callback = move |event: Event| {
            let mut state = state_clone.lock().unwrap();

            match event.event_type {
                EventType::KeyPress(key) => {
                    #[cfg(target_os = "macos")]
                    let is_target_key = matches!(key, Key::MetaRight);
                    #[cfg(not(target_os = "macos"))]
                    let is_target_key = matches!(key, Key::ControlRight);

                    if is_target_key {
                        state.ime_handler.handle(true, &app);
                    }
                }
                EventType::KeyRelease(key) => {
                    #[cfg(target_os = "macos")]
                    let is_target_key = matches!(key, Key::MetaRight);
                    #[cfg(not(target_os = "macos"))]
                    let is_target_key = matches!(key, Key::ControlRight);

                    if is_target_key {
                        state.ime_handler.handle(false, &app);
                        state.translate_bubble_handler.handle_release();
                    }
                }
                EventType::MouseMove { x, y } => {
                    state.click_outside_handler.update_mouse_position(x, y);
                }
                EventType::ButtonPress(Button::Left) => {
                    state.click_outside_handler.handle_click(&app);
                }
                _ => {}
            }
        };

        if let Err(error) = listen(callback) {
            log::error!("rdev Listening error: {:?}", error);
        }
    });

    let state_clone2 = global_state.clone();
    let app_clone2 = app.clone();
    thread::spawn(move || loop {
        {
            let mut state = state_clone2.lock().unwrap();
            if state.ime_handler.was_pressed {
                state.ime_handler.handle(true, &app_clone2);
            }
            state.translate_bubble_handler.check_timeout(&app_clone2);
        }
        thread::sleep(Duration::from_millis(16));
    });
}
