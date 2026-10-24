//! Ventana de configuración en Win32 directo (sin frameworks).
//! Corre en su propio hilo con su propio bucle de mensajes, para que nunca bloquee la bandeja.

use crate::config::Config;
use crate::i18n::{self, t, tf};
use crate::discovery::{self, Found};
use crate::kodi::{friendly_error, Kodi};
use crate::{autostart, config};
use std::ffi::c_void;
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{CreateFontIndirectW, DeleteObject, COLOR_BTNFACE, HFONT};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Controls::{
    InitCommonControlsEx, BST_CHECKED, ICC_LINK_CLASS, ICC_STANDARD_CLASSES, INITCOMMONCONTROLSEX, NMHDR, NM_CLICK, NM_RETURN,
};
use windows_sys::Win32::UI::HiDpi::{AdjustWindowRectExForDpi, GetDpiForWindow, GetSystemMetricsForDpi, SystemParametersInfoForDpi};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{EnableWindow, SetFocus};
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

const HOST_PLACEHOLDER: &str = "192.168.1.50";
const TMDB_URL: &str = "https://www.themoviedb.org/settings/api";

const ID_HOST: i32 = 100;
const ID_PORT: i32 = 101;
const ID_USER: i32 = 102;
const ID_PASS: i32 = 103;
const ID_TMDB: i32 = 104;
const ID_LIST: i32 = 105;
const ID_SEARCH: i32 = 106;
const ID_TEST: i32 = 107;
const ID_AUTOSTART: i32 = 108;
const ID_IDLE: i32 = 109;
const ID_LINK: i32 = 110;
const ID_STATUS: i32 = 111;
const ID_LANG: i32 = 112;

const WM_SEARCH_DONE: u32 = WM_APP + 1;
const WM_TEST_DONE: u32 = WM_APP + 2;

const EM_SETCUEBANNER: u32 = 0x1501;
const CLIENT_W: i32 = 440;
const CLIENT_H: i32 = 488;
const WINDOW_STYLE: u32 = WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;

static OPEN: AtomicBool = AtomicBool::new(false);
static WINDOW: AtomicIsize = AtomicIsize::new(0);

type OnSave = Arc<dyn Fn(Config) + Send + Sync>;

