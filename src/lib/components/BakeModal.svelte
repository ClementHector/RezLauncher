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

  // Fetch username when modal opens
  $: if (isOpen && !currentUsername) {
    fetchCurrentUsername();
  }

  // Clear form when not open
  $: if (!isOpen) {
    bakeName = '';
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

  // Submit form
  function handleSubmit(event: Event) {
    event.preventDefault();

    // Validate form
    if (!bakeName) {
      alert('Bake name is required');
      return;
    }

    // Prepare and dispatch form data
    const formData = {
      name: bakeName,
      uri: packageCollectionUri,
      from_version: packageCollectionVersion,
      rxt_path: '',
      tools: packageTools,
      created_at: new Date().toISOString(),
      created_by: currentUsername,
      active: true, // Adding the missing 'active' field
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
</style>