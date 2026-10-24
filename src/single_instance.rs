//! Instancia única: un mutex con nombre. Si ya hay una Butaca corriendo, la nueva le pide
//! (con un evento con nombre) que abra la configuración y se cierra.

use std::ptr::null;
use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
use windows_sys::Win32::System::Threading::{CreateEventW, CreateMutexW, SetEvent, WaitForSingleObject, INFINITE};

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

/// Mantiene vivo el mutex mientras el proceso corra.
pub struct Instance {
    _mutex: isize,
    open_event: isize,
}

/// `Some` si esta es la primera instancia; `None` si ya había otra (a la que se avisó).
pub fn acquire() -> Option<Instance> {
    unsafe {
        let mutex = CreateMutexW(null(), 0, wide(r"Local\Butaca-Instance").as_ptr());
        let already = GetLastError() == ERROR_ALREADY_EXISTS;
        // Evento de reinicio automático, compartido por nombre entre instancias.
        let event = CreateEventW(null(), 0, 0, wide(r"Local\Butaca-OpenSettings").as_ptr());
        if mutex.is_null() || event.is_null() {
            // No se pudo comprobar: mejor arrancar que no mostrar nada.
            return Some(Instance { _mutex: mutex as isize, open_event: event as isize });
        }
        if already {
            SetEvent(event);
            return None;
        }
        Some(Instance { _mutex: mutex as isize, open_event: event as isize })
    }
}

impl Instance {
    /// Ejecuta `f` cada vez que otra instancia pide abrir la configuración.
    pub fn on_open_request(&self, f: impl Fn() + Send + 'static) {
        let ev = self.open_event;
        if ev == 0 {
            return;
        }
        std::thread::spawn(move || loop {
            if unsafe { WaitForSingleObject(ev as HANDLE, INFINITE) } != 0 {
                break;
            }
            f();
        });
    }
}
