use axum::http::HeaderMap;
use tracing::{debug, info, warn};

/// Extract client IP address from headers only (without ConnectInfo)
pub fn extract_client_ip_from_headers(headers: &HeaderMap) -> String {
    debug!("Starting IP extraction from headers");
    
    // Log all headers for debugging
    for (name, value) in headers {
        debug!("Header: {} = {:?}", name, value);
    }
    
    // Check for proxy headers first (for deployments behind reverse proxies)
    debug!("Checking for x-forwarded-for header");
    if let Some(forwarded_for) = headers.get("x-forwarded-for") {
        debug!("Found x-forwarded-for header: {:?}", forwarded_for);
        if let Ok(forwarded_for_str) = forwarded_for.to_str() {
            debug!("x-forwarded-for as string: {}", forwarded_for_str);
            // X-Forwarded-For can contain multiple IPs, take the first one
            if let Some(first_ip) = forwarded_for_str.split(',').next() {
                let trimmed_ip = first_ip.trim();
                debug!("First IP from x-forwarded-for: '{}'", trimmed_ip);
                if !trimmed_ip.is_empty() {
                    info!("Using IP from x-forwarded-for: {}", trimmed_ip);
                    return trimmed_ip.to_string();
                }
            }
        } else {
            warn!("Failed to convert x-forwarded-for header to string");
        }
    } else {
        debug!("No x-forwarded-for header found");
    }
    
    // Check for X-Real-IP header
    debug!("Checking for x-real-ip header");
    if let Some(real_ip) = headers.get("x-real-ip") {
        debug!("Found x-real-ip header: {:?}", real_ip);
        if let Ok(real_ip_str) = real_ip.to_str() {
            let trimmed_ip = real_ip_str.trim();
            debug!("x-real-ip as string: '{}'", trimmed_ip);
            if !trimmed_ip.is_empty() {
                info!("Using IP from x-real-ip: {}", trimmed_ip);
                return trimmed_ip.to_string();
            }
        } else {
            warn!("Failed to convert x-real-ip header to string");
        }
    } else {
        debug!("No x-real-ip header found");
    }
    
    // Check for X-Forwarded-For with different casing
    debug!("Checking for X-Forwarded-For header (uppercase)");
    if let Some(forwarded_for) = headers.get("X-Forwarded-For") {
        debug!("Found X-Forwarded-For header: {:?}", forwarded_for);
        if let Ok(forwarded_for_str) = forwarded_for.to_str() {
            debug!("X-Forwarded-For as string: {}", forwarded_for_str);
            if let Some(first_ip) = forwarded_for_str.split(',').next() {
                let trimmed_ip = first_ip.trim();
                debug!("First IP from X-Forwarded-For: '{}'", trimmed_ip);
                if !trimmed_ip.is_empty() {
                    info!("Using IP from X-Forwarded-For: {}", trimmed_ip);
                    return trimmed_ip.to_string();
                }
            }
        } else {
            warn!("Failed to convert X-Forwarded-For header to string");
        }
    } else {
        debug!("No X-Forwarded-For header found");
    }
    
    // Check for X-Real-IP with different casing
    debug!("Checking for X-Real-IP header (uppercase)");
    if let Some(real_ip) = headers.get("X-Real-IP") {
        debug!("Found X-Real-IP header: {:?}", real_ip);
        if let Ok(real_ip_str) = real_ip.to_str() {
            let trimmed_ip = real_ip_str.trim();
            debug!("X-Real-IP as string: '{}'", trimmed_ip);
            if !trimmed_ip.is_empty() {
                info!("Using IP from X-Real-IP: {}", trimmed_ip);
                return trimmed_ip.to_string();
            }
        } else {
            warn!("Failed to convert X-Real-IP header to string");
        }
    } else {
        debug!("No X-Real-IP header found");
    }
    
    // Fall back to localhost for local development
    debug!("No proxy headers found, falling back to localhost");
    info!("Using fallback IP: 127.0.0.1");
    "127.0.0.1".to_string()
} 