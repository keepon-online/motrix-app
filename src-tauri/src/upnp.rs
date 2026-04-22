use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::LazyLock;

use igd_next::aio::tokio::{self as igd_tokio, Tokio};
use igd_next::{PortMappingProtocol, SearchOptions};
use serde::Serialize;
use tokio::sync::RwLock;
use tracing;

static UPNP_ENABLED: AtomicBool = AtomicBool::new(false);

static UPNP_STATE: LazyLock<RwLock<UpnpState>> =
    LazyLock::new(|| RwLock::new(UpnpState::new()));

struct UpnpState {
    gateway: Option<igd_next::aio::Gateway<Tokio>>,
    external_ip: Option<String>,
    mapped: Vec<MappedPort>,
}

struct MappedPort {
    port: u16,
    protocol: PortMappingProtocol,
    description: String,
}

impl UpnpState {
    fn new() -> Self {
        Self {
            gateway: None,
            external_ip: None,
            mapped: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpnpPortMapping {
    pub port: u16,
    pub protocol: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpnpStatus {
    pub enabled: bool,
    pub gateway_found: bool,
    pub external_ip: Option<String>,
    pub mapped_ports: Vec<UpnpPortMapping>,
}

pub fn set_enabled(enabled: bool) {
    UPNP_ENABLED.store(enabled, Ordering::SeqCst);
}

pub async fn map_ports(bt_port: u16, dht_port: u16) {
    if !UPNP_ENABLED.load(Ordering::SeqCst) {
        return;
    }

    tracing::info!("Starting UPnP port mapping (BT: {}, DHT: {})", bt_port, dht_port);

    // Unmap existing before mapping new ones
    unmap_all_inner().await;

    let gateway = match discover_gateway().await {
        Some(gw) => gw,
        None => return,
    };

    let mut state = UPNP_STATE.write().await;

    // Get external IP
    match gateway.get_external_ip().await {
        Ok(ip) => {
            tracing::info!("UPnP external IP: {}", ip);
            state.external_ip = Some(ip.to_string());
        }
        Err(e) => {
            tracing::warn!("Failed to get external IP via UPnP: {}", e);
            state.external_ip = None;
        }
    }

    let local_addr = "0.0.0.0:0".parse::<SocketAddr>().unwrap();
    let lease_duration = 3600u32; // 1 hour, will be re-mapped on engine restart

    let mappings: Vec<(PortMappingProtocol, u16, &str)> = vec![
        (PortMappingProtocol::TCP, bt_port, "Motrix BT TCP"),
        (PortMappingProtocol::UDP, bt_port, "Motrix BT UDP"),
        (PortMappingProtocol::UDP, dht_port, "Motrix DHT UDP"),
    ];

    for (protocol, port, description) in &mappings {
        match gateway
            .add_port(*protocol, *port, local_addr, lease_duration, description)
            .await
        {
            Ok(()) => {
                tracing::info!("UPnP mapped {} port {} ({})", proto_str(protocol), port, description);
                state.mapped.push(MappedPort {
                    port: *port,
                    protocol: *protocol,
                    description: description.to_string(),
                });
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to map {} port {} via UPnP: {}",
                    proto_str(protocol),
                    port,
                    e
                );
            }
        }
    }

    state.gateway = Some(gateway);
}

pub async fn unmap_all() {
    unmap_all_inner().await;
}

async fn unmap_all_inner() {
    let mut state = UPNP_STATE.write().await;

    if state.mapped.is_empty() {
        return;
    }

    let gateway = match &state.gateway {
        Some(gw) => gw.clone(),
        None => {
            state.mapped.clear();
            return;
        }
    };

    for mp in &state.mapped {
        match gateway.remove_port(mp.protocol, mp.port).await {
            Ok(()) => {
                tracing::info!("UPnP unmapped {} port {}", proto_str(&mp.protocol), mp.port);
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to unmap {} port {} via UPnP: {}",
                    proto_str(&mp.protocol),
                    mp.port,
                    e
                );
            }
        }
    }

    state.mapped.clear();
}

pub async fn get_status() -> UpnpStatus {
    let enabled = UPNP_ENABLED.load(Ordering::SeqCst);
    let state = UPNP_STATE.read().await;

    UpnpStatus {
        enabled,
        gateway_found: state.gateway.is_some(),
        external_ip: state.external_ip.clone(),
        mapped_ports: state
            .mapped
            .iter()
            .map(|mp| UpnpPortMapping {
                port: mp.port,
                protocol: proto_str(&mp.protocol).to_string(),
                description: mp.description.clone(),
            })
            .collect(),
    }
}

async fn discover_gateway() -> Option<igd_next::aio::Gateway<Tokio>> {
    tracing::info!("Searching for UPnP gateway...");

    let options = SearchOptions::default();
    match igd_tokio::search_gateway(options).await {
        Ok(gateway) => {
            tracing::info!("UPnP gateway found at {}", gateway);
            Some(gateway)
        }
        Err(e) => {
            tracing::warn!("No UPnP gateway found: {}", e);
            None
        }
    }
}

fn proto_str(protocol: &PortMappingProtocol) -> &'static str {
    match protocol {
        PortMappingProtocol::TCP => "TCP",
        PortMappingProtocol::UDP => "UDP",
    }
}
