<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import BakeModal from './BakeModal.svelte';
  import PushToModal from './PushToModal.svelte';
  import { invoke } from '@tauri-apps/api/tauri';

  const dispatch = createEventDispatcher();

  // Props
  export let title: string;
  export let items: any[] = [];
  export let expandable: boolean = true;
  export let showCreateNew: boolean = false;
  export let hasBakeEdit: boolean = false;
  export let mode: string = "Default";
  export let isTools: boolean = false;
  export let showLoadButtons: boolean = false;
  export let emptyMessage: string = "";

  // Event handlers
  export let onLoad = (name: string) => {};
  export let onRevert = (name: string) => {};
  export const onBake = (name: string) => {};
  export let onEdit = (name: string) => {};
  export let onCreate = () => {};
  export let onClick = (item: any) => {};
  export let onRefresh = () => {};

  // State
  let expanded = true;
  let showBakeModal = false;
  let showPushToModal = false;
  let selectedItem: any = null;
  let stageToPush: any = null;
  let packageTools: string[] = [];

  // Determine if action buttons should be shown
  $: showActions = mode === "Developer" || showLoadButtons;

  function toggleExpand() {
    expanded = !expanded;
  }

  function handleItemClick(item: any, event: MouseEvent) {
    if (event.target && (event.target as HTMLElement).tagName === 'BUTTON') {
      return;
    }
    onClick(item);
  }

  async function openBakeModal(item: any) {
    selectedItem = item;

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

  function closeBakeModal() {
    showBakeModal = false;
    selectedItem = null;
  }

  function openPushToModal(item: any) {
    stageToPush = item;
    showPushToModal = true;
  }

  function closePushToModal() {
    showPushToModal = false;
    stageToPush = null;
  }

  function handlePushComplete(event: CustomEvent) {
    console.log(`Push to ${event.detail.targetStageName} complete.`);
    handleRefreshStages();
  }

  async function handleBakeSubmit(event: CustomEvent) {
    try {
      const bakeData = event.detail;
      await invoke('save_stage_to_mongodb', { stageData: bakeData });
      dispatch('bake-complete', { success: true, data: bakeData });
    } catch (error: unknown) {
      console.error('Error saving stage:', error);
      dispatch('bake-complete', {
        success: false,
        error: error instanceof Error ? error.message : String(error)
      });
    }
  }

  function handleRefreshStages() {
    onRefresh();
    dispatch('refresh-stages');
  }
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
                <div class="stage-item">
                  <span class="item-name">{item.name}</span>
                  <div class="action-buttons">
                    {#if onRevert && showActions}
                      <button
                        class="action-button push"
                        on:click={(e) => {
                          e.stopPropagation();
                          openPushToModal(item);
                        }}
                        title="Push this stage's configuration to another stage name"
                      >
                        Push To
                      </button>
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

  <BakeModal
    isOpen={showBakeModal}
    packageCollectionName={selectedItem?.name || ''}
    packageCollectionVersion={selectedItem?.version || ''}
    packageCollectionUri={selectedItem?.uri || ''}
    packageTools={packageTools}
    on:close={closeBakeModal}
    on:submit={handleBakeSubmit}
  />

  <PushToModal
    isOpen={showPushToModal}
    stageData={stageToPush}
    on:close={closePushToModal}
    on:push-complete={handlePushComplete}
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

  .action-button.push {
    background-color: #4CAF50;
  }

  .action-button.revert {
    background-color: #ff9800;
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