<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';

  const dispatch = createEventDispatcher();

  export let isOpen: boolean = false;
  export let stageData: any = null;

  let existingStageNames: string[] = [];
  let selectedTargetStageName: string = '';
  let newStageName: string = '';
  let useNewStageName: boolean = false;
  let loading: boolean = true;
  let error: string | null = null;
  let currentUsername: string = 'unknown';

  $: if (!isOpen) {
    selectedTargetStageName = '';
    newStageName = '';
    useNewStageName = false;
    error = null;
    loading = true;
  }

  $: if (isOpen && stageData) {
    fetchInitialData();
  }

  async function fetchInitialData() {
    loading = true;
    error = null;
    try {
      currentUsername = await invoke<string>('get_current_username');

      const names = await invoke<string[]>('get_all_stage_names');
      existingStageNames = names.filter(name => name !== stageData?.name);

      // We don't set an error anymore when no existing stages found
      // That way user can create a new stage even when there's only one stage

      loading = false;
    } catch (err) {
      console.error('Error fetching data for push modal:', err);
      error = `Failed to load data: ${err}`;
      loading = false;
    }
  }

  function handleClose() {
    dispatch('close');
  }

  async function handlePush() {
    const targetName = useNewStageName ? newStageName.trim() : selectedTargetStageName;

    if (!targetName) {
      error = useNewStageName
        ? "Please enter a name for the new stage."
        : "Please select a target stage name.";
      return;
    }

    if (!stageData) {
      error = "Source stage data is missing.";
      return;
    }

    loading = true;
    error = null;

    try {
      // Create a copy of stageData with the necessary fields
      const newStageData = {
        ...stageData,
        name: targetName,
        active: true,
        created_at: new Date().toISOString(),
        created_by: currentUsername,
        // Ensure rxt field is always a string, not an empty object
        rxt: stageData.rxt || "",
        _id: undefined,
        id: undefined,
      };

      delete newStageData._id;

      console.log("Pushing new stage data:", newStageData);

      await invoke('save_stage_to_mongodb', {
        stageData: newStageData
      });

      dispatch('push-complete', {
        success: true,
        targetStageName: targetName
      });

      handleClose();

    } catch (err) {
      console.error('Error pushing stage:', err);
      error = `Failed to push stage: ${err}`;
      loading = false; // Keep modal open on error
    }
  }
</script>

