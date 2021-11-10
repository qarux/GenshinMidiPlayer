use std::fmt::{Display, Formatter};
use std::os::raw::{c_int, c_uint};
use std::sync::mpsc::Sender;
use std::{ptr, thread};
use x11::keysym::{XK_Num_Lock, XK_Scroll_Lock, XK_backslash};
use x11::xlib;
use x11::xlib::{
    AnyModifier, ButtonPress, ControlMask, GrabModeAsync, KeyPress, KeySym, LockMask, Mod1Mask,
    Mod2Mask, Mod3Mask, Mod4Mask, Mod5Mask, ShiftMask, XButtonEvent, XEvent, XKeyEvent,
};

const DEFAULT_HOTKEY: c_uint = XK_backslash;
const DEFAULT_HOTKEY_SYMBOL: &str = "\\";

pub struct HotkeyManager {
    tx: Sender<()>,
}

pub struct Hotkey {
    keycode: KeySym,
    mask: c_uint,
}

// This code is awful. Good luck
impl HotkeyManager {
    pub fn register_hotkey(hotkey: Hotkey, callback: impl 'static + Fn() + Send) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        unsafe {
            thread::spawn(move || {
                let display = open_display();
                grab_key(hotkey.keycode, hotkey.mask, display);

                let keycode = xlib::XKeysymToKeycode(display, hotkey.keycode);
                let event: *mut XEvent = &mut XEvent { type_: 0 };
                loop {
                    xlib::XNextEvent(display, event);
                    if rx.try_recv().is_ok() {
                        break;
                    }

                    let (num_lock_mask, scroll_lock_mask, caps_lock_mask) =
                        get_offending_modifiers(display);
                    if (*event).type_ == KeyPress {
                        let key_event = event.cast::<XKeyEvent>();

                        if keycode as u32 == (*key_event).keycode
                            && hotkey.mask
                                == (*key_event).state
                                    & !(num_lock_mask | scroll_lock_mask | caps_lock_mask)
                        {
                            callback();
                        }
                    } else if (*event).type_ == ButtonPress {
                        let button_event = event.cast::<XButtonEvent>();

                        if keycode as u32 == (*button_event).button
                            && hotkey.mask
                                == (*button_event).state
                                    & !(num_lock_mask | scroll_lock_mask | caps_lock_mask)
                        {
                            callback();
                        }
                    }
                }

                ungrab_key(hotkey.keycode, hotkey.mask, display);
                close_display(display);
            });
        }

        HotkeyManager { tx }
    }

    pub fn unregister(self) {}
}

impl Hotkey {
    pub fn new() -> Self {
        Hotkey {
            keycode: DEFAULT_HOTKEY as u64,
            mask: 0,
        }
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        self.tx.send(()).unwrap();
    }
}

impl Display for Hotkey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", DEFAULT_HOTKEY_SYMBOL)
    }
}

unsafe fn open_display() -> *mut xlib::Display {
    xlib::XOpenDisplay(ptr::null())
}

unsafe fn close_display(display: *mut xlib::Display) {
    xlib::XCloseDisplay(display);
}

unsafe fn grab_key(key: KeySym, mask: c_uint, display: *mut xlib::Display) {
    let key = xlib::XKeysymToKeycode(display, key) as i32;
    if key == 0 {
        return;
    }

    xlib::XSync(display, c_int::from(false));

    let (num_lock_mask, scroll_lock_mask, caps_lock_mask) = get_offending_modifiers(display);
    for i in 0..(xlib::XScreenCount(display)) {
        let window = xlib::XRootWindow(display, i);
        let modifier = mask & !(num_lock_mask | caps_lock_mask | scroll_lock_mask);

        xlib::XGrabKey(
            display,
            key,
            modifier,
            window,
            c_int::from(false),
            GrabModeAsync,
            GrabModeAsync,
        );

        if modifier == AnyModifier {
            return;
        }

        if num_lock_mask != 0 {
            xlib::XGrabKey(
                display,
                key,
                modifier | num_lock_mask,
                window,
                c_int::from(false),
                GrabModeAsync,
                GrabModeAsync,
            );
        }

        if caps_lock_mask != 0 {
            xlib::XGrabKey(
                display,
                key,
                modifier | caps_lock_mask,
                window,
                c_int::from(false),
                GrabModeAsync,
                GrabModeAsync,
            );
        }

        if scroll_lock_mask != 0 {
            xlib::XGrabKey(
                display,
                key,
                modifier | scroll_lock_mask,
                window,
                c_int::from(false),
                GrabModeAsync,
                GrabModeAsync,
            );
        }

        if num_lock_mask != 0 && caps_lock_mask != 0 {
            xlib::XGrabKey(
                display,
                key,
                modifier | num_lock_mask | caps_lock_mask,
                window,
                c_int::from(false),
                GrabModeAsync,
                GrabModeAsync,
            );
        }

        if num_lock_mask != 0 && scroll_lock_mask != 0 {
            xlib::XGrabKey(
                display,
                key,
                modifier | num_lock_mask | scroll_lock_mask,
                window,
                c_int::from(false),
                GrabModeAsync,
                GrabModeAsync,
            );
        }

        if caps_lock_mask != 0 && scroll_lock_mask != 0 {
            xlib::XGrabKey(
                display,
                key,
                modifier | caps_lock_mask | scroll_lock_mask,
                window,
                c_int::from(false),
                GrabModeAsync,
                GrabModeAsync,
            );
        }

        if num_lock_mask != 0 && caps_lock_mask != 0 && scroll_lock_mask != 0 {
            xlib::XGrabKey(
                display,
                key,
                modifier | num_lock_mask | caps_lock_mask | scroll_lock_mask,
                window,
                c_int::from(false),
                GrabModeAsync,
                GrabModeAsync,
            );
        }
    }

    xlib::XSync(display, c_int::from(false));
}

