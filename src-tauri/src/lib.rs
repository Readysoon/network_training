// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::net::{Ipv4Addr, TcpStream, TcpListener};
use std::thread;
use std::time::Duration;
use std::process::Command;
use std::io::{Read, Write};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Get own IP address
#[tauri::command]
fn get_own_ip() -> String {
    let (command, args) = if cfg!(target_os = "windows") {
        ("ipconfig", &[] as &[&str])
    } else {
        ("hostname", &["-I"] as &[&str])
    };
    
    let output = Command::new(command).args(args).output();
    
    match output {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            if cfg!(target_os = "windows") {
                // Windows: ipconfig
                for line in output_str.lines() {
                    if line.contains("IPv4") && line.contains("192.168") {
                        if let Some(ip_part) = line.split(':').nth(1) {
                            let ip = ip_part.trim();
                            if let Ok(_) = ip.parse::<Ipv4Addr>() {
                                return ip.to_string();
                            }
                        }
                    }
                }
            } else {
                // Linux/Mac: hostname -I
                for line in output_str.lines() {
                    for part in line.split_whitespace() {
                        if part.contains("192.168") {
                            if let Ok(_) = part.parse::<Ipv4Addr>() {
                                return part.to_string();
                            }
                        }
                    }
                }
            }
        }
        Err(_) => {}
    }
    
    // Fallback
    "192.168.178.98".to_string()
}

// Test if an IP has the Tauri app
#[tauri::command]
fn test_tauri_app(ip: String) -> bool {
    println!("🔍 Testing {} for Tauri app...", ip);
    
    match TcpStream::connect_timeout(
        &format!("{}:54321", ip).parse().unwrap(),
        Duration::from_millis(1000)
    ) {
        Ok(mut stream) => {
            println!("   ✅ Connected to {}:54321", ip);
            
            // Send ping message
            let message = "PING_TAURI_APP\n";
            match stream.write_all(message.as_bytes()) {
                Ok(_) => {
                    println!("   📤 Sent ping to {}", ip);
                    
                    // Wait for response
                    let mut response = [0; 100];
                    match stream.read(&mut response) {
                        Ok(_) => {
                            let response_str = String::from_utf8_lossy(&response);
                            println!("   📥 Received: '{}'", response_str.trim());
                            
                            if response_str.contains("TAURI_APP_HERE") {
                                println!("   🤝 Tauri app found at {}", ip);
                                true
                            } else {
                                println!("   ❌ No Tauri app response from {}", ip);
                                false
                            }
                        }
                        Err(e) => {
                            println!("   ❌ Could not read response from {}: {}", ip, e);
                            false
                        }
                    }
                }
                Err(e) => {
                    println!("   ❌ Could not send ping to {}: {}", ip, e);
                    false
                }
            }
        }
        Err(e) => {
            println!("   ❌ Could not connect to {}:54321 - {}", ip, e);
            false
        }
    }
}

// Start the Tauri listener
fn start_tauri_listener() {
    thread::spawn(|| {
        if let Ok(listener) = TcpListener::bind("0.0.0.0:54321") {
            println!("👂 Tauri app listener started on port 54321");
            
            for stream in listener.incoming() {
                if let Ok(mut stream) = stream {
                    let mut buffer = [0; 100];
                    if let Ok(_) = stream.read(&mut buffer) {
                        let message = String::from_utf8_lossy(&buffer);
                        
                        if message.contains("PING_TAURI_APP") {
                            println!("🤝 Received ping request, responding...");
                            let response = "TAURI_APP_HERE\n";
                            let _ = stream.write_all(response.as_bytes());
                        }
                    }
                }
            }
        }
    });
}

// Main Tauri function
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Start the listener
    start_tauri_listener();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_own_ip, test_tauri_app])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
