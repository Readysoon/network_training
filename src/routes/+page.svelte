<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import Gun from "gun";

  // Types
  interface TestResult {
    ip: string;
    hasApp: boolean;
    error?: boolean;
    timestamp: string;
    gunConnected?: boolean;
  }

  interface Patient {
    id: string;
    name: string;
    age: number;
    diagnosis: string;
    timestamp: string;
  }

  interface ConnectedPeer {
    ip: string;
    connected: boolean;
    lastSeen: string;
  }

  // State variables
  let ownIp = "";
  let newIp = "";
  let testResults: TestResult[] = [];
  let isLoading = false;

  // GunDB variables
  let gun: any;
  let patients: Patient[] = [];
  let connectedPeers: ConnectedPeer[] = [];
  let syncStatus = "Initializing...";

  // New patient form
  let newPatient = {
    name: "",
    age: 0,
    diagnosis: ""
  };

  // Initialize GunDB
  function initGun() {
    gun = Gun(['http://localhost:8765']); // Default Gun peer
    
    // Listen for patients
    gun.get('patients').map().on((data: any, key: string) => {
      if (data && key !== '_') {
        const patient: Patient = {
          id: key,
          name: data.name || '',
          age: data.age || 0,
          diagnosis: data.diagnosis || '',
          timestamp: data.timestamp || ''
        };
        
        // Update patients list
        patients = patients.filter(p => p.id !== key);
        if (data.name) { // Only add if not deleted
          patients = [...patients, patient];
        }
      }
    });

    syncStatus = "Ready - Local only";
  }

  onMount(async () => {
    try {
      ownIp = await invoke("get_own_ip") as string;
    } catch (error) {
      console.error("Failed to get own IP:", error);
      ownIp = "Error getting IP";
    }

    // Initialize GunDB
    initGun();
  });

  async function testIp() {
    if (!newIp.trim()) return;
    
    isLoading = true;
    try {
      const hasTauriApp = await invoke("test_tauri_app", { ip: newIp.trim() }) as boolean;
      
      const result: TestResult = {
        ip: newIp.trim(),
        hasApp: hasTauriApp,
        timestamp: new Date().toLocaleTimeString(),
        gunConnected: false
      };

      // If Tauri app found, try to connect to GunDB
      if (hasTauriApp) {
        result.gunConnected = await connectToGunPeer(newIp.trim());
      }
      
      testResults = [...testResults, result];
      newIp = ""; // Clear input
    } catch (error) {
      console.error("Failed to test IP:", error);
      testResults = [
        ...testResults,
        {
          ip: newIp.trim(),
          hasApp: false,
          error: true,
          timestamp: new Date().toLocaleTimeString()
        }
      ];
    } finally {
      isLoading = false;
    }
  }

  async function connectToGunPeer(ip: string): Promise<boolean> {
    try {
      // Add peer to Gun network
      const peerUrl = `http://${ip}:8765`;
      gun.opt({ peers: [peerUrl] });
      
      // Add to connected peers
      const peer: ConnectedPeer = {
        ip: ip,
        connected: true,
        lastSeen: new Date().toLocaleTimeString()
      };
      
      connectedPeers = connectedPeers.filter(p => p.ip !== ip);
      connectedPeers = [...connectedPeers, peer];
      
      updateSyncStatus();
      return true;
    } catch (error) {
      console.error("Failed to connect to Gun peer:", error);
      return false;
    }
  }

  function updateSyncStatus() {
    const activeConnections = connectedPeers.filter(p => p.connected).length;
    if (activeConnections === 0) {
      syncStatus = "Ready - Local only";
    } else {
      syncStatus = `Syncing with ${activeConnections} peer${activeConnections === 1 ? '' : 's'}`;
    }
  }

  function addPatient() {
    if (!newPatient.name.trim() || !newPatient.diagnosis.trim()) return;

    const patient: Patient = {
      id: Date.now().toString(),
      name: newPatient.name.trim(),
      age: newPatient.age,
      diagnosis: newPatient.diagnosis.trim(),
      timestamp: new Date().toLocaleString()
    };

    // Add to GunDB
    gun.get('patients').get(patient.id).put({
      name: patient.name,
      age: patient.age,
      diagnosis: patient.diagnosis,
      timestamp: patient.timestamp
    });

    // Clear form
    newPatient = {
      name: "",
      age: 0,
      diagnosis: ""
    };
  }

  function removeResult(index: number) {
    testResults = testResults.filter((_, i) => i !== index);
  }

  function disconnectPeer(ip: string) {
    connectedPeers = connectedPeers.map(peer => 
      peer.ip === ip ? { ...peer, connected: false } : peer
    );
    updateSyncStatus();
  }

  function removePatient(patientId: string) {
    // Remove from GunDB (Gun uses null to delete)
    gun.get('patients').get(patientId).put(null);
  }
