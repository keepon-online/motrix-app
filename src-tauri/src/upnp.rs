//! UPnP port mapping for BT and DHT ports

use std::net::SocketAddrV4;
use std::sync::Mutex;

static MAPPED_PORTS: Mutex<Vec<(u16, igd_next::PortMappingProtocol)>> = Mutex::new(Vec::new());

/// Map BT (TCP) and DHT (UDP) ports via UPnP
pub async fn map_ports(bt_port: u16, dht_port: u16) {
    let local_ip = match get_local_ip() {
        Some(ip) => ip,
        None => {
            tracing::warn!("UPnP: Could not determine local IP address");
            return;
        }
    };

    let gateway = match igd_next::aio::tokio::search_gateway(Default::default()).await {
        Ok(gw) => gw,
        Err(e) => {
            tracing::warn!("UPnP: Gateway discovery failed: {}", e);
            return;
        }
    };

    // Map BT port (TCP)
    let bt_addr = std::net::SocketAddr::V4(SocketAddrV4::new(local_ip, bt_port));
    match gateway
        .add_port(
            igd_next::PortMappingProtocol::TCP,
            bt_port,
            bt_addr,
            3600,
            "Motrix BT",
        )
        .await
    {
        Ok(()) => {
            tracing::info!("UPnP: Mapped TCP port {} (BT)", bt_port);
            if let Ok(mut ports) = MAPPED_PORTS.lock() {
                ports.push((bt_port, igd_next::PortMappingProtocol::TCP));
            }
        }
        Err(e) => tracing::warn!("UPnP: Failed to map TCP port {}: {}", bt_port, e),
    }

    // Map DHT port (UDP)
    let dht_addr = std::net::SocketAddr::V4(SocketAddrV4::new(local_ip, dht_port));
    match gateway
        .add_port(
            igd_next::PortMappingProtocol::UDP,
            dht_port,
            dht_addr,
            3600,
            "Motrix DHT",
        )
        .await
    {
        Ok(()) => {
            tracing::info!("UPnP: Mapped UDP port {} (DHT)", dht_port);
            if let Ok(mut ports) = MAPPED_PORTS.lock() {
                ports.push((dht_port, igd_next::PortMappingProtocol::UDP));
            }
        }
        Err(e) => tracing::warn!("UPnP: Failed to map UDP port {}: {}", dht_port, e),
    }
}

/// Remove previously mapped ports
pub async fn unmap_ports() {
    let ports = match MAPPED_PORTS.lock() {
        Ok(mut guard) => std::mem::take(&mut *guard),
        Err(_) => return,
    };

    if ports.is_empty() {
        return;
    }

    let gateway = match igd_next::aio::tokio::search_gateway(Default::default()).await {
        Ok(gw) => gw,
        Err(e) => {
            tracing::warn!("UPnP: Gateway discovery failed during cleanup: {}", e);
            return;
        }
    };

    for (port, protocol) in ports {
        match gateway.remove_port(protocol, port).await {
            Ok(()) => tracing::info!("UPnP: Unmapped {:?} port {}", protocol, port),
            Err(e) => tracing::warn!("UPnP: Failed to unmap {:?} port {}: {}", protocol, port, e),
        }
    }
}

/// Get local IPv4 address
fn get_local_ip() -> Option<std::net::Ipv4Addr> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    match socket.local_addr().ok()? {
        std::net::SocketAddr::V4(addr) => Some(*addr.ip()),
        _ => None,
    }
}
