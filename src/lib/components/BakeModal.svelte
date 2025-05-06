<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';

  // Event dispatcher
  const dispatch = createEventDispatcher();

  // Props
  export let isOpen: boolean = false;
  export const packageCollectionName: string = '';
  export let packageCollectionVersion: string = '';
  export let packageCollectionUri: string = '';
  export let packageTools: string[] = [];

  // Form state
  let bakeName = '';
  let currentUsername = '';

  // Environment variables
  let envVars: { key: string, value: string }[] = [];
  let currentEnvKey = '';
  let currentEnvValue = '';

  // Fetch username when modal opens
  $: if (isOpen && !currentUsername) {
    fetchCurrentUsername();
  }

  // Clear form when not open
  $: if (!isOpen) {
    bakeName = '';
    envVars = [];
    currentEnvKey = '';
    currentEnvValue = '';
  }

  // Fetch the current username from the backend
  async function fetchCurrentUsername() {
    try {
      currentUsername = await invoke('get_current_username') as string;
    } catch (error) {
      console.error('Error fetching username:', error);
      currentUsername = 'Unknown';
    }
  }

  // Close modal and dispatch close event
  function handleClose() {
    dispatch('close');
  }

  // Add environment variable
  function addEnvVar() {
    if (currentEnvKey.trim() && currentEnvValue.trim()) {
      // Check if the key already exists
      const existingIndex = envVars.findIndex(v => v.key === currentEnvKey.trim());

      if (existingIndex >= 0) {
        // Update existing variable
        envVars[existingIndex].value = currentEnvValue.trim();
        envVars = [...envVars]; // Trigger reactive update
      } else {
        // Add new variable
        envVars = [...envVars, {
          key: currentEnvKey.trim(),
          value: currentEnvValue.trim()
        }];
      }

      // Clear inputs
      currentEnvKey = '';
      currentEnvValue = '';
    }
  }

  // Remove environment variable
  function removeEnvVar(index: number) {
    envVars = envVars.filter((_, i) => i !== index);
  }

  // Submit form
  function handleSubmit(event: Event) {
    event.preventDefault();

    // Validate form
    if (!bakeName) {
      alert('Bake name is required');
      return;
    }

    // Check if there's any pending env var to add
    if (currentEnvKey.trim() && currentEnvValue.trim()) {
      addEnvVar();
    }

    // Prepare and dispatch form data
    const formData = {
      name: bakeName,
      uri: packageCollectionUri,
      from_version: packageCollectionVersion,
      rxt: '', // Initialize with empty string, will be populated by the backend
      tools: packageTools,
      environment_variables: envVars,
      created_at: new Date().toISOString(),
      created_by: currentUsername,
      active: true,
    };

    dispatch('submit', formData);
    handleClose();
  }
</script>

