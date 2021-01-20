use std::cell::RefCell;
use std::rc::Rc;

use sciter::types::{HWINDOW, LOAD_RESULT, POINT, SCN_LOAD_DATA};
use sciter::windowless::{
    handle_message, KeyboardEvent, Message as SciterMessage, MouseEvent,
    PaintLayer as SciterPaintLayer, KEYBOARD_STATES, KEY_EVENTS, MOUSE_BUTTONS, MOUSE_EVENTS,
};
use sciter::{
    EventHandler as SciterEventHandler, Host as SciterHost, HostHandler as SciterHostHandler,
    Value as SciterValue, HELEMENT,
};
use serde_cbor::Value as CborValue;

use crate::bindings::options::GfxLayer;

pub struct Host {
    handle: HWINDOW,
    host: SciterHost,
    host_callbacks: Rc<RefCell<Callbacks>>,
}

impl Host {
    pub fn create(handle: u64, backend: GfxLayer, transparent: bool) -> Result<Host, &'static str> {
        let handle = handle as HWINDOW;

        let host_callbacks = Rc::new(RefCell::new(Callbacks {
            data_load_callback: None,
            native_function_invocation_callback: None,
        }));

        handle_message(
            handle,
            SciterMessage::Create {
                backend: backend.to_sciter(),
                transparent,
            },
        );

        let sciter_host = Host {
            handle,
            host: SciterHost::attach_with(
                handle,
                HostHandlerWrapper {
                    callbacks: host_callbacks.clone(),
                },
            ),
            host_callbacks: host_callbacks.clone(),
        };

        sciter_host.host.event_handler(EventHandlerWrapper {
            callbacks: host_callbacks,
        });

        Ok(sciter_host)
    }

    pub fn dispatch_message(&self, message: Message) {
        handle_message(self.handle, message.to_sciter());
    }

    pub fn on_data_load(&self, data_load_callback: Box<dyn DataLoadCallback>) {
        self.host_callbacks
            .borrow_mut()
            .data_load_callback
            .replace(data_load_callback);
    }

    pub fn on_native_function_invocation(
        &self,
        native_function_invocation_callback: Box<dyn NativeFunctionInvocationCallback>,
    ) {
        self.host_callbacks
            .borrow_mut()
            .native_function_invocation_callback
            .replace(native_function_invocation_callback);
    }

    pub fn data_ready(&self, uri: String, request_id: u64, data: &[i8]) {
        let data = unsafe { &*(data as *const _ as *const [u8]) };
        self.host
            .data_ready_async(&uri, &data, Some(request_id as _));
    }

    pub fn load_html(&mut self, html: String, uri: Option<&str>) {
        self.host.load_html(html.as_bytes(), uri);
    }

    pub fn load_file(&mut self, path: String) {
        self.host.load_file(&path);
    }

    pub fn call_function(&mut self, name: String, data: &[i8]) -> Result<(), &'static str> {
        let data = unsafe { &*(data as *const _ as *const [u8]) };
        let params = sciter_serde::to_value(
            &serde_cbor::from_slice::<CborValue>(data)
                .or(Err("Couldn't deserialize cbor value"))?,
        )
        .or(Err("Couldn't convert cbor value to sciter value"))?;

        let params: Vec<SciterValue> = params.values().collect();

        self.host
            .get_root()
            .ok_or("Couldn't get root")?
            .call_function(&name, &params)
            .or(Err("Couldn't call event"))?;

        Ok(())
    }
}

pub enum Message {
    Create {
        backend: GfxLayer,
        transparent: bool,
    },
    Destroy,
    Size {
        width: u32,
        height: u32,
    },
    Resolution {
        ppi: u32,
    },
    Focus {
        enter: bool,
    },
    Heartbit {
        milliseconds: u32,
    },
    Redraw,
    Paint {
        layer: PaintLayer,
    },
    Mouse {
        event: MouseEvents,
        button: MouseButtons,
        modifiers: KeyboardStates,
        pos: Point,
    },
    Keyboard {
        event: KeyEvents,
        code: u32,
        modifiers: KeyboardStates,
    },
}

