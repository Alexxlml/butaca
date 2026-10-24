//! Logger simple a `%APPDATA%\Butaca\butaca.log`, con rotación (1 MB × 3 archivos).

use crate::config;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

const MAX_BYTES: u64 = 1024 * 1024;
const KEEP: usize = 3;

struct Inner {
    file: Option<File>,
    size: u64,
}

struct FileLogger(Mutex<Inner>);

fn timestamp() -> String {
    use windows_sys::Win32::Foundation::SYSTEMTIME;
    use windows_sys::Win32::System::SystemInformation::GetLocalTime;
    let mut t: SYSTEMTIME = unsafe { std::mem::zeroed() };
    unsafe { GetLocalTime(&mut t) };
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond)
}

pub fn log_path() -> PathBuf {
    config::config_dir().join("butaca.log")
}

/// `butaca.log` es el actual; `butaca.1.log` y `butaca.2.log` los anteriores.
fn rotated(n: usize) -> PathBuf {
    if n == 0 { log_path() } else { config::config_dir().join(format!("butaca.{n}.log")) }
}

fn rotate() {
    let _ = fs::remove_file(rotated(KEEP - 1));
    for n in (0..KEEP - 1).rev() {
        let _ = fs::rename(rotated(n), rotated(n + 1));
    }
}

fn open_current() -> (Option<File>, u64) {
    let file = OpenOptions::new().create(true).append(true).open(log_path()).ok();
    let size = file.as_ref().and_then(|f| f.metadata().ok()).map(|m| m.len()).unwrap_or(0);
    (file, size)
}

impl log::Log for FileLogger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }
    fn log(&self, r: &log::Record) {
        let line = format!("{} [{}] {}\n", timestamp(), r.level(), r.args());
        #[cfg(debug_assertions)]
        print!("{line}");
        let Ok(mut g) = self.0.lock() else { return };
        if g.size + line.len() as u64 > MAX_BYTES {
            g.file = None; // cerrar antes de renombrar (Windows no deja renombrar archivos abiertos)
            rotate();
            (g.file, g.size) = open_current();
        }
        if let Some(f) = g.file.as_mut() {
            if f.write_all(line.as_bytes()).is_ok() {
                g.size += line.len() as u64;
            }
        }
    }
    fn flush(&self) {}
}

pub fn init() {
    let _ = fs::create_dir_all(config::config_dir());
    let (file, size) = open_current();
    // El logger vive hasta el fin del proceso.
    let logger: &'static FileLogger = Box::leak(Box::new(FileLogger(Mutex::new(Inner { file, size }))));
    if log::set_logger(logger).is_ok() {
        log::set_max_level(log::LevelFilter::Info);
    }
    // Sin consola, un fallo inesperado solo se vería en el log.
    std::panic::set_hook(Box::new(|info| log::error!("Unexpected error: {info}")));
}
