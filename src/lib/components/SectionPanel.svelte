<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import BakeModal from './BakeModal.svelte';
  import { invoke } from '@tauri-apps/api/tauri';

  // Event dispatcher for custom events
  const dispatch = createEventDispatcher();

  // Props with default values
  export let title: string;                   // Section title
  export let items: any[] = [];               // Array of items to display
  export let expandable: boolean = true;      // Whether section can be expanded/collapsed
  export let showCreateNew: boolean = false;  // Whether to show "Create new" button
  export let hasBakeEdit: boolean = false;    // Whether to show Bake/Edit buttons
  export let mode: string = "Default";        // Current mode (Default/Developer)
  export let isTools: boolean = false;        // Special handling for Tools section
  export let showLoadButtons: boolean = false; // Whether to show load buttons
  export let emptyMessage: string = "";       // Message to display when no items are found

  // Event handlers with default implementations
  export let onLoad = (name: string) => {};
  export let onRevert = (name: string) => {};
  export let onBake = (name: string) => {};
  export let onEdit = (name: string) => {};
  export let onCreate = () => {};
  export let onClick = (item: any) => {};
  export let onRefresh = () => {};           // Function to refresh items list

  // State
  let expanded = true;
  let showBakeModal = false;
  let selectedItem: any = null;
  let packageTools: string[] = [];

  // Toggle expand/collapse
  function toggleExpand() {
    expanded = !expanded;
  }

  // Handle item click with event stopping for nested buttons
  function handleItemClick(item, event) {
    // Don't trigger if clicked on a button inside the item
    if (event.target.tagName === 'BUTTON') {
      return;
    }

    onClick(item);
  }

  // Open bake modal
  async function openBakeModal(item: any) {
    selectedItem = item;

    // Only fetch tools if needed for package collections
    if (!isTools && hasBakeEdit) {
      try {
        const result = await invoke('get_package_collection_tools', {
          version: item.version,
          uri: item.uri
        });

        packageTools = Array.isArray(result) ? result : [];
      } catch (error) {
        console.error('Error fetching tools for package:', error);
        packageTools = [];
      }
    }

    showBakeModal = true;
  }

  // Close bake modal
  function closeBakeModal() {
    showBakeModal = false;
    selectedItem = null;
  }

  // Handle bake form submission
  async function handleBakeSubmit(event: CustomEvent) {
    try {
      const bakeData = event.detail;

      // Save to MongoDB using Tauri command
      await invoke('save_stage_to_mongodb', { stageData: bakeData });

      // Notify any listeners
      dispatch('bake-complete', { success: true, data: bakeData });

    } catch (error) {
      console.error('Error saving stage:', error);
      dispatch('bake-complete', { success: false, error: error.toString() });
    }
  }

  // Handle refreshStages event
  function handleRefreshStages() {
    // Call the refresh function provided by the parent component
    onRefresh();
    // Also dispatch an event in case the parent is listening for it
    dispatch('refresh-stages');
  }

  // Determine if we should render action buttons based on mode
  $: showActions = mode === "Developer" || showLoadButtons;
</script>

