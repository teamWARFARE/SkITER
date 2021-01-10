use std::fs::File;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use sciter::host::{OUTPUT_SEVERITY, OUTPUT_SUBSYTEMS};
use sciter::types::{LOAD_RESULT, POINT, SCN_INVALIDATE_RECT, SCN_LOAD_DATA, _HWINDOW};
use sciter::windowless::{
    handle_message, Message, MouseEvent, PaintLayer, KEYBOARD_STATES, MOUSE_BUTTONS, MOUSE_EVENTS,
};
use sciter::Host;
use sciter::{dispatch_script_call, GFX_LAYER};

use anyhow::anyhow;

type SciterHwnd = *mut _HWINDOW;

struct SciterHandle {
    hwnd: SciterHwnd,
}

impl SciterHandle {
    fn raw(&mut self) -> SciterHwnd {
        self.hwnd
    }
    fn attach(&mut self, callbacks: Rc<dyn SciterEvents>) -> Host {
        sciter::Host::attach_with(self.raw(), HostHandler(callbacks))
    }
}

pub struct Sciter {
    hwnd: SciterHandle,
    host: Host,
    startup: Instant,
}

unsafe impl Send for Sciter {}
unsafe impl Sync for Sciter {}

static SCITER_INITIALIZED: AtomicBool = AtomicBool::new(false);

impl Sciter {
    pub fn new(width: u32, height: u32, hwnd: u64, callbacks: Box<dyn SciterEvents>) -> Sciter {
        let sciter = || -> Result<Sciter, &'static str> {
            if !SCITER_INITIALIZED.compare_and_swap(false, true, Ordering::Relaxed) {
                load_sciter().or(Err("Couldn't load sciter native library :("))?;

                // Give sciter necessary privileges
                sciter::set_options(sciter::RuntimeOptions::UxTheming(true)).unwrap();
                sciter::set_options(sciter::RuntimeOptions::DebugMode(true)).unwrap();
                sciter::set_options(sciter::RuntimeOptions::ScriptFeatures(0xFF)).unwrap();
            }

            let raw_hwnd = hwnd as SciterHwnd;
            let mut hwnd = SciterHandle { hwnd: raw_hwnd };

            Self::create(&mut hwnd);

            let callbacks: Rc<dyn SciterEvents> = Rc::from(callbacks);

            let host = hwnd.attach(callbacks.clone());

            host.attach_handler(EventHandler(callbacks));

            let mut sciter = Sciter {
                hwnd,
                host,
                startup: Instant::now(),
            };
            sciter.resolution(100);
            sciter.resize(width, height);
            Ok(sciter)
        }();

        sciter.unwrap()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        handle_message(self.hwnd.raw(), Message::Size { width, height });
    }
    pub fn resolution(&mut self, ppi: u32) {
        handle_message(self.hwnd.raw(), Message::Resolution { ppi });
    }
    pub fn heartbeat(&mut self) {
        handle_message(
            self.hwnd.raw(),
            Message::Heartbit {
                milliseconds: std::time::Instant::now()
                    .duration_since(self.startup)
                    .as_millis() as u32,
            },
        );
    }
    pub fn render(&mut self) {
        let root = self
            .host
            .get_root()
            .ok_or("Couldn't get document root for rendering")
            .unwrap()
            .as_ptr();

        handle_message(
            self.hwnd.raw(),
            Message::Paint(PaintLayer {
                element: root,
                is_foreground: true,
            }),
        );
    }
    pub fn redraw(&mut self) {
        handle_message(self.hwnd.raw(), Message::Redraw);
    }

    pub fn mouse_moved(&mut self, x: i32, y: i32) {
        let event = MouseEvent {
            event: MOUSE_EVENTS::MOUSE_MOVE,
            button: MOUSE_BUTTONS::NONE,
            modifiers: KEYBOARD_STATES::from(0),
            pos: POINT { x, y },
        };
        handle_message(self.hwnd.raw(), Message::Mouse(event));
    }
    pub fn mouse_down(&mut self, x: i32, y: i32) {
        let event = MouseEvent {
            event: MOUSE_EVENTS::MOUSE_DOWN,
            button: MOUSE_BUTTONS::MAIN,
            modifiers: KEYBOARD_STATES::from(0),
            pos: POINT { x, y },
        };
        handle_message(self.hwnd.raw(), Message::Mouse(event));
    }
    pub fn mouse_up(&mut self, x: i32, y: i32) {
        let event = MouseEvent {
            event: MOUSE_EVENTS::MOUSE_UP,
            button: MOUSE_BUTTONS::MAIN,
            modifiers: KEYBOARD_STATES::from(0),
            pos: POINT { x, y },
        };
        handle_message(self.hwnd.raw(), Message::Mouse(event));
    }
    pub fn click(&mut self, x: i32, y: i32) {
        let event = MouseEvent {
            event: MOUSE_EVENTS::MOUSE_CLICK,
            button: MOUSE_BUTTONS::MAIN,
            modifiers: KEYBOARD_STATES::from(0),
            pos: POINT { x, y },
        };
        handle_message(self.hwnd.raw(), Message::Mouse(event));
    }

    pub fn call_event(&mut self, name: String, data: String) {
        || -> Result<(), &'static str> {
            self.host
                .get_root()
                .ok_or("Couldn't get root")?
                .call_function("onEvent", &sciter::make_args!(name, data))
                .or(Err("Couldn't call event"))?;
            Ok(())
        }()
        .unwrap();
    }

    pub fn load_html_string(&mut self, html: String) {
        self.host.load_html(html.as_bytes(), None);
    }
    pub fn load_html_file(&mut self, file: String) {
        self.host.load_file(&file);
    }

    pub fn data_ready(&self, uri: String, request_id: u64, data: String) {
        let data = base64::decode(data).unwrap();
        self.host
            .data_ready_async(&uri, &data, Some(request_id as _));
    }

    fn create(hwnd: &mut SciterHandle) {
        handle_message(
            hwnd.raw(),
            Message::Create {
                backend: sciter::types::GFX_LAYER::SKIA_OPENGL,
                transparent: false,
            },
        );
    }
}

