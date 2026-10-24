// En release no abre consola; en debug se conserva para ver el log.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod autostart;
mod config;
mod discord_ipc;
mod discovery;
mod i18n;
mod kodi;
mod logger;
mod presence;
mod settings;
mod single_instance;
mod tmdb;
mod tray;
mod worker;

fn main() {
    logger::init();
    // Si ya hay una Butaca corriendo, se le pide abrir la configuración y esta se cierra.
    let Some(instance) = single_instance::acquire() else {
        return;
    };
    tray::run(instance);
}