<div class="section-panel">
  <div class="panel-header" class:tools-header={isTools}>
    <div class="title-container">
      {#if expandable}
        <span
          class="expand-icon"
          on:click={toggleExpand}
          on:keydown={(e) => e.key === 'Enter' && toggleExpand()}
          tabindex="0"
          role="button"
          aria-label={expanded ? "Collapse" : "Expand"}
        >
          {expanded ? '▼' : '►'}
        </span>
      {/if}
      <h2>{title}</h2>
    </div>
    {#if showCreateNew && mode === "Developer"}
      <button
        class="create-button"
        on:click={onCreate}
        aria-label="Create new"
      >
        <span class="plus-icon">+</span>
      </button>
    {/if}
  </div>

  {#if expanded || !expandable}
    <div class="panel-content">
      {#if items && items.length > 0}
        <ul>
          {#each items as item, i}
            <li
              class="item"
              class:active-item={item.active}
              on:click={(e) => handleItemClick(item, e)}
              on:keydown={(e) => e.key === 'Enter' && onClick(item)}
              tabindex="0"
              role="option"
              aria-selected={item.active || false}
            >
              {#if isTools}
                <!-- Tools display -->
                <div class="tool-item">
                  <span class="item-name">
                    {typeof item === 'string' ? item : item.name}
                  </span>
                  {#if showActions && !(typeof item === 'string' ? item : item.name)?.includes("No tools")}
                    <button
                      class="action-button"
                      on:click={(e) => {
                        e.stopPropagation();
                        onLoad(typeof item === 'string' ? item : item.name);
                      }}
                    >
                      Load
                    </button>
                  {/if}
                </div>
              {:else if hasBakeEdit}
                <!-- Package Collection display -->
                <div class="package-item">
                  <span class="item-name">
                    {item.version || item.name}
                  </span>
                  {#if showActions}
                    <div class="action-buttons">
                      <button
                        class="action-button edit"
                        on:click={(e) => {
                          e.stopPropagation();
                          onEdit(item.version || item.name);
                        }}
                      >
                        Create From
                      </button>
                      <button
                        class="action-button bake"
                        on:click={(e) => {
                          e.stopPropagation();
                          openBakeModal(item);
                        }}
                      >
                        Bake
                      </button>
                      <button
                        class="action-button"
                        on:click={(e) => {
                          e.stopPropagation();
                          onLoad(item.version || item.name);
                        }}
                      >
                        Load
                      </button>
                    </div>
                  {/if}
                </div>
              {:else}
                <!-- Stage display -->
                <div class="stage-item">
                  <span class="item-name">{item.name}</span>
                  <div class="action-buttons">
                    {#if onRevert && showActions}
                      <button
                        class="action-button revert"
                        on:click={(e) => {
                          e.stopPropagation();
                          onRevert(item.name);
                        }}
                      >
                        Revert
                      </button>
                    {/if}
                    <button
                      class="action-button"
                      on:click={(e) => {
                        e.stopPropagation();
                        onLoad(item.name);
                      }}
                    >
                      Load
                    </button>
                  </div>
                </div>
              {/if}
            </li>
          {/each}
        </ul>
      {:else}
        <div class="empty-message">{emptyMessage}</div>
      {/if}
    </div>
  {/if}

  <!-- Bake Modal -->
  <BakeModal
    isOpen={showBakeModal}
    packageCollectionName={selectedItem?.name || ''}
    packageCollectionVersion={selectedItem?.version || ''}
    packageCollectionUri={selectedItem?.uri || ''}
    packageTools={packageTools}
    on:close={closeBakeModal}
    on:submit={handleBakeSubmit}
  />
</div>

<style>
  .section-panel {
    background: var(--card-background);
    border-radius: 8px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    margin-bottom: 15px;
    overflow: hidden;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 15px;
    background-color: var(--primary-color);
    color: white;
  }

  .tools-header {
    background-color: var(--accent-color);
  }

  .title-container {
    display: flex;
    align-items: center;
  }

  .expand-icon {
    margin-right: 8px;
    cursor: pointer;
    font-size: 10px;
  }

  h2 {
    font-size: 16px;
    font-weight: 500;
  }

  .panel-content {
    padding: 10px 0;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .item {
    padding: 8px 15px;
    border-bottom: 1px solid var(--border-color);
    transition: background-color 0.1s;
    cursor: pointer;
  }

  .item:last-child {
    border-bottom: none;
  }

  .item:hover {
    background-color: rgba(0, 0, 0, 0.05);
  }

  .active-item {
    background-color: rgba(0, 153, 204, 0.1);
    border-left: 3px solid var(--primary-color);
  }

  .package-item, .stage-item, .tool-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .item-name {
    font-size: 14px;
  }

  .action-buttons {
    display: flex;
    gap: 5px;
  }

  .action-button {
    padding: 4px 10px;
    background: var(--button-color);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    transition: background-color 0.2s;
  }

  .action-button:hover {
    filter: brightness(1.1);
  }

  .action-button.bake {
    background-color: #00cc66;
  }

  .action-button.edit {
    background-color: #ff9900;
  }

  .action-button.revert {
    background-color: #ff3333;
  }

  .empty-message {
    padding: 12px 15px;
    color: #888;
    font-style: italic;
    font-size: 14px;
  }

  .create-button {
    background: rgba(255, 255, 255, 0.2);
    color: white;
    border: none;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .create-button:hover {
    background: rgba(255, 255, 255, 0.3);
  }

  .plus-icon {
    font-size: 16px;
    line-height: 1;
  }
</style>