unsafe fn ungrab_key(key: KeySym, mask: c_uint, display: *mut xlib::Display) {
    let key = xlib::XKeysymToKeycode(display, key) as i32;
    if key == 0 {
        return;
    }

    xlib::XSync(display, c_int::from(false));

    let (num_lock_mask, scroll_lock_mask, caps_lock_mask) = get_offending_modifiers(display);
    for i in 0..(xlib::XScreenCount(display)) {
        let window = xlib::XRootWindow(display, i);
        let modifier = mask & !(num_lock_mask | caps_lock_mask | scroll_lock_mask);

        xlib::XUngrabKey(display, key, modifier, window);

        if modifier == AnyModifier {
            return;
        }

        if num_lock_mask != 0 {
            xlib::XUngrabKey(display, key, modifier | num_lock_mask, window);
        }

        if caps_lock_mask != 0 {
            xlib::XUngrabKey(display, key, modifier | caps_lock_mask, window);
        }

        if scroll_lock_mask != 0 {
            xlib::XUngrabKey(display, key, modifier | scroll_lock_mask, window);
        }

        if num_lock_mask != 0 && caps_lock_mask != 0 {
            xlib::XUngrabKey(
                display,
                key,
                modifier | num_lock_mask | caps_lock_mask,
                window,
            );
        }

        if num_lock_mask != 0 && scroll_lock_mask != 0 {
            xlib::XUngrabKey(display, key, modifier | scroll_lock_mask, window);
        }

        if caps_lock_mask != 0 && scroll_lock_mask != 0 {
            xlib::XUngrabKey(
                display,
                key,
                modifier | caps_lock_mask | scroll_lock_mask,
                window,
            );
        }

        if num_lock_mask != 0 && caps_lock_mask != 0 && scroll_lock_mask != 0 {
            xlib::XUngrabKey(
                display,
                key,
                modifier | num_lock_mask | caps_lock_mask | scroll_lock_mask,
                window,
            );
        }
    }

    xlib::XSync(display, c_int::from(false));
}

unsafe fn get_offending_modifiers(display: *mut xlib::Display) -> (c_uint, c_uint, c_uint) {
    let mask_table = [
        ShiftMask,
        LockMask,
        ControlMask,
        Mod1Mask,
        Mod2Mask,
        Mod3Mask,
        Mod4Mask,
        Mod5Mask,
    ];

    let mut num_lock_mask = 0;
    let mut scroll_lock_mask = 0;
    let nlock = xlib::XKeysymToKeycode(display, XK_Num_Lock as u64);
    let slock = xlib::XKeysymToKeycode(display, XK_Scroll_Lock as u64);
    let modmap = xlib::XGetModifierMapping(display);

    if !modmap.is_null() && (*modmap).max_keypermod > 0 {
        for i in 0..(mask_table.len() * ((*modmap).max_keypermod as usize)) {
            if (*(*modmap).modifiermap.add(i)) == nlock && nlock != 0 {
                num_lock_mask = mask_table[i / ((*modmap).max_keypermod as usize)];
            } else if (*(*modmap).modifiermap.add(i)) == slock && slock != 0 {
                scroll_lock_mask = mask_table[i / ((*modmap).max_keypermod as usize)];
            }
        }
    }
    let caps_lock = LockMask;

    if !modmap.is_null() {
        xlib::XFreeModifiermap(modmap);
    }

    (num_lock_mask, scroll_lock_mask, caps_lock)
}
