<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';

  // Event dispatcher
  const dispatch = createEventDispatcher();

  // Props
  export let isOpen: boolean = false;
  export let stageName: string = '';
  export let stageUri: string = '';

  // Local state
  let stageVersions = [];
  let selectedStageId = null;
  let loading = true;
  let error = null;

  // Reset state when modal closes
  $: if (!isOpen) {
    selectedStageId = null;
  }

  // Fetch stage history when modal opens
  $: if (isOpen && stageName && stageUri) {
    fetchStageHistory();
  }

  async function fetchStageHistory() {
    loading = true;
    error = null;

    try {
      const versions = await invoke('get_stage_history', {
        stageName: stageName,
        uri: stageUri
      });

      stageVersions = versions.map(stage => ({
        ...stage,
        formattedDate: formatDate(stage.created_at)
      }));

      loading = false;
    } catch (err) {
      console.error('Error fetching stage history:', err);
      error = `Failed to fetch stage history: ${err}`;
      loading = false;
    }
  }

  function formatDate(dateString) {
    try {
      const date = new Date(dateString);
      return date.toLocaleString();
    } catch (e) {
      return dateString;
    }
  }

  function selectStage(stageId) {
    selectedStageId = stageId;
  }

  function handleClose() {
    dispatch('close');
  }

  async function handleRevert() {
    if (!selectedStageId) {
      error = "Please select a stage version to revert to";
      return;
    }

    try {
      await invoke('revert_stage', {
        stageId: selectedStageId
      });

      dispatch('revert-complete', {
        success: true,
        stageName: stageName
      });

      handleClose();
    } catch (err) {
      console.error('Error reverting stage:', err);
      error = `Failed to revert stage: ${err}`;
    }
  }
</script>

{#if isOpen}
<div class="modal-overlay">
  <div class="modal">
    <div class="modal-header">
      <h2>Revert Stage: {stageName}</h2>
      <button class="close-button" on:click={handleClose}>×</button>
    </div>

    <div class="modal-body">
      {#if loading}
        <div class="loading">Loading stage history...</div>
      {:else if error}
        <div class="error-message">{error}</div>
      {:else if stageVersions.length === 0}
        <div class="empty-message">No stage versions found.</div>
      {:else}
        <p class="instructions">Select a stage version to revert to:</p>

        <div class="stage-list">
          {#each stageVersions as stage}
            {console.log('Stage _id:', stage._id)}
            <div
              class="stage-item"
              class:active={stage._id === selectedStageId}
              class:current-active={stage.active}
              on:click={() => selectStage(stage._id)}
              on:keydown={(e) => e.key === 'Enter' && selectStage(stage._id)}
              tabindex="0"
              role="option"
              aria-selected={stage._id === selectedStageId}
            >
              <div class="stage-info">
                <div class="stage-header">
                  <span class="stage-version">{stage.from_version}</span>
                  {#if stage.active}
                    <span class="active-badge">Current</span>
                  {/if}
                </div>
                <div class="stage-details">
                  <span class="stage-date">Created: {stage.formattedDate}</span>
                  <span class="stage-creator">By: {stage.created_by || 'Unknown'}</span>
                </div>
              </div>
            </div>
          {/each}
        </div>
      {/if}

      <div class="form-actions">
        <button type="button" class="cancel-button" on:click={handleClose}>Cancel</button>
        <button
          type="button"
          class="submit-button"
          on:click={handleRevert}
          disabled={!selectedStageId || loading}
        >
          OK
        </button>
      </div>
    </div>
  </div>
</div>
{/if}

<style>
  /* Modal styles - Shared with other modals */
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

  /* Stage list styles */
  .instructions {
    margin-bottom: 15px;
    font-size: 14px;
  }

  .stage-list {
    max-height: 300px;
    overflow-y: auto;
    border: 1px solid #ddd;
    border-radius: 4px;
    margin-bottom: 20px;
  }

  .stage-item {
    padding: 12px 15px;
    border-bottom: 1px solid #eee;
    cursor: pointer;
    transition: background-color 0.1s;
  }

  .stage-item:last-child {
    border-bottom: none;
  }

  .stage-item:hover {
    background-color: rgba(0, 153, 204, 0.05);
  }

  .stage-item.active {
    background-color: rgba(0, 153, 204, 0.1);
    border-left: 3px solid #0099cc;
  }

  .stage-item.current-active {
    border-left: 3px solid #00cc66;
  }

  .stage-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 5px;
  }

  .stage-version {
    font-weight: 500;
    font-size: 15px;
  }

  .active-badge {
    background-color: #00cc66;
    color: white;
    padding: 2px 6px;
    border-radius: 10px;
    font-size: 12px;
  }

  .stage-details {
    display: flex;
    font-size: 13px;
    color: #666;
  }

  .stage-date {
    margin-right: 15px;
  }

  /* Status messages */
  .loading, .error-message, .empty-message {
    padding: 20px;
    text-align: center;
    font-size: 14px;
  }

  .error-message {
    color: #d9534f;
  }

  .empty-message {
    color: #666;
    font-style: italic;
  }

  /* Action buttons */
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

  .submit-button:disabled {
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

  :global(.theme-dark) .stage-list {
    border-color: #4a4a4a;
  }

  :global(.theme-dark) .stage-item {
    border-bottom-color: #4a4a4a;
  }

  :global(.theme-dark) .stage-details {
    color: #aaa;
  }

  :global(.theme-dark) .cancel-button {
    background-color: #444;
    border-color: #555;
    color: #eee;
  }

  :global(.theme-dark) .empty-message {
    color: #aaa;
  }
</style>