<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import SectionPanel from "$lib/components/SectionPanel.svelte";
  import PackageModal from "$lib/components/PackageModal.svelte";
  import RevertModal from "$lib/components/RevertModal.svelte";

  let mode = $state("Config");
  let theme = $state("Light");

  type LogEntry = {
    message: string;
    type: "info" | "success" | "warning" | "error";
    icon: string;
  };

  let logs = $state<LogEntry[]>([]);
  let packageModalOpen = $state(false);
  let revertModalOpen = $state(false);
  let isEditingPackage = $state(false);
  let currentPackageData = $state<PackageCollection | null>(null);
  let currentStageName = $state("");
  let currentStageUri = $state("");
  let currentUsername = $state("");
  let packageCollections = $state<PackageCollection[]>([]);
  let packageCollectionMessage = $state("");
  let stageMessage = $state("");
  let allUris = $state<string[]>([]);

  type Package = {
    version: string;
    baked: boolean;
    active?: boolean;
    uri?: string;
  };

  type PackageCollection = {
    version: string;
    uri?: string;
    created_at?: string;
    created_by?: string;
    tools?: any[];
    [key: string]: any;
  };

  let currentUri = $state("");
  let projectSelection = $state("select");
  let modelingSelection = $state("select");
  let applicationSelection = $state("select");

  let showModelingCombobox = $state(false);
  let showApplicationCombobox = $state(false);

  let isAddingProjectOption = $state(false);
  let isAddingModelingOption = $state(false);
  let isAddingApplicationOption = $state(false);
  let newOptionText = $state("");

  type Stage = {
    name: string;
    loaded?: boolean;
    uri?: string;
    from_version?: string;
    rxt_path?: string;
    tools?: any[];
    active?: boolean;
  };

  let stages = $state<Stage[]>([]);
  let activePackage = $state<string | null>(null);
  let activeStage = $state<string | null>(null);
  let packages = $state<Package[]>([]);
  type Tool = string | { name: string; loaded?: boolean; [key: string]: any };
  let tools = $state<Tool[]>([]);
  let applicationDropdownOpen = $state(false);

  let projectDropdownOpen = $state(false);
  let modelingDropdownOpen = $state(false);
  let projectOptions = $state<string[]>(["select"]);
  let modelingOptions = $state<string[]>(["select"]);
  let applicationOptions = $state<string[]>(["select"]);

  function buildCurrentUri() {
    if (projectSelection === "select") return "";

    if (!showModelingCombobox || modelingSelection === "select") {
      return `/${projectSelection}`;
    } else if (!showApplicationCombobox || applicationSelection === "select") {
      return `/${projectSelection}/${modelingSelection}`;
    } else {
      return `/${projectSelection}/${modelingSelection}/${applicationSelection}`;
    }
  }

  function toggleDropdown(dropdownName: string) {
    projectDropdownOpen = dropdownName === 'project' ? !projectDropdownOpen : false;
    modelingDropdownOpen = dropdownName === 'modeling' ? !modelingDropdownOpen : false;
    applicationDropdownOpen = dropdownName === 'application' ? !applicationDropdownOpen : false;
  }

  function selectOption(dropdownName: string, option: string) {
    switch(dropdownName) {
      case 'project':
        projectSelection = option;
        projectDropdownOpen = false;
        modelingSelection = "select";
        applicationSelection = "select";
        showModelingCombobox = true;
        showApplicationCombobox = false;
        break;
      case 'modeling':
        modelingSelection = option;
        modelingDropdownOpen = false;
        applicationSelection = "select";
        showApplicationCombobox = true;
        break;
      case 'application':
        applicationSelection = option;
        applicationDropdownOpen = false;
        break;
    }
    addLog(`Selected ${dropdownName}: ${option}`);

    activeStage = null;
    activePackage = null;
    tools = [];

    fetchPackageCollectionsByUri();
    fetchStagesByUri();
  }

  function addNewOption(dropdownType: string) {
    if (!newOptionText.trim()) {
      addLog("Option name cannot be empty", "warning");
      return;
    }

    switch(dropdownType) {
      case 'project':
        if (projectOptions.includes(newOptionText)) {
          addLog(`Project "${newOptionText}" already exists`, "warning");
        } else {
          projectOptions = [...projectOptions, newOptionText];
          projectSelection = newOptionText;
          isAddingProjectOption = false;
          showModelingCombobox = true;
          addLog(`Added new project: ${newOptionText}`, "success");
        }
        break;
      case 'modeling':
        if (modelingOptions.includes(newOptionText)) {
          addLog(`Modeling type "${newOptionText}" already exists`, "warning");
        } else {
          modelingOptions = [...modelingOptions, newOptionText];
          modelingSelection = newOptionText;
          isAddingModelingOption = false;
          showApplicationCombobox = true;
          addLog(`Added new modeling type: ${newOptionText}`, "success");
        }
        break;
      case 'application':
        if (applicationOptions.includes(newOptionText)) {
          addLog(`Application "${newOptionText}" already exists`, "warning");
        } else {
          applicationOptions = [...applicationOptions, newOptionText];
          applicationSelection = newOptionText;
          isAddingApplicationOption = false;
          addLog(`Added new application: ${newOptionText}`, "success");
        }
        break;
    }

    newOptionText = "";
    fetchPackageCollectionsByUri();
    fetchStagesByUri();
  }

  function startAddingOption(dropdownType: string) {
    newOptionText = "";
    isAddingProjectOption = dropdownType === 'project';
    isAddingModelingOption = dropdownType === 'modeling';
    isAddingApplicationOption = dropdownType === 'application';

    projectDropdownOpen = false;
    modelingDropdownOpen = false;
    applicationDropdownOpen = false;
  }

  function cancelAddingOption() {
    isAddingProjectOption = false;
    isAddingModelingOption = false;
    isAddingApplicationOption = false;
    newOptionText = "";
  }

  function toggleDarkMode() {
    theme = theme === "Light" ? "Dark" : "Light";
  }

  async function handlePackageClick(item: { version: string }) {
    try {
      activeStage = null;
      packages = packages.map(pkg => ({
        ...pkg,
        active: pkg.version === item.version
      }));
      stages = stages.map(stage => ({
        ...stage,
        active: false
      }));
      activePackage = item.version;
      addLog(`Selected package collection: ${item.version}`, "info");
      await fetchToolsForPackage(item.version);
    } catch (error) {
      addLog(`Error selecting package: ${error}`, "error");
    }
  }

  async function handleStageClick(item: Stage) {
    try {
      activePackage = null;
      stages = stages.map(stage => ({
        ...stage,
        active: stage.name === item.name
      }));
      packages = packages.map(pkg => ({
        ...pkg,
        active: false
      }));
      activeStage = item.name;
      addLog(`Selected stage: ${item.name}`, "info");
      await fetchToolsForStage(item);
    } catch (error) {
      addLog(`Error selecting stage: ${error}`, "error");
    }
  }

  async function fetchToolsForPackage(packageVersion: string) {
    try {
      addLog(`Fetching tools for package collection: ${packageVersion}`, "info");
      const selectedPackage = packageCollections.find(pkg => pkg.version === packageVersion);

      if (!selectedPackage) {
        addLog(`Package collection ${packageVersion} not found`, "error");
        return;
      }

      if (selectedPackage.tools && Array.isArray(selectedPackage.tools) && selectedPackage.tools.length > 0) {
        tools = selectedPackage.tools.map(tool => tool.name || tool);
        addLog(`Found ${tools.length} tools in package ${packageVersion}`, "success");
      } else {
        tools = ["No tools for the selection"];
        addLog(`No tools found in package ${packageVersion}`, "warning");
      }
    } catch (error) {
      addLog(`Error fetching tools: ${error}`, "error");
      tools = ["No tools for the selection"];
    }
  }

  async function fetchToolsForStage(stage: Stage) {
    try {
      addLog(`Fetching tools for stage: ${stage.name}`, "info");
      if (stage.tools && Array.isArray(stage.tools) && stage.tools.length > 0) {
        tools = stage.tools.map(tool => tool.name || tool);
        addLog(`Found ${tools.length} tools in stage ${stage.name}`, "success");
      } else {
        tools = ["No tools for the selection"];
        addLog(`No tools found in stage ${stage.name}`, "warning");
      }
    } catch (error) {
      addLog(`Error fetching tools for stage: ${error}`, "error");
      tools = ["No tools for the selection"];
    }
  }

  async function loadStage(stageName: string) {
    try {
      addLog(`Loading stage: ${stageName}`);
      stages = stages.map(stage =>
        stage.name === stageName
          ? { ...stage, loaded: true }
          : { ...stage, loaded: false }
      );
    } catch (error) {
      addLog(`Error loading stage: ${error}`, "error");
    }
  }

  async function revertStage(stageName: string) {
    try {
      const stageToRevert = stages.find(stage => stage.name === stageName);
      if (stageToRevert) {
        currentStageName = stageName;
        currentStageUri = stageToRevert.uri || "";
        revertModalOpen = true;
        addLog(`Opening revert dialog for stage: ${stageName}`);
      } else {
        addLog(`Stage ${stageName} not found for reverting`, "error");
      }
    } catch (error) {
      addLog(`Error opening revert modal: ${error}`, "error");
    }
  }

  function closeRevertModal() {
    revertModalOpen = false;
    currentStageName = "";
    currentStageUri = "";
    addLog("Cancelled revert operation");
  }

  async function handleRevertComplete(event: CustomEvent) {
    const { success, stageName } = event.detail;

    if (success) {
      addLog(`Successfully reverted stage: ${stageName}`, "success");
      await fetchStagesByUri();
    } else {
      addLog(`Failed to revert stage: ${event.detail.error}`, "error");
    }
  }

  async function bakePackage(packageName: string) {
    try {
      addLog(`Baking package: ${packageName}`);
      packages = packages.map(pkg =>
        pkg.version === packageName
          ? { ...pkg, baked: true }
          : pkg
      );
    } catch (error) {
      addLog(`Error baking package: ${error}`, "error");
    }
  }

  async function createFromPackage(packageVersion: string) {
    try {
      const packageToCreateFrom = packageCollections.find(pkg => pkg.version === packageVersion);
      if (packageToCreateFrom) {
        currentPackageData = packageToCreateFrom;
        isEditingPackage = true;
        packageModalOpen = true;
        addLog(`Creating from package collection: ${packageVersion}`);
      } else {
        addLog(`Package collection ${packageVersion} not found for creating from`, "error");
      }
    } catch (error) {
      addLog(`Error creating from package: ${error}`, "error");
    }
  }

  async function loadTool(toolName: string) {
    try {
      addLog(`Loading tool: ${toolName}`);

      tools = tools.map(tool => {
        const name = typeof tool === 'string' ? tool : tool.name;
        return name === toolName
          ? typeof tool === 'string' ? { name: tool, loaded: true } : { ...tool, loaded: true }
          : typeof tool === 'string' ? tool : { ...tool, loaded: false };
      });

      if (toolName && !toolName.includes("No tools")) {
        addLog(`Launching tool in terminal: ${toolName}`, "info");

        // Récupérer la liste des packages en fonction de la sélection actuelle
        let packages: string[] = [];

        if (activeStage) {
          // Si un stage est sélectionné, récupérer ses packages
          const selectedStage = stages.find(stage => stage.name === activeStage);
          if (selectedStage && selectedStage.from_version) {
            // Trouver le package collection correspondant au from_version du stage
            const stagePackage = packageCollections.find(pkg => pkg.version === selectedStage.from_version);
            if (stagePackage && stagePackage.packages && Array.isArray(stagePackage.packages)) {
              packages = stagePackage.packages;
              addLog(`Using ${packages.length} packages from stage "${activeStage}"`, "info");
            } else {
              addLog(`No packages found for stage "${activeStage}", using empty package list`, "warning");
            }
          }
        } else if (activePackage) {
          // Si un package collection est sélectionné, récupérer ses packages
          const selectedPackage = packageCollections.find(pkg => pkg.version === activePackage);
          if (selectedPackage && selectedPackage.packages && Array.isArray(selectedPackage.packages)) {
            packages = selectedPackage.packages;
            addLog(`Using ${packages.length} packages from package collection "${activePackage}"`, "info");
          } else {
            addLog(`No packages found for package collection "${activePackage}", using empty package list`, "warning");
          }
        }

        // Appeler la fonction Tauri en passant la liste des packages
        await invoke("open_tool_in_terminal", {
          toolName: toolName,
          packages: packages
        });

        addLog(`Tool ${toolName} launched successfully in terminal with ${packages.length} packages`, "success");
      }
    } catch (error) {
      addLog(`Error loading tool: ${error}`, "error");
    }
  }

  function addLog(message: string, type: "info" | "success" | "warning" | "error" = "info") {
    const icon = type === "success" ? "✓" :
                type === "warning" ? "⚠" :
                type === "error" ? "✕" : "i";
    logs = [{ message, type, icon }, ...logs];
  }

  function createNewPackage() {
    packageModalOpen = true;
    addLog("Opening package collection creation form...");
  }

  async function handlePackageSubmit(event: CustomEvent) {
    try {
      const formData = event.detail;
      const uri = buildCurrentUri();

      if (!uri) {
        addLog("Cannot create a package: please select at least one project", "error");
        packageModalOpen = false;
        return;
      }

      const newPackage = {
        ...formData,
        created_at: new Date().toISOString(),
        created_by: currentUsername,
        uri: uri
      };

      await invoke("save_package_collection", { packageData: newPackage });
      packages = [...packages, { version: formData.version, baked: false }];

      const actionType = isEditingPackage ? "Created new package collection from existing" : "Created new package collection";
      addLog(`${actionType} with version: ${formData.version} by ${currentUsername}`, "success");

      isEditingPackage = false;
      currentPackageData = null;
      packageModalOpen = false;

      fetchPackageCollectionsByUri();
    } catch (error) {
      addLog(`Error saving package collection: ${error}`, "error");
    }
  }

  function closePackageModal() {
    packageModalOpen = false;
    isEditingPackage = false;
    currentPackageData = null;
    addLog("Cancelled package collection creation/edit");
  }

  function resetToHome() {
    projectSelection = "select";
    modelingSelection = "select";
    applicationSelection = "select";
    showModelingCombobox = false;
    showApplicationCombobox = false;
    addLog("Reset URI path to home", "info");
    currentUri = buildCurrentUri();
  }

  function refreshPackages() {
    addLog("Refreshing package collections and stages...", "info");
    fetchPackageCollectionsByUri();
    fetchStagesByUri();
  }

  async function fetchCurrentUsername() {
    try {
      const username = await invoke("get_current_username");
      currentUsername = username as string;
      addLog(`Session user identified: ${username}`, "info");
    } catch (error) {
      addLog(`Failed to get current username: ${error}`, "warning");
      currentUsername = "unknown_user";
    }
  }

  async function fetchPackageCollectionsByUri() {
    try {
      currentUri = buildCurrentUri();

      addLog(`Fetching package collections for URI: ${currentUri}`, "info");
      const result = await invoke("get_package_collections_by_uri", { uri: currentUri }) as {
        success: boolean;
        collections?: PackageCollection[];
        message?: string;
      };

      if (result.success) {
        if (result.collections) {
          packageCollections = result.collections;
          packages = packageCollections.map(collection => ({
            version: collection.version,
            baked: false,
            uri: collection.uri,
            active: false
          }));
          packageCollectionMessage = "";
          addLog(`Found ${packageCollections.length} package collections for ${currentUri}`, "success");
        } else {
          packageCollections = [];
          packages = [];
          packageCollectionMessage = result.message || `no collection found in ${currentUri}`;
          addLog(packageCollectionMessage, "warning");
        }
      } else {
        addLog(`Error fetching package collections: ${result.message}`, "error");
        packageCollectionMessage = `Error: ${result.message}`;
        packageCollections = [];
        packages = [];
      }
      activePackage = null;
    } catch (error) {
      addLog(`Error fetching package collections: ${error}`, "error");
      packageCollectionMessage = `Error: ${error}`;
      packageCollections = [];
      packages = [];
      activePackage = null;
    }
  }

  async function fetchAllUrisAndUpdateOptions() {
    try {
      addLog("Fetching all available URIs from MongoDB...");
      const result = await invoke("get_all_package_collections") as {
        success: boolean;
        collections?: PackageCollection[];
        message?: string;
      };

      if (result.success && result.collections) {
        allUris = result.collections.map(collection => collection.uri || "");

        const projectSet = new Set(["select"]);
        const modelingSet = new Set(["select"]);
        const appSet = new Set(["select"]);

        allUris.forEach(uri => {
          if (!uri) return;
          const parts = uri.split('/').filter(part => part);
          if (parts.length >= 1) projectSet.add(parts[0]);
          if (parts.length >= 2) modelingSet.add(parts[1]);
          if (parts.length >= 3) appSet.add(parts[2]);
        });

        projectOptions = Array.from(projectSet);
        modelingOptions = Array.from(modelingSet);
        applicationOptions = Array.from(appSet);

        addLog(`Updated dropdown options from ${allUris.length} URIs in MongoDB`, "success");
      } else {
        addLog("Failed to fetch URIs from MongoDB", "warning");
      }
    } catch (error) {
      addLog(`Error fetching URIs: ${error}`, "error");
    }
  }

  async function fetchStagesByUri(showActiveOnly = true) {
    try {
      currentUri = buildCurrentUri();

      addLog(`Fetching stages for URI: ${currentUri}${showActiveOnly ? ' (active only)' : ''}`, "info");
      const stagesData = await invoke("get_stages_by_uri", { uri: currentUri, activeOnly: showActiveOnly });

      if (Array.isArray(stagesData) && stagesData.length > 0) {
        stages = stagesData.map(stage => ({
          name: stage.name,
          loaded: false,
          uri: stage.uri,
          from_version: stage.from_version,
          rxt_path: stage.rxt_path,
          tools: stage.tools,
          active: false
        }));
        stageMessage = "";
        addLog(`Found ${stages.length} stages for ${currentUri}`, "success");
      } else {
        stages = [];
        stageMessage = `No ${showActiveOnly ? 'active ' : ''}stages found for ${currentUri}`;
        addLog(`No ${showActiveOnly ? 'active ' : ''}stages found for ${currentUri}`, "warning");
      }
      activeStage = null;
    } catch (error) {
      addLog(`Error fetching stages: ${error}`, "error");
      stages = [];
      stageMessage = `Error: ${error}`;
      activeStage = null;
    }
  }

  async function handleBakeComplete(event: CustomEvent) {
    try {
      const { success, data } = event.detail;
      if (success) {
        addLog(`Stage "${data.name}" created successfully from package ${data.from_version}`, "success");
        await fetchStagesByUri();
      } else {
        addLog(`Failed to create stage: ${event.detail.error}`, "error");
      }
    } catch (error) {
      addLog(`Error handling bake completion: ${error}`, "error");
    }
  }

  async function refreshStages() {
    addLog("Refreshing stages...", "info");
    await fetchStagesByUri();
  }

  async function initApp() {
    await fetchCurrentUsername();
    await fetchAllUrisAndUpdateOptions();
    await fetchPackageCollectionsByUri();
    await fetchStagesByUri();
    addLog("RezLauncher started");
  }

  initApp();
