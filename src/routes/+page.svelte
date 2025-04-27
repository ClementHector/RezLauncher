<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import SectionPanel from "$lib/components/SectionPanel.svelte";
  import PackageModal from "$lib/components/PackageModal.svelte";

  // State management
  let mode = $state("Developer"); // "Default" or "Developer"
  let theme = $state("Light"); // "Light" or "Dark"
  let logs = $state([]); // Array of log messages
  let packageModalOpen = $state(false); // State for package creation modal
  let isEditingPackage = $state(false); // State to track if we're editing or creating
  let currentPackageData = $state(null); // Store the current package being edited
  let currentUsername = $state(""); // Store the current OS username
  let packageCollections = $state([]); // Store package collections
  let packageCollectionMessage = $state(""); // Message when no collections are found
  let stageMessage = $state(""); // Message when no stages are found
  let allUris = $state([]); // Store all URIs from MongoDB

  // Message to display when no URI is selected
  let noUriSelectedMessage = "Select a URI to view stages or package collections";

  // URI global variable
  let currentUri = $state("/project"); // URI global pour toute l'interface

  // Navigation state
  let projectSelection = $state("select");
  let modelingSelection = $state("select");
  let applicationSelection = $state("select");

  // Fonction pour construire l'URI basé sur les sélections actuelles
  function buildCurrentUri() {
    if (projectSelection === "select") {
      return "";
    }

    if (!showModelingCombobox || modelingSelection === "select") {
      return `/${projectSelection}`;
    } else if (!showApplicationCombobox || applicationSelection === "select") {
      return `/${projectSelection}/${modelingSelection}`;
    } else {
      return `/${projectSelection}/${modelingSelection}/${applicationSelection}`;
    }
  }

  // Visibilité des comboboxes
  let showModelingCombobox = $state(false);
  let showApplicationCombobox = $state(false);

  // Mode d'ajout pour chaque combobox
  let isAddingProjectOption = $state(false);
  let isAddingModelingOption = $state(false);
  let isAddingApplicationOption = $state(false);
  let newOptionText = $state("");

  // Options disponibles pour chaque combobox
  let projectOptions = $state([]);
  let modelingOptions = $state([]);
  let applicationOptions = $state([]);

  // État d'ouverture pour chaque combobox
  let projectDropdownOpen = $state(false);
  let modelingDropdownOpen = $state(false);
  let applicationDropdownOpen = $state(false);

  // Fonctions pour gérer les combobox
  function toggleDropdown(dropdownName: string) {
    switch(dropdownName) {
      case 'project':
        projectDropdownOpen = !projectDropdownOpen;
        modelingDropdownOpen = false;
        applicationDropdownOpen = false;
        break;
      case 'modeling':
        modelingDropdownOpen = !modelingDropdownOpen;
        projectDropdownOpen = false;
        applicationDropdownOpen = false;
        break;
      case 'application':
        applicationDropdownOpen = !applicationDropdownOpen;
        projectDropdownOpen = false;
        modelingDropdownOpen = false;
        break;
    }
  }

  function selectOption(dropdownName: string, option: string) {
    switch(dropdownName) {
      case 'project':
        projectSelection = option;
        projectDropdownOpen = false;
        // Reset the child selections when changing projects
        modelingSelection = "select";
        applicationSelection = "select";
        showModelingCombobox = true;
        showApplicationCombobox = false; // Masquer la combobox d'application lorsqu'on change de projet
        break;
      case 'modeling':
        modelingSelection = option;
        modelingDropdownOpen = false;
        // Reset the application selection when changing modeling type
        applicationSelection = "select";
        showApplicationCombobox = true;
        break;
      case 'application':
        applicationSelection = option;
        applicationDropdownOpen = false;
        break;
    }
    addLog(`Selected ${dropdownName}: ${option}`);

    // Fetch package collections whenever the navigation path changes
    fetchPackageCollectionsByUri();
    // Also fetch stages for the current URI
    fetchStagesByUri();
  }

  // Fonction pour ajouter une nouvelle option
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
  }

  // Fonctions pour démarrer l'ajout d'une nouvelle option
  function startAddingOption(dropdownType: string) {
    newOptionText = "";
    isAddingProjectOption = dropdownType === 'project';
    isAddingModelingOption = dropdownType === 'modeling';
    isAddingApplicationOption = dropdownType === 'application';

    // Fermer les dropdowns
    projectDropdownOpen = false;
    modelingDropdownOpen = false;
    applicationDropdownOpen = false;
  }

  // Fonction pour annuler l'ajout
  function cancelAddingOption() {
    isAddingProjectOption = false;
    isAddingModelingOption = false;
    isAddingApplicationOption = false;
    newOptionText = "";
  }

  // Data for stages and packages
  let stages = $state([]);

  // Track active package collection and active stage
  let activePackage = $state(null);
  let activeStage = $state(null);
  let packages = $state([]);

  let tools = $state([]);

  // Toggle dark mode
  function toggleDarkMode() {
    theme = theme === "Light" ? "Dark" : "Light";
  }

  // Handle selecting a package collection
  async function handlePackageClick(item) {
    try {
      // Deselect any active stage when a package is selected
      activeStage = null;

      // Update active status for all packages
      packages = packages.map(pkg => ({
        ...pkg,
        active: pkg.version === item.version
      }));

      // Update active status for all stages (none should be active)
      stages = stages.map(stage => ({
        ...stage,
        active: false
      }));

      // Set the active package
      activePackage = item.version;

      // Log the selection
      addLog(`Selected package collection: ${item.version}`, "info");

      // Fetch tools for this package collection
      await fetchToolsForPackage(item.version);
    } catch (error) {
      addLog(`Error selecting package: ${error}`, "error");
    }
  }

  // Handle selecting a stage
  async function handleStageClick(item) {
    try {
      // Deselect any active package when a stage is selected
      activePackage = null;

      // Update active status for all stages
      stages = stages.map(stage => ({
        ...stage,
        active: stage.name === item.name
      }));

      // Update active status for all packages (none should be active)
      packages = packages.map(pkg => ({
        ...pkg,
        active: false
      }));

      // Set the active stage
      activeStage = item.name;

      // Log the selection
      addLog(`Selected stage: ${item.name}`, "info");

      // Fetch tools for this stage
      await fetchToolsForStage(item);
    } catch (error) {
      addLog(`Error selecting stage: ${error}`, "error");
    }
  }

  // Function to fetch tools for a specific package
  async function fetchToolsForPackage(packageVersion) {
    try {
      addLog(`Fetching tools for package collection: ${packageVersion}`, "info");

      // Rechercher le package collection dans la liste selon la version
      const selectedPackage = packageCollections.find(pkg => pkg.version === packageVersion);

      if (!selectedPackage) {
        addLog(`Package collection ${packageVersion} not found`, "error");
        return;
      }

      // Vérifier si le package a des outils définis
      if (selectedPackage.tools && Array.isArray(selectedPackage.tools) && selectedPackage.tools.length > 0) {
        // Extraire uniquement les noms des outils
        tools = selectedPackage.tools.map(tool => tool.name || tool);

        addLog(`Found ${tools.length} tools in package ${packageVersion}`, "success");
      } else {
        // Si aucun outil n'est défini dans le package, nous affichons un message spécial
        tools = ["No tools for the selection"];
        addLog(`Aucun outil trouvé dans le package ${packageVersion}`, "warning");
      }
    } catch (error) {
      addLog(`Error fetching tools: ${error}`, "error");
      tools = ["No tools for the selection"]; // Afficher le message d'erreur dans la liste d'outils
    }
  }

  // Function to fetch tools for a specific stage
  async function fetchToolsForStage(stage) {
    try {
      addLog(`Fetching tools for stage: ${stage.name}`, "info");

      // Vérifier si le stage a des outils définis
      if (stage.tools && Array.isArray(stage.tools) && stage.tools.length > 0) {
        // Extraire uniquement les noms des outils
        tools = stage.tools.map(tool => tool.name || tool);

        addLog(`Found ${tools.length} tools in stage ${stage.name}`, "success");
      } else {
        // Si aucun outil n'est défini dans le stage, nous affichons un message spécial
        tools = ["No tools for the selection"];
        addLog(`Aucun outil trouvé dans le stage ${stage.name}`, "warning");
      }
    } catch (error) {
      addLog(`Error fetching tools for stage: ${error}`, "error");
      tools = ["No tools for the selection"]; // Afficher le message d'erreur dans la liste d'outils
    }
  }

  // Functions to handle button clicks
  async function loadStage(stageName: string) {
    try {
      // This would call a Tauri command to load the stage
      // await invoke("load_stage", { name: stageName });
      addLog(`Loading stage: ${stageName}`);
      // Find and update the loaded stage
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
      // This would call a Tauri command to revert the stage
      // await invoke("revert_stage", { name: stageName });
      addLog(`Reverting stage: ${stageName}`);
    } catch (error) {
      addLog(`Error reverting stage: ${error}`, "error");
    }
  }

  async function bakePackage(packageName: string) {
    try {
      // This would call a Tauri command to bake the package
      // await invoke("bake_package", { name: packageName });
      addLog(`Baking package: ${packageName}`);
      // Update the package state
      packages = packages.map(pkg =>
        pkg.name === packageName
          ? { ...pkg, baked: true }
          : pkg
      );
    } catch (error) {
      addLog(`Error baking package: ${error}`, "error");
    }
  }

  async function createFromPackage(packageVersion: string) {
    try {
      // Find the package collection to create from our list
      const packageToCreateFrom = packageCollections.find(pkg => pkg.version === packageVersion);

      if (packageToCreateFrom) {
        // Set current package data and open modal in edit mode
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
      // This would call a Tauri command to load the tool
      // await invoke("load_tool", { name: toolName });
      addLog(`Loading tool: ${toolName}`);
      // Find and update the loaded tool
      tools = tools.map(tool =>
        tool.name === toolName
          ? { ...tool, loaded: true }
          : { ...tool, loaded: false }
      );
    } catch (error) {
      addLog(`Error loading tool: ${error}`, "error");
    }
  }

  function addLog(message: string, type: "info" | "success" | "warning" | "error" = "info") {
    const icon = type === "success" ? "✓" :
                type === "warning" ? "⚠" :
                type === "error" ? "✕" : "i";
    logs = [...logs, { message, type, icon }];
  }

  // Open package creation modal
  function createNewPackage() {
    packageModalOpen = true;
    addLog("Opening package collection creation form...");
  }

  // Handle package collection submission
  async function handlePackageSubmit(event: CustomEvent) {
    try {
      const formData = event.detail;

      // Utiliser la fonction buildCurrentUri pour créer l'URI
      const uri = buildCurrentUri();

      if (!uri) {
        addLog("Impossible de créer un package: veuillez sélectionner au moins un projet", "error");
        packageModalOpen = false;
        return;
      }

      // Create a new package collection regardless of whether we're editing or creating
      // Even when using "Create From", we always create a new collection
      const newPackage = {
        ...formData,
        created_at: new Date().toISOString(),
        created_by: currentUsername,
        uri: uri
      };

      // Call Tauri command to store in MongoDB
      await invoke("save_package_collection", { packageData: newPackage });

      // Add to UI list (temporary until we implement loading from MongoDB)
      packages = [...packages, { version: formData.version, baked: false }];

      // Log success
      const actionType = isEditingPackage ? "Created new package collection from existing" : "Created new package collection";
      addLog(`${actionType} with version: ${formData.version} by ${currentUsername}`, "success");

      // Reset form state and close modal
      isEditingPackage = false;
      currentPackageData = null;
      packageModalOpen = false;

      // Refresh the package collections to show the updated data
      fetchPackageCollectionsByUri();
    } catch (error) {
      addLog(`Error saving package collection: ${error}`, "error");
    }
  }

  // Close package modal
  function closePackageModal() {
    packageModalOpen = false;
    isEditingPackage = false; // Reset editing mode
    currentPackageData = null; // Clear current package data
    addLog("Cancelled package collection creation/edit");
  }

  // Function to reset URI path to default values
  function resetToHome() {
    projectSelection = "select";
    modelingSelection = "select";
    applicationSelection = "select";
    // Réinitialiser la visibilité des comboboxes
    showModelingCombobox = false;
    showApplicationCombobox = false;
    addLog("Reset URI path to home", "info");
    currentUri = buildCurrentUri();
  }

  // Function to refresh package collections
  function refreshPackages() {
    addLog("Refreshing package collections and stages...", "info");
    fetchPackageCollectionsByUri();
    fetchStagesByUri(); // Also fetch stages when refreshing
  }

  // Fetch current username on component initialization
  async function fetchCurrentUsername() {
    try {
      const username = await invoke("get_current_username");
      currentUsername = username;
      addLog(`Session user identified: ${username}`, "info");
    } catch (error) {
      addLog(`Failed to get current username: ${error}`, "warning");
      currentUsername = "unknown_user";
    }
  }

  // Function to fetch package collections by URI
  async function fetchPackageCollectionsByUri() {
    try {
      // Utiliser la fonction buildCurrentUri pour construire l'URI actuel
      currentUri = buildCurrentUri();

      if (!currentUri) {
        // Aucun projet sélectionné, URI vide
        addLog(noUriSelectedMessage, "warning");
        packageCollections = [];
        packages = [];
        packageCollectionMessage = noUriSelectedMessage;
        return;
      }

      addLog(`Fetching package collections for URI: ${currentUri}`, "info");

      const result = await invoke("get_package_collections_by_uri", { uri: currentUri });

      if (result.success) {
        if (result.collections) {
          packageCollections = result.collections;
          // Update the packages list to display in the UI - using version instead of name
          packages = packageCollections.map(collection => ({
            version: collection.version, // Utiliser version au lieu de name
            baked: false, // We can enhance this in the future to store bake status
            uri: collection.uri // Ajouter l'URI pour pouvoir l'utiliser lors du Bake
          }));
          packageCollectionMessage = "";
          addLog(`Found ${packageCollections.length} package collections for ${currentUri}`, "success");
        } else {
          // No collections found, display message
          packageCollections = [];
          packages = [];
          packageCollectionMessage = result.message || `no collection found in ${currentUri}`;
          addLog(packageCollectionMessage, "warning");
        }
      } else {
        addLog(`Error fetching package collections: ${result.message}`, "error");
        packageCollectionMessage = `Error: ${result.message}`;
      }
    } catch (error) {
      addLog(`Error fetching package collections: ${error}`, "error");
      packageCollectionMessage = `Error: ${error}`;
    }
  }

  // Function to fetch all available URIs from MongoDB and populate dropdown options
  async function fetchAllUrisAndUpdateOptions() {
    try {
      addLog("Fetching all available URIs from MongoDB...");

      // Call Rust function to get all package collections
      const result = await invoke("get_all_package_collections");

      if (result.success && result.collections) {
        // Store all URIs for future use
        allUris = result.collections.map(collection => collection.uri || "");

        // Create sets to track unique values
        const projectSet = new Set();
        const modelingSet = new Set();
        const appSet = new Set();

        // Add "select" option as default
        projectSet.add("select");
        modelingSet.add("select");
        appSet.add("select");

        // Process each URI to extract components
        allUris.forEach(uri => {
          if (!uri) return;

          const parts = uri.split('/').filter(part => part);

          if (parts.length >= 1) {
            projectSet.add(parts[0]);
          }

          if (parts.length >= 2) {
            modelingSet.add(parts[1]);
          }

          if (parts.length >= 3) {
            appSet.add(parts[2]);
          }
        });

        // Replace static options with dynamically generated ones from MongoDB
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

  // Function to fetch stages by URI
  async function fetchStagesByUri() {
    try {
      // Utiliser la fonction buildCurrentUri pour construire l'URI actuel
      currentUri = buildCurrentUri();

      if (!currentUri) {
        // Aucun projet sélectionné, URI vide
        addLog(noUriSelectedMessage, "warning");
        stages = [];
        stageMessage = noUriSelectedMessage;
        return;
      }

      addLog(`Fetching stages for URI: ${currentUri}`, "info");

      // Use the SectionPanel component's fetchStagesByUri function
      const stagesData = await invoke("get_stages_by_uri", { uri: currentUri });

      if (Array.isArray(stagesData) && stagesData.length > 0) {
        // Transform the data to the format expected by the UI
        stages = stagesData.map(stage => ({
          name: stage.name,
          loaded: false, // Initialize as not loaded
          uri: stage.uri,
          from_version: stage.from_version,
          rxt_path: stage.rxt_path,
          tools: stage.tools
        }));

        stageMessage = ""; // Clear the message when stages are found
        addLog(`Found ${stages.length} stages for ${currentUri}`, "success");
      } else {
        stages = [];
        stageMessage = `No stages found for ${currentUri}`;
        addLog(`No stages found for ${currentUri}`, "warning");
      }
    } catch (error) {
      addLog(`Error fetching stages: ${error}`, "error");
      stages = [];
      stageMessage = `Error: ${error}`;
    }
  }

  // Handle bake completion event
  async function handleBakeComplete(event: CustomEvent) {
    try {
      const { success, data } = event.detail;

      if (success) {
        addLog(`Stage "${data.name}" created successfully from package ${data.from_version}`, "success");
        // Refresh the stages list to show the newly created stage
        await fetchStagesByUri();
      } else {
        addLog(`Failed to create stage: ${event.detail.error}`, "error");
      }
    } catch (error) {
      addLog(`Error handling bake completion: ${error}`, "error");
    }
  }

  // Function to refresh stages
  async function refreshStages() {
    addLog("Refreshing stages...", "info");
    await fetchStagesByUri();
  }

  // Initialize app
  async function initApp() {
    await fetchCurrentUsername();
    await fetchAllUrisAndUpdateOptions(); // Fetch and update dropdown options from MongoDB URIs
    await fetchPackageCollectionsByUri();
    await fetchStagesByUri(); // Also fetch stages during initialization
    addLog("RezLauncher started");
  }

  // Call the initialization function
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
        <input type="radio" bind:group={mode} value="Default">
        Default
      </label>
      <label>
        <input type="radio" bind:group={mode} value="Developer">
        Developer
      </label>
      <label class="theme-toggle">
        <input type="checkbox" checked={theme === "Dark"} onchange={toggleDarkMode}>
        Dark
      </label>
    </div>
  </header>

  <nav class="breadcrumb">
    <span class="uri-label">URI:</span>

    {#if isAddingProjectOption}
      <div class="add-option-form">
        <input
          type="text"
          bind:value={newOptionText}
          placeholder="Nouveau projet..."
          onkeypress={(e) => e.key === 'Enter' && addNewOption('project')}
        />
        <button class="add-btn" onclick={() => addNewOption('project')}>Ajouter</button>
        <button class="cancel-btn" onclick={cancelAddingOption}>Annuler</button>
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
              <span class="plus-icon">+</span> Ajouter...
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
          <button class="add-btn" onclick={() => addNewOption('modeling')}>Ajouter</button>
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
                <span class="plus-icon">+</span> Ajouter...
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
          <button class="add-btn" onclick={() => addNewOption('application')}>Ajouter</button>
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
                <span class="plus-icon">+</span> Ajouter...
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

      {#if mode === "Developer"}
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

  <!-- Package creation/editing modal -->
  <PackageModal
    isOpen={packageModalOpen}
    packageData={currentPackageData}
    on:close={closePackageModal}
    on:submit={handlePackageSubmit}
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
  /* Reset and base styles */
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

  /* Header styles */
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

  /* Navigation/Breadcrumb styles */
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

  .home-button svg, .refresh-button svg {
    width: 16px;
    height: 16px;
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

  /* Content area styles */
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

  /* Section styles - moved to SectionPanel.svelte */

  .expand-icon {
    margin-right: 8px;
    cursor: pointer;
  }

  ul {
    list-style: none;
  }

  .item-name {
    font-size: 14px;
  }

  .action-buttons {
    display: flex;
    gap: 5px;
  }

  .action-button {
    padding: 5px 12px;
    background: var(--button-color);
    color: var(--button-text);
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

  /* Footer/Logs styles */
  footer {
    padding: 10px 20px;
    background: var(--footer-background);
    border-top: 1px solid var(--border-color);
    height: 200px; /* Fixed height for the footer */
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
    height: calc(100% - 25px); /* Full height minus the heading */
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

  /* Responsive adjustments */
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
