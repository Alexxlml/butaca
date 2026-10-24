//! Descubrimiento de Kodi en la red local (mDNS/Zeroconf: `_xbmc-jsonrpc-h._tcp.local.`).
//! En Kodi hay que activar "Anunciar estos servicios a otros sistemas" (Servicios → Control).

use mdns_sd::{ServiceDaemon, ServiceEvent};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

const SERVICE: &str = "_xbmc-jsonrpc-h._tcp.local.";

#[derive(Clone, Debug)]
pub struct Found {
    pub name: String,
    pub host: String,
    pub port: u16,
}

/// Escucha unos segundos y devuelve los Kodi encontrados.
pub fn search(timeout: Duration) -> Result<Vec<Found>, String> {
    let daemon = ServiceDaemon::new().map_err(|e| e.to_string())?;
    let rx = daemon.browse(SERVICE).map_err(|e| e.to_string())?;
    let end = Instant::now() + timeout;
    let mut found: BTreeMap<String, Found> = BTreeMap::new();

    while let Some(left) = end.checked_duration_since(Instant::now()) {
        match rx.recv_timeout(left) {
            Ok(ServiceEvent::ServiceResolved(info)) => {
                // Se prefiere IPv4: es lo que se escribe a mano en el campo.
                let addr = info
                    .get_addresses()
                    .iter()
                    .map(|a| a.to_ip_addr())
                    .find(|a| a.is_ipv4())
                    .or_else(|| info.get_addresses().iter().map(|a| a.to_ip_addr()).next());
                if let Some(addr) = addr {
                    let full = info.get_fullname();
                    let name = full.strip_suffix(&format!(".{SERVICE}")).unwrap_or(full).to_string();
                    found.insert(full.to_string(), Found { name, host: addr.to_string(), port: info.get_port() });
                }
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }
    let _ = daemon.shutdown();
    Ok(found.into_values().collect())
}
