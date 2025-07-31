# 🌐 Network Medical Database

A Tauri-based application that creates a distributed medical patient database using GunDB for real-time synchronization across multiple network devices.

## ✨ Features

### 🔍 Network Discovery
- **Auto IP Detection**: Automatically detects your local IP address
- **Network Scanning**: Test and connect to other devices running the same application
- **Connection Status**: Real-time status of Tauri app detection and GunDB connections

### 👥 Patient Database
- **Distributed Database**: Uses GunDB for decentralized, real-time patient data synchronization
- **Multi-Device Sync**: Patient records sync automatically across all connected devices
- **CRUD Operations**: Add, view, and remove patient records
- **Real-time Updates**: Changes appear instantly on all connected devices

### 🔄 Synchronization
- **Peer-to-Peer**: Direct connections between devices without central server
- **Auto-Discovery**: Automatically connects to GunDB peers on successfully tested IPs
- **Sync Status**: Visual indicators showing connection status and number of active peers
- **Offline Capable**: Works locally and syncs when connections are restored

## 🚀 Quick Start

### Prerequisites
- Node.js (v16 or higher)
- Rust and Cargo
- Tauri CLI

### Installation
```bash
# Clone the repository
git clone <your-repo-url>
cd network_training

# Install dependencies
npm install

# Run the application
npm run tauri dev
```

### Usage

1. **Start the Application**: Launch the app on multiple devices in the same network
2. **Check Your IP**: Your local IP address will be displayed at the top
3. **Connect to Peers**: 
   - Enter the IP addresses of other devices running the app
   - Click "Test & Connect" to establish both Tauri and GunDB connections
4. **Manage Patients**:
   - Add new patients using the form (Name, Age, Diagnosis)
   - View all patients in the synchronized database
   - Remove patients using the × button
5. **Monitor Sync**: Watch the sync status to see how many peers you're connected to

## 🏗️ Architecture

### Frontend (Svelte)
- **UI Components**: Modern, responsive interface with real-time updates
- **GunDB Integration**: Client-side database with automatic synchronization
- **State Management**: Reactive Svelte stores for UI updates

### Backend (Rust/Tauri)
- **Network Detection**: Cross-platform IP address detection
- **Peer Discovery**: TCP-based discovery on port 54321
- **API Bridge**: Secure communication between frontend and system

### Database (GunDB)
- **Distributed**: No central server required
- **Real-time**: Automatic synchronization across peers
- **Conflict Resolution**: Built-in handling of concurrent updates
- **Schema**: Simple patient records with ID, name, age, diagnosis, and timestamp

## 🔧 Technical Details

### Ports Used
- **54321**: Tauri app discovery and handshake
- **8765**: GunDB peer-to-peer communication (default)

### Data Structure
```typescript
interface Patient {
  id: string;
  name: string;
  age: number;
  diagnosis: string;
  timestamp: string;
}
```

### Network Flow
1. Device A scans for Device B using Tauri discovery protocol
2. If Device B responds with Tauri handshake, GunDB connection is attempted
3. GunDB establishes WebRTC/WebSocket connection for database sync
4. Patient data automatically synchronizes between all connected peers

## 🛡️ Security Considerations
- All connections are within the local network
- No external internet dependencies for core functionality
- Patient data remains on local network devices
- Consider implementing encryption for sensitive medical data in production

## 🔮 Future Enhancements
- [ ] Patient search and filtering
- [ ] Data encryption for sensitive information
- [ ] Export/import functionality
- [ ] User authentication and access control
- [ ] Advanced medical record fields
- [ ] Backup and restore capabilities

## 🤝 Contributing
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## 📝 License
MIT License - see LICENSE file for details
