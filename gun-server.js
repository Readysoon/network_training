// Simple GunDB server for medical database sync
const Gun = require('gun');

// Start Gun server on port 8765
const server = require('http').createServer();
const gun = Gun({
  web: server,
  peers: ['http://localhost:8765'] // Allow self as peer
});

console.log('🏥 GunDB Medical Database Server running on port 8765');
console.log('📡 Ready to sync with other peers...');

server.listen(8765, () => {
  console.log('✅ Server started successfully!');
  console.log('🌐 Other apps can connect to: http://localhost:8765');
}); 