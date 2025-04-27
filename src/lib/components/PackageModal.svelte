<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';

  // Event dispatcher
  const dispatch = createEventDispatcher();

  // Props
  export let isOpen: boolean = false;
  export let packageData: any = null;

  // Form state
  let formData = {
    version: '',
    packages: [] as string[],
    herit: '',
    tools: [] as string[]
  };
  
  // Current input values for tags
  let currentPackageInput = '';
  let currentToolInput = '';

  // Initialize form with package data if provided
  $: if (packageData && isOpen) {
    initFormData();
  }

  function initFormData() {
    formData = {
      version: packageData?.version || '',
      packages: packageData?.packages ? [...packageData.packages] : [],
      herit: packageData?.herit || '',
      tools: packageData?.tools ? [...packageData.tools] : []
    };
    currentPackageInput = '';
    currentToolInput = '';
  }

  // Reset form on close
  $: if (!isOpen) {
    resetForm();
  }

  // Clear form
  function resetForm() {
    formData = {
      version: '',
      packages: [],
      herit: '',
      tools: []
    };
    currentPackageInput = '';
    currentToolInput = '';
  }

  // Close modal and reset form
  function handleClose() {
    dispatch('close');
  }

  // Add new package tag
  function addPackage(e?: KeyboardEvent) {
    if (e && e.key !== 'Enter' && e.key !== ',') {
      return;
    }
    
    if (e) e.preventDefault();
    
    const value = currentPackageInput.trim();
    if (value && !formData.packages.includes(value)) {
      formData.packages = [...formData.packages, value];
      currentPackageInput = '';
    }
  }
  
  // Add new tool tag
  function addTool(e?: KeyboardEvent) {
    if (e && e.key !== 'Enter' && e.key !== ',') {
      return;
    }
    
    if (e) e.preventDefault();
    
    const value = currentToolInput.trim();
    if (value && !formData.tools.includes(value)) {
      formData.tools = [...formData.tools, value];
      currentToolInput = '';
    }
  }
  
  // Remove a package tag
  function removePackage(index: number) {
    formData.packages = formData.packages.filter((_, i) => i !== index);
  }
  
  // Remove a tool tag
  function removeTool(index: number) {
    formData.tools = formData.tools.filter((_, i) => i !== index);
  }

  // Submit form
  function handleSubmit(event: Event) {
    event.preventDefault();

    // Validate form
    if (!formData.version) {
      alert('Version is required');
      return;
    }

    // Add any remaining input as tags
    if (currentPackageInput.trim()) {
      addPackage();
    }
    
    if (currentToolInput.trim()) {
      addTool();
    }

    // Prepare and dispatch form data
    const submitData = {
      version: formData.version,
      packages: formData.packages,
      herit: formData.herit,
      tools: formData.tools,
      // Include original data for identification during update
      originalData: packageData
    };

    dispatch('submit', submitData);
    handleClose();
  }
</script>

{#if isOpen}
<div class="modal-overlay">
  <div class="modal">
    <div class="modal-header">
      <h2>Create Package Collection</h2>
      <button class="close-button" on:click={handleClose}>×</button>
    </div>

    <div class="modal-body">
      <form on:submit={handleSubmit}>
        <div class="form-group">
          <label for="version">Version:</label>
          <input
            type="text"
            id="version"
            bind:value={formData.version}
            placeholder="e.g., 1.0.0"
            required
          />
        </div>

        <div class="form-group">
          <label for="packages">Packages Required:</label>
          <div class="tag-input-container">
            <div class="tags-container">
              {#each formData.packages as pkg, index}
                <div class="tag">
                  {pkg}
                  <button type="button" class="tag-remove" on:click={() => removePackage(index)}>×</button>
                </div>
              {/each}
            </div>
            <input
              type="text"
              id="packages"
              bind:value={currentPackageInput}
              on:keydown={addPackage}
              on:blur={addPackage}
              placeholder="Type and press Enter to add"
            />
          </div>
          <small>Press Enter or comma to add multiple packages</small>
        </div>

        <div class="form-group">
          <label for="herit">Herit:</label>
          <input
            type="text"
            id="herit"
            bind:value={formData.herit}
            placeholder="e.g., base package"
          />
        </div>

        <div class="form-group">
          <label for="tools">Tools:</label>
          <div class="tag-input-container">
            <div class="tags-container">
              {#each formData.tools as tool, index}
                <div class="tag">
                  {tool}
                  <button type="button" class="tag-remove" on:click={() => removeTool(index)}>×</button>
                </div>
              {/each}
            </div>
            <input
              type="text"
              id="tools"
              bind:value={currentToolInput}
              on:keydown={addTool}
              on:blur={addTool}
              placeholder="Type and press Enter to add"
            />
          </div>
          <small>Press Enter or comma to add multiple tools</small>
        </div>

        <div class="form-actions">
          <button type="button" class="cancel-button" on:click={handleClose}>Cancel</button>
          <button type="submit" class="submit-button">Save</button>
        </div>
      </form>
    </div>
  </div>
</div>
{/if}

<style>
  /* Modal styles */
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

  small {
    display: block;
    margin-top: 4px;
    color: #666;
    font-size: 12px;
  }
  
  /* Tag input styling */
  .tag-input-container {
    border: 1px solid #ddd;
    border-radius: 4px;
    padding: 5px;
    background-color: white;
  }
  
  .tag-input-container input {
    border: none;
    padding: 5px;
    width: 100%;
    outline: none;
  }
  
  .tags-container {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
    margin-bottom: 5px;
  }
  
  .tag {
    background-color: #0099cc;
    color: white;
    padding: 3px 8px;
    border-radius: 12px;
    font-size: 12px;
    display: flex;
    align-items: center;
    gap: 5px;
  }
  
  .tag-remove {
    background: none;
    border: none;
    color: white;
    font-size: 14px;
    cursor: pointer;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
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
  
  :global(.theme-dark) .tag-input-container {
    background-color: #2f2f2f;
    border-color: #4a4a4a;
  }

  :global(.theme-dark) .cancel-button {
    background-color: #444;
    border-color: #555;
    color: #eee;
  }
  
  :global(.theme-dark) .tag {
    background-color: #007799;
  }
</style>