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
    tools: [] as string[],
    environment_variables: [] as { key: string, value: string }[]
  };

  // Current input values for tags
  let currentPackageInput = '';
  let currentToolInput = '';
  
  // Current input values for environment variables
  let currentEnvKey = '';
  let currentEnvValue = '';

  // Initialize form with package data if provided
  $: if (packageData && isOpen) {
    initFormData();
  }

  function initFormData() {
    formData = {
      version: packageData?.version || '',
      packages: packageData?.packages ? [...packageData.packages] : [],
      herit: packageData?.herit || '',
      tools: packageData?.tools ? [...packageData.tools] : [],
      environment_variables: packageData?.environment_variables ? [...packageData.environment_variables] : []
    };
    currentPackageInput = '';
    currentToolInput = '';
    currentEnvKey = '';
    currentEnvValue = '';
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
      tools: [],
      environment_variables: []
    };
    currentPackageInput = '';
    currentToolInput = '';
    currentEnvKey = '';
    currentEnvValue = '';
  }

  // Close modal and reset form
  function handleClose() {
    dispatch('close');
  }

  // Add new package tag
  function addPackage(e?: KeyboardEvent | FocusEvent) {
    if (e && 'key' in e && e.key !== 'Enter' && e.key !== ',') {
      return;
    }

    if (e && 'preventDefault' in e) e.preventDefault();

    const value = currentPackageInput.trim();
    if (value && !formData.packages.includes(value)) {
      formData.packages = [...formData.packages, value];
      currentPackageInput = '';
    }
  }

  // Add new tool tag
  function addTool(e?: KeyboardEvent | FocusEvent) {
    if (e && 'key' in e && e.key !== 'Enter' && e.key !== ',') {
      return;
    }

    if (e && 'preventDefault' in e) e.preventDefault();

    const value = currentToolInput.trim();
    if (value && !formData.tools.includes(value)) {
      formData.tools = [...formData.tools, value];
      currentToolInput = '';
    }
  }

  // Add environment variable
  function addEnvVar() {
    if (currentEnvKey.trim() && currentEnvValue.trim()) {
      // Check if the key already exists
      const existingIndex = formData.environment_variables.findIndex(v => v.key === currentEnvKey.trim());
      
      if (existingIndex >= 0) {
        // Update existing variable
        formData.environment_variables[existingIndex].value = currentEnvValue.trim();
        formData.environment_variables = [...formData.environment_variables]; // Trigger reactive update
      } else {
        // Add new variable
        formData.environment_variables = [...formData.environment_variables, { 
          key: currentEnvKey.trim(), 
          value: currentEnvValue.trim() 
        }];
      }
      
      // Clear inputs
      currentEnvKey = '';
      currentEnvValue = '';
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

  // Remove environment variable
  function removeEnvVar(index: number) {
    formData.environment_variables = formData.environment_variables.filter((_, i) => i !== index);
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

    // Add any remaining env var input
    if (currentEnvKey.trim() && currentEnvValue.trim()) {
      addEnvVar();
    }

    // Prepare and dispatch form data
    const submitData = {
      version: formData.version,
      packages: formData.packages,
      herit: formData.herit,
      tools: formData.tools,
      environment_variables: formData.environment_variables,
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

        <div class="form-group">
          <label>Environment Variables:</label>
          <div class="env-vars-container">
            {#if formData.environment_variables.length > 0}
              <div class="env-vars-list">
                {#each formData.environment_variables as envVar, index}
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
                placeholder="Key"
                bind:value={currentEnvKey}
              />
              <span class="env-var-equals">=</span>
              <input
                type="text"
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