impl Message {
    pub fn create(backend: GfxLayer, transparent: bool) -> Message {
        Message::Create {
            backend,
            transparent,
        }
    }

    pub fn destroy() -> Message {
        Message::Destroy
    }

    pub fn size(width: u32, height: u32) -> Message {
        Message::Size { width, height }
    }

    pub fn resolution(ppi: u32) -> Message {
        Message::Resolution { ppi }
    }

    pub fn focus(enter: bool) -> Message {
        Message::Focus { enter }
    }

    pub fn heartbit(milliseconds: u32) -> Message {
        Message::Heartbit { milliseconds }
    }

    pub fn redraw() -> Message {
        Message::Redraw
    }

    pub fn paint(layer: PaintLayer) -> Message {
        Message::Paint { layer }
    }

    //TODO: RenderTo (bitmap)

    pub fn mouse(
        event: MouseEvents,
        button: MouseButtons,
        modifiers: KeyboardStates,
        pos: Point,
    ) -> Message {
        Message::Mouse {
            event,
            button,
            modifiers,
            pos,
        }
    }

    pub fn keyboard(event: KeyEvents, code: u32, modifiers: KeyboardStates) -> Message {
        Message::Keyboard {
            event,
            code,
            modifiers,
        }
    }
}

impl Message {
    fn to_sciter(&self) -> SciterMessage {
        match self {
            Message::Create {
                backend,
                transparent,
            } => SciterMessage::Create {
                backend: backend.to_sciter(),
                transparent: *transparent,
            },
            Message::Destroy => SciterMessage::Destroy,
            Message::Size { width, height } => SciterMessage::Size {
                width: *width,
                height: *height,
            },
            Message::Resolution { ppi } => SciterMessage::Resolution { ppi: *ppi },
            Message::Focus { enter } => SciterMessage::Focus { enter: *enter },
            Message::Heartbit { milliseconds } => SciterMessage::Heartbit {
                milliseconds: *milliseconds,
            },
            Message::Redraw => SciterMessage::Redraw,
            Message::Paint { layer } => SciterMessage::Paint(layer.to_sciter()),
            Message::Mouse {
                event,
                button,
                modifiers,
                pos,
            } => SciterMessage::Mouse(MouseEvent {
                event: event.to_sciter(),
                button: button.to_sciter(),
                modifiers: modifiers.to_sciter(),
                pos: pos.to_sciter(),
            }),
            Message::Keyboard {
                event,
                code,
                modifiers,
            } => SciterMessage::Keyboard(KeyboardEvent {
                event: event.to_sciter(),
                code: *code,
                modifiers: modifiers.to_sciter(),
            }),
        }
    }
}

pub trait DataLoadCallback {
    fn on_data_load(&self, uri: String, request_id: u64) -> i32;
}

pub trait NativeFunctionInvocationCallback {
    fn on_native_function_invocation(&self, name: String, data: &[i8]) -> bool;
}

struct Callbacks {
    data_load_callback: Option<Box<dyn DataLoadCallback>>,
    native_function_invocation_callback: Option<Box<dyn NativeFunctionInvocationCallback>>,
}

struct HostHandlerWrapper {
    callbacks: Rc<RefCell<Callbacks>>,
}

impl SciterHostHandler for HostHandlerWrapper {
    fn on_data_load(&mut self, pnm: &mut SCN_LOAD_DATA) -> Option<LOAD_RESULT> {
        if let Some(callback) = &self.callbacks.borrow().data_load_callback {
            let uri = sciter::utf::w2s(pnm.uri);
            let request_id = pnm.request_id as u64;

            match callback.on_data_load(uri, request_id) {
                -1 => None,
                0 => Some(LOAD_RESULT::LOAD_DEFAULT),
                1 => Some(LOAD_RESULT::LOAD_DISCARD),
                2 => Some(LOAD_RESULT::LOAD_DELAYED),
                3 => Some(LOAD_RESULT::LOAD_MYSELF),
                _ => panic!("Invalid LOAD_RESULT"),
            }
        } else {
            None
        }
    }
}

struct EventHandlerWrapper {
    callbacks: Rc<RefCell<Callbacks>>,
}