struct State {
    dpi: i32,
    font: HFONT,
    on_save: OnSave,
    base: Config,
    found: Vec<Found>,
    host: HWND,
    port: HWND,
    user: HWND,
    pass: HWND,
    tmdb: HWND,
    list: HWND,
    link: HWND,
    status: HWND,
    btn_search: HWND,
    btn_test: HWND,
    chk_autostart: HWND,
    chk_idle: HWND,
    combo: HWND,
    /// Controles con texto traducible y la clave de cada uno.
    texts: Vec<(HWND, &'static str)>,
    saved: bool,
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

fn lo(w: WPARAM) -> i32 {
    (w & 0xFFFF) as i32
}

fn hi(w: WPARAM) -> i32 {
    ((w >> 16) & 0xFFFF) as i32
}

unsafe fn get_text(h: HWND) -> String {
    let len = GetWindowTextLengthW(h);
    let mut buf = vec![0u16; len as usize + 1];
    let n = GetWindowTextW(h, buf.as_mut_ptr(), buf.len() as i32);
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}

unsafe fn set_text(h: HWND, s: &str) {
    SetWindowTextW(h, wide(s).as_ptr());
}

/// Entrega el resultado de un hilo auxiliar a la ventana; si ya se cerró, lo descarta.
fn post<T>(hwnd: usize, msg: u32, value: T) {
    let p = Box::into_raw(Box::new(value));
    if unsafe { PostMessageW(hwnd as HWND, msg, 0, p as LPARAM) } == 0 {
        drop(unsafe { Box::from_raw(p) });
    }
}

/// Abre la ventana, o trae al frente la que ya está abierta.
pub fn open(cfg: Config, on_save: OnSave) {
    if OPEN.swap(true, Ordering::SeqCst) {
        let h = WINDOW.load(Ordering::SeqCst) as HWND;
        if !h.is_null() {
            unsafe {
                ShowWindow(h, SW_RESTORE);
                SetForegroundWindow(h);
            }
        }
        return;
    }
    std::thread::spawn(move || {
        unsafe { run(cfg, on_save) };
        WINDOW.store(0, Ordering::SeqCst);
        OPEN.store(false, Ordering::SeqCst);
    });
}

unsafe fn run(cfg: Config, on_save: OnSave) {
    let hinst = GetModuleHandleW(null());
    InitCommonControlsEx(&INITCOMMONCONTROLSEX {
        dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
        dwICC: ICC_LINK_CLASS | ICC_STANDARD_CLASSES,
    });

    let class = wide("ButacaSettings");
    let wc = WNDCLASSW {
        style: 0,
        lpfnWndProc: Some(wndproc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinst,
        hIcon: LoadIconW(hinst, 1 as *const u16), // recurso 1 = ícono embebido (build.rs)
        hCursor: LoadCursorW(null_mut(), IDC_ARROW),
        hbrBackground: (COLOR_BTNFACE + 1) as usize as _,
        lpszMenuName: null(),
        lpszClassName: class.as_ptr(),
    };
    RegisterClassW(&wc); // devuelve 0 si ya estaba registrada: no importa

    let state = Box::into_raw(Box::new(State {
        dpi: 96,
        font: null_mut(),
        on_save,
        base: cfg,
        found: Vec::new(),
        host: null_mut(),
        port: null_mut(),
        user: null_mut(),
        pass: null_mut(),
        tmdb: null_mut(),
        list: null_mut(),
        link: null_mut(),
        status: null_mut(),
        btn_search: null_mut(),
        btn_test: null_mut(),
        chk_autostart: null_mut(),
        chk_idle: null_mut(),
        combo: null_mut(),
        texts: Vec::new(),
        saved: false,
    }));
    let title = wide(&t("settings.title"));
    let hwnd = CreateWindowExW(
        0,
        class.as_ptr(),
        title.as_ptr(),
        WINDOW_STYLE,
        0,
        0,
        100,
        100,
        null_mut(),
        null_mut(),
        hinst,
        state as *const c_void,
    );
    if hwnd.is_null() {
        drop(Box::from_raw(state));
        log::error!("Could not create the settings window");
        return;
    }
    WINDOW.store(hwnd as isize, Ordering::SeqCst);

    // Tamaño y posición según el DPI del monitor; centrada en la pantalla.
    let dpi = GetDpiForWindow(hwnd) as i32;
    let mut rc = RECT { left: 0, top: 0, right: CLIENT_W * dpi / 96, bottom: CLIENT_H * dpi / 96 };
    AdjustWindowRectExForDpi(&mut rc, WINDOW_STYLE, 0, 0, dpi as u32);
    let (w, h) = (rc.right - rc.left, rc.bottom - rc.top);
    let x = (GetSystemMetricsForDpi(SM_CXSCREEN, dpi as u32) - w) / 2;
    let y = (GetSystemMetricsForDpi(SM_CYSCREEN, dpi as u32) - h) / 2;
    SetWindowPos(hwnd, null_mut(), x.max(0), y.max(0), w, h, SWP_NOZORDER);
    ShowWindow(hwnd, SW_SHOW);
    SetForegroundWindow(hwnd);
    SetFocus((*state).host);

    let mut msg: MSG = std::mem::zeroed();
    while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
        // IsDialogMessage da Tab, Esc (cancelar) y Enter (guardar) sin ser un diálogo.
        if IsDialogMessageW(hwnd, &msg) == 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

struct Builder {
    parent: HWND,
    hinst: *mut c_void,
    dpi: i32,
    font: HFONT,
}

impl Builder {
    fn s(&self, v: i32) -> i32 {
        (v * self.dpi + 48) / 96
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn add(&self, class: &str, text: &str, style: u32, ex: u32, x: i32, y: i32, w: i32, h: i32, id: i32) -> HWND {
        let (c, t) = (wide(class), wide(text));
        let hwnd = CreateWindowExW(
            ex,
            c.as_ptr(),
            t.as_ptr(),
            WS_CHILD | WS_VISIBLE | style,
            self.s(x),
            self.s(y),
            self.s(w),
            self.s(h),
            self.parent,
            id as usize as _,
            self.hinst,
            null(),
        );
        SendMessageW(hwnd, WM_SETFONT, self.font as WPARAM, 1);
        hwnd
    }

    unsafe fn label(&self, text: &str, x: i32, y: i32, w: i32, h: i32) -> HWND {
        self.add("STATIC", text, 0, 0, x, y, w, h, -1)
    }

    unsafe fn edit(&self, x: i32, y: i32, w: i32, id: i32, extra: u32) -> HWND {
        self.add("EDIT", "", WS_TABSTOP | ES_AUTOHSCROLL as u32 | extra, WS_EX_CLIENTEDGE, x, y, w, 23, id)
    }

    unsafe fn button(&self, text: &str, x: i32, y: i32, w: i32, h: i32, id: i32, style: i32) -> HWND {
        self.add("BUTTON", text, WS_TABSTOP | style as u32, 0, x, y, w, h, id)
    }
}

unsafe fn on_create(hwnd: HWND, st: &mut State) {
    let hinst = GetModuleHandleW(null());
    st.dpi = GetDpiForWindow(hwnd) as i32;

    // Fuente del sistema (Segoe UI) al DPI de este monitor.
    let mut ncm: NONCLIENTMETRICSW = std::mem::zeroed();
    ncm.cbSize = std::mem::size_of::<NONCLIENTMETRICSW>() as u32;
    SystemParametersInfoForDpi(SPI_GETNONCLIENTMETRICS, ncm.cbSize, &mut ncm as *mut _ as *mut c_void, 0, st.dpi as u32);
    st.font = CreateFontIndirectW(&ncm.lfMessageFont);

    let b = Builder { parent: hwnd, hinst, dpi: st.dpi, font: st.font };
    let cfg = &st.base;

    let h = b.label(&t("settings.host"), 16, 14, 230, 18);
    st.texts.push((h, "settings.host"));
    st.host = b.edit(16, 34, 230, ID_HOST, 0);
    set_text(st.host, &cfg.kodi_host);
    // El texto del placeholder debe seguir vivo mientras exista el control.
    let cue: &'static [u16] = Box::leak(wide(HOST_PLACEHOLDER).into_boxed_slice());
    SendMessageW(st.host, EM_SETCUEBANNER, 1, cue.as_ptr() as LPARAM);
    st.btn_search = b.button(&t("settings.search"), 254, 33, 170, 25, ID_SEARCH, BS_PUSHBUTTON);
    st.texts.push((st.btn_search, "settings.search"));

    let h = b.label(&t("settings.found"), 16, 68, 408, 18);
    st.texts.push((h, "settings.found"));
    st.list = b.add("LISTBOX", "", WS_TABSTOP | WS_VSCROLL | LBS_NOTIFY as u32, WS_EX_CLIENTEDGE, 16, 88, 408, 54, ID_LIST);

    let h = b.label(&t("settings.port"), 16, 152, 80, 18);
    st.texts.push((h, "settings.port"));
    st.port = b.edit(16, 172, 80, ID_PORT, ES_NUMBER as u32);
    set_text(st.port, &cfg.kodi_port.to_string());
    let h = b.label(&t("settings.user"), 108, 152, 140, 18);
    st.texts.push((h, "settings.user"));
    st.user = b.edit(108, 172, 140, ID_USER, 0);
    set_text(st.user, &cfg.kodi_user);
    let h = b.label(&t("settings.password"), 260, 152, 164, 18);
    st.texts.push((h, "settings.password"));
    st.pass = b.edit(260, 172, 164, ID_PASS, ES_PASSWORD as u32);
    set_text(st.pass, &cfg.kodi_password);

    st.btn_test = b.button(&t("settings.test"), 16, 208, 140, 25, ID_TEST, BS_PUSHBUTTON);
    st.texts.push((st.btn_test, "settings.test"));
    st.status = b.add("STATIC", "", 0, 0, 166, 206, 258, 52, ID_STATUS);

    let h = b.label(&t("settings.tmdb"), 16, 268, 408, 18);
    st.texts.push((h, "settings.tmdb"));
    st.tmdb = b.edit(16, 288, 408, ID_TMDB, 0);
    set_text(st.tmdb, &cfg.tmdb_api_key);
    st.link = b.add("SysLink", &t("settings.tmdb_link"), WS_TABSTOP, 0, 16, 318, 408, 18, ID_LINK);
    st.texts.push((st.link, "settings.tmdb_link"));

    st.chk_autostart = b.button(&t("settings.autostart"), 16, 358, 408, 20, ID_AUTOSTART, BS_AUTOCHECKBOX);
    st.texts.push((st.chk_autostart, "settings.autostart"));
    if autostart::is_enabled() {
        SendMessageW(st.chk_autostart, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
    }
    st.chk_idle = b.button(&t("settings.idle"), 16, 382, 408, 20, ID_IDLE, BS_AUTOCHECKBOX);
    st.texts.push((st.chk_idle, "settings.idle"));
    if cfg.show_idle_status {
        SendMessageW(st.chk_idle, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
    }

    let h = b.label(&t("settings.language"), 16, 414, 90, 18);
    st.texts.push((h, "settings.language"));
    st.combo = b.add("COMBOBOX", "", WS_TABSTOP | WS_VSCROLL | CBS_DROPDOWNLIST as u32, 0, 110, 410, 150, 160, ID_LANG);
    let current = i18n::normalize(&cfg.language);
    for (i, (code, name)) in i18n::languages().enumerate() {
        SendMessageW(st.combo, CB_ADDSTRING, 0, wide(name).as_ptr() as LPARAM);
        if code == current {
            SendMessageW(st.combo, CB_SETCURSEL, i, 0);
        }
    }

    let h = b.button(&t("settings.save"), 244, 450, 88, 26, IDOK, BS_DEFPUSHBUTTON);
    st.texts.push((h, "settings.save"));
    let h = b.button(&t("settings.cancel"), 336, 450, 88, 26, IDCANCEL, BS_PUSHBUTTON);
    st.texts.push((h, "settings.cancel"));
}

/// Vuelve a escribir los textos de la ventana con el idioma actual.
unsafe fn apply_texts(hwnd: HWND, st: &State) {
    set_text(hwnd, &t("settings.title"));
    for (h, key) in &st.texts {
        set_text(*h, &t(key));
    }
}

/// Código del idioma elegido en el combo.
unsafe fn selected_language(st: &State) -> &'static str {
    let i = SendMessageW(st.combo, CB_GETCURSEL, 0, 0);
    usize::try_from(i).ok().and_then(|i| i18n::languages().nth(i)).map(|(c, _)| c).unwrap_or(i18n::DEFAULT)
}

unsafe fn state<'a>(hwnd: HWND) -> Option<&'a mut State> {
    (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut State).as_mut()
}

unsafe fn on_save(hwnd: HWND, st: &mut State) {
    let host = get_text(st.host).trim().to_string();
    if host.is_empty() {
        set_text(st.status, &t("settings.need_host"));
        SetFocus(st.host);
        return;
    }
    let Some(port) = get_text(st.port).trim().parse::<u16>().ok().filter(|p| *p > 0) else {
        set_text(st.status, &t("settings.bad_port"));
        SetFocus(st.port);
        return;
    };
    let mut cfg = st.base.clone();
    cfg.kodi_host = host;
    cfg.kodi_port = port;
    cfg.kodi_user = get_text(st.user).trim().to_string();
    cfg.kodi_password = get_text(st.pass);
    cfg.tmdb_api_key = get_text(st.tmdb).trim().to_string();
    cfg.show_idle_status = SendMessageW(st.chk_idle, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
    cfg.language = selected_language(st).to_string();

    if let Err(e) = config::save(&cfg) {
        log::error!("{e}");
        set_text(st.status, &tf("settings.save_failed", &[("error", &e)]));
        return;
    }
    let want_autostart = SendMessageW(st.chk_autostart, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
    if let Err(e) = autostart::set(want_autostart) {
        log::warn!("Could not change the start-with-Windows entry: {e}");
        set_text(st.status, &tf("settings.autostart_failed", &[("error", &e)]));
        // La configuración ya quedó guardada; se aplica igual.
        st.saved = true;
        (st.on_save)(cfg);
        return;
    }
    st.saved = true;
    (st.on_save)(cfg);
    DestroyWindow(hwnd);
}

unsafe fn on_search(st: &mut State, hwnd: HWND) {
    EnableWindow(st.btn_search, 0);
    set_text(st.status, &t("settings.searching"));
    let h = hwnd as usize;
    std::thread::spawn(move || {
        post(h, WM_SEARCH_DONE, discovery::search(Duration::from_secs(4)));
    });
}

unsafe fn on_search_done(st: &mut State, result: Result<Vec<Found>, String>) {
    EnableWindow(st.btn_search, 1);
    SendMessageW(st.list, LB_RESETCONTENT, 0, 0);
    match result {
        Ok(found) if !found.is_empty() => {
            for f in &found {
                let line = wide(&format!("{} — {}:{}", f.name, f.host, f.port));
                SendMessageW(st.list, LB_ADDSTRING, 0, line.as_ptr() as LPARAM);
            }
            let n = found.len();
            st.found = found;
            if n == 1 {
                SendMessageW(st.list, LB_SETCURSEL, 0, 0);
                fill_from_list(st);
            }
            if n == 1 {
                set_text(st.status, &t("settings.found_one"));
            } else {
                set_text(st.status, &tf("settings.found_many", &[("count", &n.to_string())]));
            }
        }
        Ok(_) => {
            st.found.clear();
            set_text(st.status, &t("settings.none_found"));
        }
        Err(e) => {
            st.found.clear();
            set_text(st.status, &tf("settings.search_failed", &[("error", &e)]));
        }
    }
}

unsafe fn fill_from_list(st: &mut State) {
    let sel = SendMessageW(st.list, LB_GETCURSEL, 0, 0);
    if let Some(f) = usize::try_from(sel).ok().and_then(|i| st.found.get(i)) {
        set_text(st.host, &f.host);
        set_text(st.port, &f.port.to_string());
    }
}

unsafe fn on_test(st: &mut State, hwnd: HWND) {
    let host = get_text(st.host).trim().to_string();
    if host.is_empty() {
        set_text(st.status, &t("settings.need_host"));
        SetFocus(st.host);
        return;
    }
    let Some(port) = get_text(st.port).trim().parse::<u16>().ok().filter(|p| *p > 0) else {
        set_text(st.status, &t("settings.bad_port"));
        SetFocus(st.port);
        return;
    };
    let (user, pass) = (get_text(st.user).trim().to_string(), get_text(st.pass));
    EnableWindow(st.btn_test, 0);
    set_text(st.status, &t("settings.testing"));
    let h = hwnd as usize;
    let lang = i18n::current().to_string();
    std::thread::spawn(move || {
        let result = Kodi::new(&host, port, &user, &pass).ping().map_err(|e| friendly_error(&e, &lang));
        post(h, WM_TEST_DONE, result);
    });
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_NCCREATE {
        let cs = &*(lparam as *const CREATESTRUCTW);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, cs.lpCreateParams as isize);
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    let Some(st) = state(hwnd) else {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    };
    match msg {
        WM_CREATE => {
            on_create(hwnd, st);
            0
        }
        WM_COMMAND => {
            match (lo(wparam), hi(wparam)) {
                (IDOK, _) => on_save(hwnd, st),
                (IDCANCEL, _) => {
                    DestroyWindow(hwnd);
                }
                (ID_SEARCH, _) => on_search(st, hwnd),
                (ID_TEST, _) => on_test(st, hwnd),
                (ID_LIST, c) if c == LBN_SELCHANGE as i32 => fill_from_list(st),
                (ID_LANG, c) if c == CBN_SELCHANGE as i32 => {
                    // Vista previa inmediata; si se cancela, se restaura el idioma anterior.
                    i18n::set(selected_language(st));
                    apply_texts(hwnd, st);
                }
                _ => {}
            }
            0
        }
        WM_NOTIFY => {
            let nm = &*(lparam as *const NMHDR);
            if nm.hwndFrom == st.link && (nm.code == NM_CLICK || nm.code == NM_RETURN) {
                ShellExecuteW(null_mut(), wide("open").as_ptr(), wide(TMDB_URL).as_ptr(), null(), null(), SW_SHOWNORMAL);
            }
            0
        }
        WM_SEARCH_DONE => {
            let r = Box::from_raw(lparam as *mut Result<Vec<Found>, String>);
            on_search_done(st, *r);
            0
        }
        WM_TEST_DONE => {
            let r = Box::from_raw(lparam as *mut Result<String, String>);
            EnableWindow(st.btn_test, 1);
            match *r {
                Ok(v) => set_text(st.status, &tf("settings.test_ok", &[("version", &v)])),
                Err(e) => set_text(st.status, &e),
            }
            0
        }
        WM_DESTROY => {
            if !st.saved {
                i18n::set(&st.base.language);
            }
            PostQuitMessage(0);
            0
        }
        WM_NCDESTROY => {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            let st = Box::from_raw(st as *mut State);
            if !st.font.is_null() {
                DeleteObject(st.font as _);
            }
            drop(st);
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
