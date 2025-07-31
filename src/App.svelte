<script>
  import { invoke } from "@tauri-apps/api/tauri";
  import { onMount } from "svelte";

  let ownIp = "";
  let newIp = "";
  let testResults = [];
  let isLoading = false;

  onMount(async () => {
    try {
      ownIp = await invoke("get_own_ip");
    } catch (error) {
      console.error("Failed to get own IP:", error);
      ownIp = "Error getting IP";
    }
  });

  async function testIp() {
    if (!newIp.trim()) return;
    
    isLoading = true;
    try {
      const hasTauriApp = await invoke("test_tauri_app", { ip: newIp.trim() });
      
      testResults = [
        ...testResults,
        {
          ip: newIp.trim(),
          hasApp: hasTauriApp,
          timestamp: new Date().toLocaleTimeString()
        }
      ];
      
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

  function removeResult(index) {
    testResults = testResults.filter((_, i) => i !== index);
  }
</script>

<main>
  <div class="container">
    <h1>🌐 Network Tauri App Scanner</h1>
    
    <div class="own-ip">
      <h2>🏠 Your IP Address:</h2>
      <div class="ip-display">{ownIp}</div>
    </div>

    <div class="test-section">
      <h2>🔍 Test Other PCs</h2>
      <div class="input-group">
        <input 
          type="text" 
          bind:value={newIp} 
          placeholder="Enter IP address (e.g., 192.168.178.114)"
          on:keydown={(e) => e.key === 'Enter' && testIp()}
        />
        <button on:click={testIp} disabled={isLoading || !newIp.trim()}>
          {isLoading ? 'Testing...' : 'Test IP'}
        </button>
      </div>
    </div>

    <div class="results">
      <h2>📊 Test Results</h2>
      {#if testResults.length === 0}
        <p class="no-results">No tests performed yet.</p>
      {:else}
        <div class="result-list">
          {#each testResults as result, index}
            <div class="result-item {result.hasApp ? 'success' : 'error'}">
              <div class="result-info">
                <span class="ip">{result.ip}</span>
                <span class="status">
                  {#if result.error}
                    ❌ Connection failed
                  {:else if result.hasApp}
                    ✅ Tauri app found
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
  </div>
</main>

<style>
  .container {
    max-width: 800px;
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

  .own-ip {
    background: #f8f9fa;
    padding: 20px;
    border-radius: 8px;
    margin-bottom: 30px;
  }

  .ip-display {
    font-size: 1.5em;
    font-weight: bold;
    color: #007bff;
    text-align: center;
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

  .results {
    background: #f8f9fa;
    padding: 20px;
    border-radius: 8px;
  }

  .no-results {
    text-align: center;
    color: #666;
    font-style: italic;
  }

  .result-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .result-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 15px;
    border-radius: 6px;
    background: white;
    border-left: 4px solid;
  }

  .result-item.success {
    border-left-color: #28a745;
  }

  .result-item.error {
    border-left-color: #dc3545;
  }

  .result-info {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .ip {
    font-weight: bold;
    font-size: 1.1em;
  }

  .status {
    font-size: 0.9em;
  }

  .time {
    font-size: 0.8em;
    color: #666;
  }

  .remove-btn {
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

  .remove-btn:hover {
    background: #c82333;
  }
</style> 