{#if isOpen}
<div class="modal-overlay"
     on:click|self={handleClose}
     on:keydown={(e) => { if (e.key === 'Escape') handleClose(); }}
     role="dialog"
     tabindex="-1"
     aria-modal="true"
     aria-labelledby="modal-title">
  <div class="modal">
    <div class="modal-header">
      <h2 id="modal-title">Push Stage: {stageData?.name || 'N/A'}</h2>
      <button class="close-button" on:click={handleClose} aria-label="Close modal">×</button>
    </div>

    <div class="modal-body">
      {#if loading}
        <div class="loading">Loading...</div>
      {:else}
        <p class="instructions">Push the current configuration of '{stageData?.name}' to:</p>

        <div class="option-toggles">
          <button
            class="option-button"
            class:selected={!useNewStageName}
            on:click={() => useNewStageName = false}
          >
            Use Existing Stage
          </button>
          <button
            class="option-button"
            class:selected={useNewStageName}
            on:click={() => useNewStageName = true}
          >
            Create New Stage
          </button>
        </div>

        {#if useNewStageName}
          <div class="form-group">
            <label for="new-stage-name">New Stage Name:</label>
            <input
              id="new-stage-name"
              type="text"
              bind:value={newStageName}
              placeholder="Enter new stage name"
            />
          </div>
        {:else}
          <div class="form-group">
            <label for="target-stage-select">Target Stage Name:</label>
            {#if existingStageNames.length > 0}
              <select
                id="target-stage-select"
                bind:value={selectedTargetStageName}
              >
                <option value="" disabled selected>-- Select a stage --</option>
                {#each existingStageNames as name}
                  <option value={name}>{name}</option>
                {/each}
              </select>
            {:else}
              <div class="info-message">No other existing stages found. You can create a new stage instead.</div>
            {/if}
          </div>
        {/if}

        {#if error}
          <div class="error-message">{error}</div>
        {/if}

        <div class="form-actions">
          <button type="button" class="cancel-button" on:click={handleClose}>Cancel</button>
          <button
            type="button"
            class="submit-button"
            on:click={handlePush}
            disabled={(useNewStageName ? !newStageName : !selectedTargetStageName && existingStageNames.length > 0) || loading}
          >
            OK
          </button>
        </div>
      {/if}
    </div>
  </div>
</div>
{/if}

<style>
  /* Reusing modal styles - consider extracting to a shared CSS file or component */
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
    width: 450px;
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

  .instructions {
    margin-bottom: 15px;
    font-size: 14px;
    color: #333;
  }

  .option-toggles {
    display: flex;
    margin-bottom: 20px;
    border-radius: 4px;
    overflow: hidden;
  }

  .option-button {
    flex: 1;
    padding: 8px 12px;
    background-color: #f0f0f0;
    border: 1px solid #ccc;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .option-button.selected {
    background-color: #0099cc;
    color: white;
    border-color: #0099cc;
  }

  .form-group {
    margin-bottom: 20px;
  }

  .form-group label {
    display: block;
    margin-bottom: 5px;
    font-size: 14px;
    font-weight: 500;
  }

  .form-group select,
  .form-group input {
    width: 100%;
    padding: 8px 10px;
    border: 1px solid #ccc;
    border-radius: 4px;
    font-size: 14px;
    background-color: white;
  }

  .form-group select:disabled {
    background-color: #f0f0f0;
    cursor: not-allowed;
  }

  .info-message {
    padding: 10px;
    background-color: #e7f5fe;
    border: 1px solid #bbe1fa;
    border-radius: 4px;
    color: #31708f;
    font-size: 13px;
  }

  /* Status messages */
  .loading, .error-message {
    padding: 15px 0;
    text-align: center;
    font-size: 14px;
  }

  .error-message {
    color: #d9534f;
    background-color: #f2dede;
    border: 1px solid #ebccd1;
    border-radius: 4px;
    padding: 10px;
    margin-bottom: 15px;
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
    color: #333;
  }
   .cancel-button:hover {
     background-color: #e0e0e0;
   }

  .submit-button {
    padding: 8px 16px;
    background-color: #0099cc; /* Primary action color */
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .submit-button:hover {
    background-color: #0077aa;
  }

  .submit-button:disabled {
    background-color: #cccccc;
    cursor: not-allowed;
  }

  /* Dark mode support (optional, copy if needed) */
  :global(.theme-dark) .modal {
    background-color: #3f3f3f;
    color: #f6f6f6;
  }
  :global(.theme-dark) .modal-header {
    background-color: #2f2f2f;
    border-bottom-color: #4a4a4a;
  }
  :global(.theme-dark) .modal-header h2 { color: #f6f6f6; }
  :global(.theme-dark) .close-button { color: #ccc; }
  :global(.theme-dark) .instructions { color: #ddd; }
  :global(.theme-dark) .form-group label { color: #eee; }
  :global(.theme-dark) .form-group select,
  :global(.theme-dark) .form-group input {
    background-color: #555;
    border-color: #666;
    color: #eee;
  }
  :global(.theme-dark) .form-group select:disabled {
    background-color: #4a4a4a;
  }
  :global(.theme-dark) .info-message {
    background-color: #31474f;
    border-color: #356980;
    color: #bbe1fa;
  }
  :global(.theme-dark) .option-button {
    background-color: #444;
    border-color: #555;
    color: #eee;
  }
  :global(.theme-dark) .option-button.selected {
    background-color: #0088bb;
    border-color: #0099cc;
  }
  :global(.theme-dark) .error-message {
    background-color: #5a3e3e;
    border-color: #7a5a5a;
    color: #fadddd;
  }
  :global(.theme-dark) .cancel-button {
    background-color: #444;
    border-color: #555;
    color: #eee;
  }
   :global(.theme-dark) .cancel-button:hover {
     background-color: #555;
   }
  :global(.theme-dark) .submit-button:disabled { background-color: #555; }

</style>