</script>

<main class="rezlauncher {theme === 'Dark' ? 'theme-dark' : ''}">
  <header>
    <div class="title">
      <h1>RezLauncher</h1>
    </div>
    <div class="mode-selection">
      <span>Mode:</span>
      <label>
        <input type="radio" bind:group={mode} value="Launcher">
        Launcher
      </label>
      <label>
        <input type="radio" bind:group={mode} value="Config">
        Config
      </label>
      <label class="theme-toggle">
        <input type="checkbox" checked={theme === "Dark"} onchange={toggleDarkMode}>
        Dark
      </label>
    </div>
  </header>

  <nav class="breadcrumb">
    <span class="uri-label">LAYERS:</span>

    {#if isAddingProjectOption}
      <div class="add-option-form">
        <input
          type="text"
          bind:value={newOptionText}
            placeholder="New project..."
          onkeypress={(e) => e.key === 'Enter' && addNewOption('project')}
        />
        <button class="add-btn" onclick={() => addNewOption('project')}>Add</button>
        <button class="cancel-btn" onclick={cancelAddingOption}>Cancel</button>
      </div>
    {:else}
      <div class="dropdown">
        <button class="dropdown-button" onclick={() => toggleDropdown('project')}>
          {projectSelection}
        </button>
        {#if projectDropdownOpen}
          <ul class="dropdown-menu">
            {#each projectOptions as option}
              <button
                type="button"
                class="dropdown-item"
                class:selected={option === projectSelection}
                onclick={() => selectOption('project', option)}
                onkeydown={(e) => e.key === 'Enter' && selectOption('project', option)}
              >
                {option}
              </button>
            {/each}
            <button
              type="button"
              class="add-new-option dropdown-item"
              onclick={() => startAddingOption('project')}
              onkeydown={(e) => e.key === 'Enter' && startAddingOption('project')}
            >
                <span class="plus-icon">+</span> Add...
            </button>
          </ul>
        {/if}
      </div>
    {/if}

    {#if showModelingCombobox}
      <span>/</span>
      {#if isAddingModelingOption}
        <div class="add-option-form">
          <input
            type="text"
            bind:value={newOptionText}
            placeholder="Nouveau type..."
            onkeypress={(e) => e.key === 'Enter' && addNewOption('modeling')}
          />
            <button class="add-btn" onclick={() => addNewOption('modeling')}>Add</button>
          <button class="cancel-btn" onclick={cancelAddingOption}>Annuler</button>
        </div>
      {:else}
        <div class="dropdown">
          <button class="dropdown-button" onclick={() => toggleDropdown('modeling')}>
            {modelingSelection}
          </button>
          {#if modelingDropdownOpen}
            <ul class="dropdown-menu">
              {#each modelingOptions as option}
                <button
                  type="button"
                  class="dropdown-item"
                  class:selected={option === modelingSelection}
                  onclick={() => selectOption('modeling', option)}
                  onkeydown={(e) => e.key === 'Enter' && selectOption('modeling', option)}
                >
                  {option}
                </button>
              {/each}
              <button
                type="button"
                class="add-new-option dropdown-item"
                onclick={() => startAddingOption('modeling')}
                onkeydown={(e) => e.key === 'Enter' && startAddingOption('modeling')}
              >
                <span class="plus-icon">+</span> Add...
              </button>
            </ul>
          {/if}
        </div>
      {/if}
    {/if}

    {#if showApplicationCombobox}
      <span>/</span>
      {#if isAddingApplicationOption}
        <div class="add-option-form">
          <input
            type="text"
            bind:value={newOptionText}
            placeholder="Nouvelle application..."
            onkeypress={(e) => e.key === 'Enter' && addNewOption('application')}
          />
          <button class="add-btn" onclick={() => addNewOption('application')}>Add</button>
          <button class="cancel-btn" onclick={cancelAddingOption}>Annuler</button>
        </div>
      {:else}
        <div class="dropdown">
          <button class="dropdown-button" onclick={() => toggleDropdown('application')}>
            {applicationSelection}
          </button>
          {#if applicationDropdownOpen}
            <ul class="dropdown-menu">
              {#each applicationOptions as option}
                <button
                  type="button"
                  class="dropdown-item"
                  class:selected={option === applicationSelection}
                  onclick={() => selectOption('application', option)}
                  onkeydown={(e) => e.key === 'Enter' && selectOption('application', option)}
                >
                  {option}
                </button>
              {/each}
              <button
                type="button"
                class="add-new-option dropdown-item"
                onclick={() => startAddingOption('application')}
                onkeydown={(e) => e.key === 'Enter' && startAddingOption('application')}
              >
                <span class="plus-icon">+</span> Add...
              </button>
            </ul>
          {/if}
        </div>
      {/if}
    {/if}

    <div class="nav-buttons">
      <button
        class="icon-button home-button"
        onclick={resetToHome}
        title="Home"
        aria-label="Reset to Home"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path>
          <polyline points="9 22 9 12 15 12 15 22"></polyline>
        </svg>
      </button>
      <button
        class="icon-button refresh-button"
        onclick={refreshPackages}
        title="Refresh"
        aria-label="Refresh Packages"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M23 4v6h-6"></path>
          <path d="M1 20v-6h6"></path>
          <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10"></path>
          <path d="M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
        </svg>
      </button>
    </div>
  </nav>

  <div class="content-area">
    <div class="left-panel">
      <SectionPanel
        title="Stages"
        items={stages}
        expandable={true}
        mode={mode}
        emptyMessage={stageMessage}
        onLoad={loadStage}
        onRevert={revertStage}
        onClick={handleStageClick}
        onRefresh={refreshStages}
      />

      {#if mode === "Config"}
        <SectionPanel
          title="Package Collections"
          items={packages}
          expandable={true}
          showCreateNew={true}
          hasBakeEdit={true}
          mode={mode}
          emptyMessage={packageCollectionMessage}
          onLoad={() => {}}
          onBake={bakePackage}
          onEdit={createFromPackage}
          onCreate={createNewPackage}
          onClick={handlePackageClick}
          on:bake-complete={handleBakeComplete}
        />
      {/if}
    </div>

    <div class="right-panel">
      <SectionPanel
        title="Tools"
        items={tools}
        expandable={false}
        isTools={true}
        mode={mode}
        onLoad={loadTool}
        showLoadButtons={true}
      />
    </div>
  </div>

  <PackageModal
    isOpen={packageModalOpen}
    packageData={currentPackageData}
    on:close={closePackageModal}
    on:submit={handlePackageSubmit}
  />

  <RevertModal
    isOpen={revertModalOpen}
    stageName={currentStageName}
    stageUri={currentStageUri}
    on:close={closeRevertModal}
    on:revert-complete={handleRevertComplete}
  />

  <footer>
    <div class="logs">
      <h3>Logs:</h3>
      <div class="logs-container">
        {#each logs as log}
          <div class="log-entry log-{log.type}">
            <span class="log-icon">[{log.icon}]</span>
            <span class="log-message">{log.message}</span>
          </div>
        {/each}
      </div>
    </div>
  </footer>
</main>

<style>
  :root {
    --primary-color: #0099cc;
    --accent-color: #00cc99;
    --background-color: #f6f6f6;
    --card-background: #ffffff;
    --text-color: #2f2f2f;
    --button-color: #0099cc;
    --button-text: white;
    --border-color: #e0e0e0;
    --header-background: #ffffff;
    --footer-background: #f0f0f0;
  }

  .theme-dark {
    --background-color: #2f2f2f;
    --card-background: #3f3f3f;
    --text-color: #f6f6f6;
    --button-color: #00789e;
    --border-color: #4a4a4a;
    --header-background: #252525;
    --footer-background: #303030;
  }

  * {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  }

  .rezlauncher {
    display: flex;
    flex-direction: column;
    height: 100vh;
    color: var(--text-color);
    background-color: var(--background-color);
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 20px;
    background: var(--header-background);
    border-bottom: 1px solid var(--border-color);
  }

  .title h1 {
    color: var(--primary-color);
    font-size: 24px;
  }

  .mode-selection {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .mode-selection label {
    display: flex;
    align-items: center;
    gap: 5px;
    cursor: pointer;
  }

  .breadcrumb {
    display: flex;
    align-items: center;
    padding: 8px 20px;
    background-color: rgba(0, 0, 0, 0.05);
    gap: 10px;
  }

  .uri-label {
    font-weight: 500;
    color: var(--primary-color);
  }

  .dropdown {
    position: relative;
  }

  .dropdown-button {
    padding: 5px 10px;
    background: var(--card-background);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-width: 120px;
    font-size: 14px;
  }

  .dropdown-button::after {
    content: "▾";
    margin-left: 8px;
    font-size: 10px;
  }

  .dropdown-menu {
    position: absolute;
    top: 100%;
    left: 0;
    background: var(--card-background);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    box-shadow: 0 2px 5px rgba(0, 0, 0, 0.1);
    z-index: 10;
    list-style: none;
    padding: 0;
    margin: 2px 0 0 0;
    width: 100%;
    max-height: 200px;
    overflow-y: auto;
  }

  .dropdown-item {
    display: block;
    width: 100%;
    padding: 8px 12px;
    cursor: pointer;
    font-size: 14px;
    text-align: left;
    border: none;
    background: none;
    color: inherit;
    transition: background-color 0.1s;
  }

  .dropdown-item:hover {
    background-color: rgba(0, 153, 204, 0.1);
  }

  .dropdown-item.selected {
    background-color: rgba(0, 153, 204, 0.2);
    font-weight: 500;
  }

  .nav-buttons {
    display: flex;
    gap: 5px;
  }

  .icon-button {
    background: none;
    border: none;
    cursor: pointer;
    padding: 5px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background-color 0.2s;
  }

  .icon-button:hover {
    background-color: rgba(0, 0, 0, 0.1);
  }

  .add-option-form {
    display: flex;
    align-items: center;
    gap: 5px;
  }

  .add-option-form input {
    padding: 5px 10px;
    border: 1px solid var(--primary-color);
    border-radius: 4px;
    font-size: 14px;
    min-width: 180px;
  }

  .add-option-form .add-btn {
    padding: 5px 10px;
    background-color: var(--primary-color);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .add-option-form .cancel-btn {
    padding: 5px 10px;
    background-color: #f0f0f0;
    color: #666;
    border: 1px solid #ddd;
    border-radius: 4px;
    cursor: pointer;
  }

  .add-new-option {
    border-top: 1px dashed var(--border-color) !important;
    color: var(--primary-color);
    font-weight: 500;
  }

  .plus-icon {
    font-weight: bold;
    margin-right: 4px;
  }

  .content-area {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .left-panel, .right-panel {
    flex: 1;
    padding: 15px;
    overflow-y: auto;
    border-right: 1px solid var(--border-color);
  }

  footer {
    padding: 10px 20px;
    background: var(--footer-background);
    border-top: 1px solid var(--border-color);
    height: 200px;
    display: flex;
    flex-direction: column;
  }

  .logs {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .logs h3 {
    font-size: 14px;
    margin-bottom: 5px;
  }

  .logs-container {
    background: var(--card-background);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    height: calc(100% - 25px);
    overflow-y: auto;
    padding: 5px 0;
  }

  .log-entry {
    padding: 3px 10px;
    font-family: monospace;
    font-size: 13px;
    line-height: 1.3;
    display: flex;
    align-items: flex-start;
  }

  .log-icon {
    margin-right: 8px;
    flex-shrink: 0;
  }

  .log-success .log-icon {
    color: #00cc66;
  }

  .log-warning .log-icon {
    color: #ff9900;
  }

  .log-error .log-icon {
    color: #ff3300;
  }

  @media (max-width: 768px) {
    .content-area {
      flex-direction: column;
    }

    .left-panel, .right-panel {
      width: 100%;
      border-right: none;
      border-bottom: 1px solid var(--border-color);
    }
  }
</style>