/// Extracts sciter dynamic library to a temporary file and loads it dynamically
fn load_sciter() -> anyhow::Result<()> {
    // Remove old sciter libraries in temp
    for file in std::fs::read_dir(std::env::temp_dir())? {
        if let Ok(entry) = file {
            let path = entry.path();
            if path.is_dir()
                && path
                    .file_name()
                    .unwrap_or_default()
                    .to_os_string()
                    .into_string()
                    .unwrap_or_default()
                    .starts_with("sciter")
            {
                std::fs::remove_dir_all(path).ok();
            }
        }
    }

    #[cfg(target_os = "linux")]
    let sciter = include_bytes!("../resources/libsciter.so.zs");

    #[cfg(target_os = "windows")]
    let sciter = include_bytes!("../resources/sciter.dll.zs");

    // Write decompressed sciter library to temporary file
    let tmp_dir = tempfile::Builder::new().prefix("sciter").tempdir()?;
    let tmp_file = tmp_dir.path().join("sciter.dll");

    // Decompress sciter and write to disk
    zstd::stream::copy_decode(&sciter[..], File::create(&tmp_file)?)?;

    let path = &tmp_file.into_os_string().into_string().or(Err(anyhow!(
        "Couldn't convert temp file into appropriate path"
    )))?;

    // Leak directory
    tmp_dir.into_path();

    // Initialize sciter with library in temp
    sciter::set_library(path).or(Err(anyhow!("Couldn't set library")))?;

    Ok(())
}

struct EventHandler(Rc<dyn SciterEvents>);

impl EventHandler {
    fn call_event(&self, name: String, data: String) {
        self.0.on_event(name, data);
    }
}

impl sciter::EventHandler for EventHandler {
    dispatch_script_call! {
        fn call_event(String, String);
    }
}

struct HostHandler(Rc<dyn SciterEvents>);

impl sciter::HostHandler for HostHandler {
    fn on_data_load(&mut self, pnm: &mut SCN_LOAD_DATA) -> Option<LOAD_RESULT> {
        let uri = sciter::utf::w2s(pnm.uri);

        return if !uri.starts_with("sciter:") {
            self.0.on_load_resource(uri.clone(), pnm.request_id as u64);
            Some(LOAD_RESULT::LOAD_DELAYED)
        } else {
            None
        };
    }

    fn on_graphics_critical_failure(&mut self) {
        println!("Critical graphics failure");
    }

    fn on_invalidate(&mut self, _pnm: &SCN_INVALIDATE_RECT) {
        self.0.on_redraw_required();
    }

    fn on_debug_output(
        &mut self,
        _subsystem: OUTPUT_SUBSYTEMS,
        _severity: OUTPUT_SEVERITY,
        message: &str,
    ) {
        println!("{}", message);
    }
}

pub trait SciterEvents {
    fn on_redraw_required(&self);
    fn on_event(&self, name: String, data: String);
    fn on_load_resource(&self, uri: String, request_id: u64);
}

include!(concat!(env!("OUT_DIR"), "/skiter.uniffi.rs"));
