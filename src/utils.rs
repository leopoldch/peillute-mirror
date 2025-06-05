pub fn get_mac_address() -> Option<String> {
    use pnet::datalink;

    let interfaces = datalink::interfaces();
    for iface in interfaces {
        // Ignore loopback et interfaces sans MAC
        if iface.is_up() && !iface.is_loopback() {
            if let Some(mac) = iface.mac {
                if mac.octets() != [0, 0, 0, 0, 0, 0] {
                    return Some(mac.to_string().replace(":", ""));
                }
            }
        }
    }
    None
}

const FILE_PATH: &str = "local_state.json";

pub async fn reload_existing_site(
    peer_interaction_addr: std::net::SocketAddr,
    nb_sites_on_network: usize,
    peers_addrs: Vec<std::net::SocketAddr>,
) -> bool {
    use crate::state::LOCAL_APP_STATE;
    use log::info;
    use std::fs;

    let data = match fs::read_to_string(FILE_PATH) {
        Ok(content) => content,
        Err(_) => return false,
    };

    let json: serde_json::Value = match serde_json::from_str(&data) {
        Ok(val) => val,
        Err(_) => return false,
    };

    let site_id = match json.get("site_id").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
        None => return false,
    };

    let clock: crate::clock::Clock = match json.get("clock") {
        Some(clock_value) => match serde_json::from_value(clock_value.clone()) {
            Ok(clock) => clock,
            Err(_) => return false,
        },
        None => return false,
    };

    {
        let mut state = LOCAL_APP_STATE.lock().await;
        state.site_id = site_id.clone();
        state.clocks = clock.clone();
        state.local_addr = peer_interaction_addr;
        state.nb_sites_on_network = nb_sites_on_network;
        state.peer_addrs = peers_addrs.clone();
    }

    info!("Existing site state reloaded from {}", FILE_PATH);
    true
}


pub async fn save_local_state(){
    // this is likely to be called whenever the clocks are updated
    use crate::state::LOCAL_APP_STATE;
    use log::info;
    use serde_json::json;

    let (site_id, clock) = {
        let state = LOCAL_APP_STATE.lock().await;
        (
            state.get_site_id().to_string(),
            state.get_clock().clone(),
        )
    };
    
    let json_data = json!({
        "site_id": site_id,
        "clock": clock,
    })
    .to_string();

    std::fs::write(FILE_PATH, json_data).expect("Unable to write file");
    info!("Local state saved to {}", FILE_PATH);

}