</script>

<main>
  <div class="container">
    <h1>🌐 Network Medical Database</h1>
    
    <div class="own-ip">
      <h2>🏠 Your IP Address:</h2>
      <div class="ip-display">{ownIp}</div>
    </div>

    <div class="sync-status">
      <h2>🔄 Database Status:</h2>
      <div class="status-display {connectedPeers.filter(p => p.connected).length > 0 ? 'syncing' : 'local'}">{syncStatus}</div>
    </div>

    <div class="test-section">
      <h2>🔍 Connect to Other PCs</h2>
      <div class="input-group">
        <input 
          type="text" 
          bind:value={newIp} 
          placeholder="Enter IP address (e.g., 192.168.178.114)"
          on:keydown={(e) => e.key === 'Enter' && testIp()}
        />
        <button on:click={testIp} disabled={isLoading || !newIp.trim()}>
          {isLoading ? 'Testing...' : 'Test & Connect'}
        </button>
      </div>
    </div>

    <div class="results">
      <h2>📊 Connection Results</h2>
      {#if testResults.length === 0}
        <p class="no-results">No connections tested yet.</p>
      {:else}
        <div class="result-list">
          {#each testResults as result, index}
            <div class="result-item {result.hasApp ? 'success' : 'error'}">
              <div class="result-info">
                <span class="ip">{result.ip}</span>
                <span class="status">
                  {#if result.error}
                    ❌ Connection failed
                  {:else if result.hasApp && result.gunConnected}
                    ✅ Connected to database
                  {:else if result.hasApp}
                    ⚠️ Tauri app found, database connection failed
                  {:else}
                    ❌ No Tauri app
                  {/if}
                </span>
                <span class="time">{result.timestamp}</span>
              </div>
              <button class="remove-btn" on:click={() => removeResult(index)}>×</button>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    {#if connectedPeers.filter(p => p.connected).length > 0}
      <div class="connected-peers">
        <h2>🔗 Connected Database Peers</h2>
        <div class="peer-list">
          {#each connectedPeers.filter(p => p.connected) as peer}
            <div class="peer-item">
              <span class="peer-ip">{peer.ip}</span>
              <span class="peer-status">Last seen: {peer.lastSeen}</span>
              <button class="disconnect-btn" on:click={() => disconnectPeer(peer.ip)}>Disconnect</button>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <div class="patients-section">
      <h2>👥 Patient Database</h2>
      
      <div class="add-patient">
        <h3>➕ Add New Patient</h3>
        <div class="patient-form">
          <input 
            type="text" 
            bind:value={newPatient.name} 
            placeholder="Patient name"
          />
          <input 
            type="number" 
            bind:value={newPatient.age} 
            placeholder="Age"
            min="0"
          />
          <input 
            type="text" 
            bind:value={newPatient.diagnosis} 
            placeholder="Diagnosis"
          />
          <button on:click={addPatient} disabled={!newPatient.name.trim() || !newPatient.diagnosis.trim()}>
            Add Patient
          </button>
        </div>
      </div>

      <div class="patient-list">
        <h3>📋 Current Patients ({patients.length})</h3>
        {#if patients.length === 0}
          <p class="no-patients">No patients in database.</p>
        {:else}
          <div class="patients">
            {#each patients as patient}
              <div class="patient-item">
                <div class="patient-info">
                  <div class="patient-name">{patient.name}</div>
                  <div class="patient-details">Age: {patient.age} | {patient.diagnosis}</div>
                  <div class="patient-time">Added: {patient.timestamp}</div>
                </div>
                <button class="remove-patient-btn" on:click={() => removePatient(patient.id)}>×</button>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
</main>

<style>
  .container {
    max-width: 1000px;
    margin: 0 auto;
    padding: 20px;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  }

  h1 {
    text-align: center;
    color: #333;
    margin-bottom: 30px;
  }

  h2 {
    color: #555;
    margin-bottom: 15px;
  }

  h3 {
    color: #666;
    margin-bottom: 10px;
  }

  .own-ip, .sync-status {
    background: #f8f9fa;
    padding: 20px;
    border-radius: 8px;
    margin-bottom: 20px;
  }

  .ip-display, .status-display {
    font-size: 1.2em;
    font-weight: bold;
    text-align: center;
  }

  .ip-display {
    color: #007bff;
  }

  .status-display.local {
    color: #6c757d;
  }

  .status-display.syncing {
    color: #28a745;
  }

  .test-section {
    margin-bottom: 30px;
  }

  .input-group {
    display: flex;
    gap: 10px;
  }

  input {
    flex: 1;
    padding: 12px;
    border: 2px solid #ddd;
    border-radius: 6px;
    font-size: 16px;
  }

  input:focus {
    outline: none;
    border-color: #007bff;
  }

  button {
    padding: 12px 20px;
    background: #007bff;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 16px;
  }

  button:hover:not(:disabled) {
    background: #0056b3;
  }

  button:disabled {
    background: #ccc;
    cursor: not-allowed;
  }

  .results, .connected-peers, .patients-section {
    background: #f8f9fa;
    padding: 20px;
    border-radius: 8px;
    margin-bottom: 20px;
  }

  .no-results, .no-patients {
    text-align: center;
    color: #666;
    font-style: italic;
  }

  .result-list, .peer-list, .patients {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .result-item, .peer-item, .patient-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 15px;
    border-radius: 6px;
    background: white;
  }

  .result-item {
    border-left: 4px solid;
  }

  .result-item.success {
    border-left-color: #28a745;
  }

  .result-item.error {
    border-left-color: #dc3545;
  }

  .result-info, .patient-info {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .ip, .patient-name {
    font-weight: bold;
    font-size: 1.1em;
  }

  .status, .patient-details {
    font-size: 0.9em;
  }

  .time, .patient-time, .peer-status {
    font-size: 0.8em;
    color: #666;
  }

  .remove-btn, .remove-patient-btn {
    background: #dc3545;
    color: white;
    border: none;
    border-radius: 50%;
    width: 30px;
    height: 30px;
    cursor: pointer;
    font-size: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .remove-btn:hover, .remove-patient-btn:hover {
    background: #c82333;
  }

  .disconnect-btn {
    background: #ffc107;
    color: #212529;
    font-size: 14px;
    padding: 8px 12px;
  }

  .disconnect-btn:hover {
    background: #e0a800;
  }

  .patient-form {
    display: grid;
    grid-template-columns: 2fr 1fr 2fr 1fr;
    gap: 10px;
    margin-bottom: 20px;
  }

  .add-patient {
    margin-bottom: 20px;
    padding-bottom: 20px;
    border-bottom: 1px solid #dee2e6;
  }

  @media (max-width: 768px) {
    .patient-form {
      grid-template-columns: 1fr;
    }
    
    .input-group {
      flex-direction: column;
    }
  }
</style>