{#if isOpen}
<div class="modal-overlay">
  <div class="modal">
    <div class="modal-header">
      <h2>Create Bake</h2>
      <button class="close-button" on:click={handleClose}>×</button>
    </div>

    <div class="modal-body">
      <form on:submit={handleSubmit}>
        <div class="form-group">
          <label for="bake-name">Bake Name:</label>
          <input
            type="text"
            id="bake-name"
            bind:value={bakeName}
            placeholder="Enter bake name"
            required
          />
        </div>

        <div class="form-group">
          <label for="package-version">From Version:</label>
          <div id="package-version" class="info-field">{packageCollectionVersion}</div>
        </div>

        <div class="form-group">
          <label for="package-tools">Tools:</label>
          <div id="package-tools" class="info-field tools-list">
            {#if packageTools && packageTools.length > 0}
              {#each packageTools as tool}
                <span class="tool-badge">{tool}</span>
              {/each}
            {:else}
              <span class="no-tools">No tools available</span>
            {/if}
          </div>
        </div>

        <div class="form-group">
          <label for="env-var-key-bake">Environment Variables:</label>
          <div class="env-vars-container">
            {#if envVars.length > 0}
              <div class="env-vars-list">
                {#each envVars as envVar, index}
                  <div class="env-var-item">
                    <span class="env-var-key">{envVar.key}</span>
                    <span class="env-var-equals">=</span>
                    <span class="env-var-value">{envVar.value}</span>
                    <button
                      type="button"
                      class="env-var-remove"
                      on:click={() => removeEnvVar(index)}
                    >×</button>
                  </div>
                {/each}
              </div>
            {/if}
            <div class="env-var-input-row">
              <input
                type="text"
                id="env-var-key-bake"
                placeholder="Key"
                bind:value={currentEnvKey}
              />
              <span class="env-var-equals">=</span>
              <input
                type="text"
                id="env-var-value-bake"
                placeholder="Value"
                bind:value={currentEnvValue}
              />
              <button
                type="button"
                class="env-var-add"
                on:click={addEnvVar}
                disabled={!currentEnvKey.trim() || !currentEnvValue.trim()}
              >+</button>
            </div>
          </div>
          <small>Add environment variables as key-value pairs</small>
        </div>

        <div class="form-actions">
          <button type="button" class="cancel-button" on:click={handleClose}>Cancel</button>
          <button type="submit" class="submit-button">OK</button>
        </div>
      </form>
    </div>
  </div>
</div>
{/if}

<style>
  /* Modal styles - Shared with PackageModal */
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
  }

  .modal {
    width: 500px;
    max-width: 90%;
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    background-color: #f6f6f6;
    border-bottom: 1px solid #e0e0e0;
  }

  .modal-header h2 {
    margin: 0;
    font-size: 18px;
    color: #333;
  }

  .close-button {
    background: none;
    border: none;
    font-size: 24px;
    cursor: pointer;
    color: #666;
  }

  .modal-body {
    padding: 20px;
  }

  /* Form styles */
  .form-group {
    margin-bottom: 16px;
  }

  label {
    display: block;
    margin-bottom: 6px;
    font-weight: 500;
  }

  input[type="text"] {
    width: 100%;
    padding: 10px;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 14px;
  }

  .info-field {
    padding: 10px;
    background-color: #f8f8f8;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 14px;
  }

  .tools-list {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .tool-badge {
    background-color: #0099cc;
    color: white;
    padding: 3px 8px;
    border-radius: 12px;
    font-size: 12px;
  }

  .no-tools {
    font-style: italic;
    color: #999;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 20px;
  }

  .cancel-button {
    padding: 8px 16px;
    background-color: #f0f0f0;
    border: 1px solid #ddd;
    border-radius: 4px;
    cursor: pointer;
  }

  .submit-button {
    padding: 8px 16px;
    background-color: #0099cc;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .submit-button:hover {
    background-color: #0077aa;
  }

  /* Environment Variables Styles */
  .env-vars-container {
    border: 1px solid #ddd;
    border-radius: 4px;
    padding: 10px;
    margin-bottom: 5px;
  }

  .env-vars-list {
    margin-bottom: 10px;
    max-height: 150px;
    overflow-y: auto;
  }

  .env-var-item {
    display: flex;
    align-items: center;
    background-color: #f0f8ff;
    padding: 5px 8px;
    border-radius: 4px;
    margin-bottom: 5px;
  }

  .env-var-key {
    font-weight: 500;
    color: #0066cc;
  }

  .env-var-equals {
    margin: 0 6px;
    color: #666;
  }

  .env-var-value {
    flex-grow: 1;
    word-break: break-all;
  }

  .env-var-remove {
    background: none;
    border: none;
    color: #cc0000;
    font-size: 16px;
    cursor: pointer;
    padding: 0 5px;
  }

  .env-var-input-row {
    display: flex;
    align-items: center;
  }

  .env-var-input-row input {
    flex: 1;
    padding: 6px 8px;
  }

  .env-var-add {
    background-color: #0099cc;
    color: white;
    border: none;
    border-radius: 4px;
    width: 28px;
    height: 28px;
    font-size: 16px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-left: 6px;
  }

  .env-var-add:disabled {
    background-color: #cccccc;
    cursor: not-allowed;
  }

  /* Dark mode support */
  :global(.theme-dark) .modal {
    background-color: #3f3f3f;
    color: #f6f6f6;
  }

  :global(.theme-dark) .modal-header {
    background-color: #2f2f2f;
    border-bottom-color: #4a4a4a;
  }

  :global(.theme-dark) .modal-header h2 {
    color: #f6f6f6;
  }

  :global(.theme-dark) .close-button {
    color: #ccc;
  }

  :global(.theme-dark) input[type="text"] {
    background-color: #2f2f2f;
    border-color: #4a4a4a;
    color: #f6f6f6;
  }

  :global(.theme-dark) .info-field {
    background-color: #2f2f2f;
    border-color: #4a4a4a;
    color: #f6f6f6;
  }

  :global(.theme-dark) .tool-badge {
    background-color: #007799;
  }

  :global(.theme-dark) .cancel-button {
    background-color: #444;
    border-color: #555;
    color: #eee;
  }

  :global(.theme-dark) .env-vars-container {
    border-color: #4a4a4a;
  }

  :global(.theme-dark) .env-var-item {
    background-color: #2d3748;
  }

  :global(.theme-dark) .env-var-key {
    color: #63b3ed;
  }

  :global(.theme-dark) .env-var-equals {
    color: #a0aec0;
  }

  :global(.theme-dark) .env-var-add {
    background-color: #2b6cb0;
  }

  :global(.theme-dark) .env-var-add:disabled {
    background-color: #4a5568;
  }

  :global(.theme-dark) .env-var-remove {
    color: #fc8181;
  }
</style>