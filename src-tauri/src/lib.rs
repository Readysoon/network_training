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

// Des isch unser Hauptfunktion - die macht alles
fn scan_network() {
    println!("🔍 Starting Tauri app detection...");
    
    // Erst mol alle Geräte im Netzwerk finde
    let devices = find_all_devices();
    println!("📊 Found {} devices", devices.len());
    
    // Jetzt schaue, welche von de Geräte unser Tauri App han
    println!("🤝 Checking for Tauri apps...");
    let tauri_apps = find_tauri_apps(&devices);
    
    println!("✅ Found {} Tauri apps:", tauri_apps.len());
    for app in &tauri_apps {
        println!("   🖥️  {}", app);
    }
}

// Die Funktion pingt a Gerät und schaut, ob es antwortet
fn ping_host(ip: &Ipv4Addr) -> bool {
    let (command, args) = if cfg!(target_os = "windows") {
        ("ping", &["-n", "1", "-w", "50", &ip.to_string()] as &[&str])
    } else {
        ("ping", &["-c", "1", "-W", "1", &ip.to_string()] as &[&str])
    };
    
    let output = Command::new(command).args(args).output();
    
    match output {
        Ok(output) => output.status.success(), // Wenn ping erfolgreich war
        Err(_) => false
    }
}

// Die Funktion findt alle Geräte im Netzwerk (schnell!)
fn find_all_devices() -> Vec<Ipv4Addr> {
    let mut devices = Vec::new();
    
    // Gucke, welches Netzwerk wir han (192.168.178.x)
    if let Some(network) = get_local_network() {
        println!("🌐 Scanning {}.{}.{}.x network...", network.0, network.1, network.2);
        
        // Finde unser eigene IP
        let own_ip = get_own_ip();
        println!("🏠 Own IP: {}", own_ip);
        
        // Erstelle alle IPs zum Testen
        let mut ips_to_test = Vec::new();
        for i in 1..=254 {
            let ip = Ipv4Addr::new(network.0, network.1, network.2, i);
            if ip != own_ip {
                ips_to_test.push(ip);
            }
        }
        
        println!("🚀 Testing {} IPs in parallel...", ips_to_test.len());
        
        // Teste alle IPs parallel mit Threads
        let mut handles = Vec::new();
        for ip in ips_to_test {
            let handle = thread::spawn(move || {
                if ping_host(&ip) {
                    Some(ip)
                } else {
                    None
                }
            });
            handles.push(handle);
        }
        
        // Sammle alle Ergebnisse
        for handle in handles {
            if let Ok(Some(ip)) = handle.join() {
                println!("   ✅ Found device at {}", ip);
                devices.push(ip);
            }
        }
    }
    
    devices
}

// Die Funktion findt unser eigene IP
fn get_own_ip() -> Ipv4Addr {
    // Gucke, welches Betriebssystem wir han
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
                            if let Ok(parsed_ip) = ip.parse::<Ipv4Addr>() {
                                return parsed_ip;
                            }
                        }
                    }
                }
            } else {
                // Linux/Mac: hostname -I
                for line in output_str.lines() {
                    for part in line.split_whitespace() {
                        if part.contains("192.168") {
                            if let Ok(parsed_ip) = part.parse::<Ipv4Addr>() {
                                return parsed_ip;
                            }
                        }
                    }
                }
            }
        }
        Err(_) => {}
    }
    
    // Fallback: Default IP
    Ipv4Addr::new(192, 168, 178, 98)
}


// Die Funktion schaut, welche Geräte unser Tauri App han
fn find_tauri_apps(devices: &[Ipv4Addr]) -> Vec<String> {
    let mut tauri_apps = Vec::new();
    
    for device in devices {
        println!("🔍 Testing device {} for Tauri app...", device);
        
        // Versuche, zum Port 54321 zu verbinde (des isch unser spezieller Port)
        match TcpStream::connect_timeout(
            &format!("{}:54321", device).parse().unwrap(),
            Duration::from_millis(500) // Länger warten
        ) {
            Ok(mut stream) => {
                println!("   ✅ Connected to {}:54321", device);
                
                // Schick a "PING_TAURI_APP" Nachricht
                let message = "PING_TAURI_APP\n";
                match stream.write_all(message.as_bytes()) {
                    Ok(_) => {
                        println!("   📤 Sent ping to {}", device);
                        
                        // Warte auf Antwort
                        let mut response = [0; 100];
                        match stream.read(&mut response) {
                            Ok(_) => {
                                let response_str = String::from_utf8_lossy(&response);
                                println!("   📥 Received: '{}'", response_str.trim());
                                
                                // Wenn die Antwort "TAURI_APP_HERE" enthält, dann han die unser App
                                if response_str.contains("TAURI_APP_HERE") {
                                    println!("   🤝 Tauri app found at {}", device);
                                    tauri_apps.push(format!("Tauri app at {}", device));
                                } else {
                                    println!("   ❌ No Tauri app response from {}", device);
                                }
                            }
                            Err(e) => {
                                println!("   ❌ Could not read response from {}: {}", device, e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("   ❌ Could not send ping to {}: {}", device, e);
                    }
                }
            }
            Err(e) => {
                println!("   ❌ Could not connect to {}:54321 - {}", device, e);
            }
        }
    }
    
    tauri_apps
}

// Die Funktion findt raus, welches Netzwerk wir han (192.168.178.x)
fn get_local_network() -> Option<(u8, u8, u8)> {
    let own_ip = get_own_ip();
    
    // Von unser eigene IP das Netzwerk ableite
    let octets = own_ip.octets();
    Some((octets[0], octets[1], octets[2]))
}


// Des isch die Hauptfunktion von Tauri - die startet alles
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Starte den Listener (damit unser App auf Pings antworten kann)
    start_tauri_listener();
    
    // Starte die Netzwerk-Suche in a separaten Thread
    thread::spawn(|| {
        thread::sleep(Duration::from_millis(1000));
        scan_network();
    });
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Die Funktion startet a Listener, der auf Pings wartet
fn start_tauri_listener() {
    thread::spawn(|| {
        // Höre auf Port 54321 (des isch unser spezieller Port)
        if let Ok(listener) = TcpListener::bind("0.0.0.0:54321") {
            println!("👂 Tauri app listener started on port 54321");
            
            // Für jede Verbindung, die kommt
            for stream in listener.incoming() {
                if let Ok(mut stream) = stream {
                    // Lese die Nachricht
                    let mut buffer = [0; 100];
                    if let Ok(_) = stream.read(&mut buffer) {
                        let message = String::from_utf8_lossy(&buffer);
                        
                        // Wenn es a "PING_TAURI_APP" Nachricht isch, antworte
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
