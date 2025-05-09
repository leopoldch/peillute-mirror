// --- Vector Clock ---

#[cfg(feature = "server")]
pub fn increment_vector(state: &mut crate::data::AppState) -> Vec<u64> {
    use std::sync::atomic::Ordering;
    let site_id = state.site_id as usize;
    if site_id < state.vector_clock.len() {
        state.vector_clock[site_id].fetch_add(1, Ordering::SeqCst);
        println!(
            "Site {}: Vector clock incremented at index {}",
            site_id, site_id
        );
    } else {
        tracing::error!(
            "Site {}: Invalid site_id for vector clock increment",
            site_id,
        );
    }
    get_vector_clock(state)
}

#[cfg(feature = "server")]
pub fn update_vector_on_receive(
    state: &mut crate::data::AppState,
    received_vc: &[u64],
) -> Vec<u64> {
    use std::sync::atomic::Ordering;
    let site_id = state.site_id as usize;
    println!(
        "Site {}: Updating vector clock on receive. Received VC: {:?}. Current VC: {:?}",
        site_id,
        received_vc,
        get_vector_clock(state)
    );

    if received_vc.len() != state.vector_clock.len() {
        tracing::warn!(
            "Site {}: Received vector clock of different size ({} vs {})",
            site_id,
            received_vc.len(),
            state.vector_clock.len()
        );
    } else {
        for i in 0..state.nb_sites_on_network {
            if i != site_id {
                let current_val = state.vector_clock[i].load(Ordering::SeqCst);
                let received_val = received_vc[i];
                let max_val = current_val.max(received_val);
                state.vector_clock[i].store(max_val, Ordering::SeqCst);
            }
        }
    }

    // Increment own clock for the receive event AFTER updating from received vector
    if site_id < state.vector_clock.len() {
        use std::sync::atomic::Ordering;
        state.vector_clock[site_id].fetch_add(1, Ordering::SeqCst);
        println!(
            "Site {}: Vector clock incremented at index {} for receive event",
            site_id, site_id
        );
    } else {
        tracing::error!(
            "Site {}: Invalid site_id for vector clock increment post-receive",
            site_id,
        );
    }
    get_vector_clock(state)
}

#[cfg(feature = "server")]
pub fn get_vector_clock(state: &crate::data::AppState) -> Vec<u64> {
    use std::sync::atomic::Ordering;
    state
        .vector_clock
        .iter()
        .map(|a| a.load(Ordering::SeqCst))
        .collect()
}

// --- Lamport Clock ---

#[cfg(feature = "server")]
pub fn increment_lamport_clock(state: &mut crate::data::AppState) -> u64 {
    use std::sync::atomic::Ordering;
    state.lamport_clock.fetch_add(1, Ordering::SeqCst);
    state.lamport_clock.load(Ordering::SeqCst)
}

#[cfg(feature = "server")]
pub fn get_lamport_clock(state: &crate::data::AppState) -> u64 {
    use std::sync::atomic::Ordering;
    state.lamport_clock.load(Ordering::SeqCst)
}