impl SciterEventHandler for EventHandlerWrapper {
    fn on_script_call(
        &mut self,
        _root: HELEMENT,
        name: &str,
        args: &[SciterValue],
    ) -> Option<SciterValue> {
        if let Some(callback) = &self.callbacks.borrow().native_function_invocation_callback {
            let cbor = sciter_serde::from_value::<CborValue>(&args.iter().cloned().collect());
            if let Ok(cbor) = cbor {
                let data = serde_cbor::to_vec(&cbor);
                if let Ok(data) = data {
                    let data = unsafe { &*(data.as_slice() as *const _ as *const [i8]) };
                    if callback.on_native_function_invocation(name.to_owned(), data) {
                        return Some(SciterValue::from(()));
                    }
                } else {
                    println!("[SKITER] [ERROR] Couldn't serialize cbor value");
                }
            } else {
                println!("[SKITER] [ERROR] Couldn't convert sciter value to cbor value");
            }
        }
        None
    }
}

#[derive(Copy, Clone)]
pub struct PaintLayer {
    pub element: u64, //TODO: Typesafe
    pub is_foreground: bool,
}

impl PaintLayer {
    pub fn new(element: u64, is_foreground: bool) -> PaintLayer {
        PaintLayer {
            element,
            is_foreground,
        }
    }
}

impl PaintLayer {
    fn to_sciter(&self) -> SciterPaintLayer {
        SciterPaintLayer {
            element: self.element as HELEMENT,
            is_foreground: self.is_foreground,
        }
    }
}

#[derive(Copy, Clone)]
pub enum MouseEvents {
    MouseMove,
    MouseUp,
    MouseDown,
    MouseWheel,
    MouseClick,
}

impl MouseEvents {
    fn to_sciter(&self) -> MOUSE_EVENTS {
        match self {
            MouseEvents::MouseMove => MOUSE_EVENTS::MOUSE_MOVE,
            MouseEvents::MouseUp => MOUSE_EVENTS::MOUSE_UP,
            MouseEvents::MouseDown => MOUSE_EVENTS::MOUSE_DOWN,
            MouseEvents::MouseWheel => MOUSE_EVENTS::MOUSE_WHEEL,
            MouseEvents::MouseClick => MOUSE_EVENTS::MOUSE_CLICK,
        }
    }
}

#[derive(Copy, Clone)]
pub enum MouseButtons {
    Left,
    Right,
    Middle,
    None,
}

impl MouseButtons {
    fn to_sciter(&self) -> MOUSE_BUTTONS {
        match self {
            MouseButtons::Left => MOUSE_BUTTONS::MAIN,
            MouseButtons::Right => MOUSE_BUTTONS::PROP,
            MouseButtons::Middle => MOUSE_BUTTONS::MIDDLE,
            MouseButtons::None => MOUSE_BUTTONS::NONE,
        }
    }
}

#[derive(Copy, Clone)]
pub enum KeyboardStates {
    ControlKeyPressed,
    ShiftKeyPressed,
    AltKeyPressed,
    None,
}

impl KeyboardStates {
    fn to_sciter(&self) -> KEYBOARD_STATES {
        match self {
            KeyboardStates::ControlKeyPressed => KEYBOARD_STATES::CONTROL_KEY_PRESSED,
            KeyboardStates::ShiftKeyPressed => KEYBOARD_STATES::SHIFT_KEY_PRESSED,
            KeyboardStates::AltKeyPressed => KEYBOARD_STATES::ALT_KEY_PRESSED,
            KeyboardStates::None => KEYBOARD_STATES::from(0),
        }
    }
}

#[derive(Copy, Clone)]
pub enum KeyEvents {
    KeyDown,
    KeyUp,
    KeyChar,
}

impl KeyEvents {
    fn to_sciter(&self) -> KEY_EVENTS {
        match self {
            KeyEvents::KeyDown => KEY_EVENTS::KEY_DOWN,
            KeyEvents::KeyUp => KEY_EVENTS::KEY_UP,
            KeyEvents::KeyChar => KEY_EVENTS::KEY_CHAR,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }
}

impl Point {
    fn to_sciter(&self) -> POINT {
        POINT {
            x: self.x,
            y: self.y,
        }
    }
}
