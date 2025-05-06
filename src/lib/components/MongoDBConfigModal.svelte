<script>
  import { createEventDispatcher } from 'svelte';
  import { invoke } from "@tauri-apps/api/tauri";
  import { getCookie, setCookie } from "../utils/cookies";

  const { isOpen = false, errorMessage = "" } = $props();

  const dispatch = createEventDispatcher();

  let mongoURI = $state("");
  let connectionStatus = $state("");
  let isConnecting = $state(false);

  $effect(() => {
    if (isOpen && !mongoURI) {
      loadSavedURI();
    }
  });

  async function loadSavedURI() {
    const savedURI = await getCookie("mongoURI");
    if (savedURI) {
      mongoURI = savedURI;
    } else {
      mongoURI = "mongodb://localhost:27017";
    }
  }

  async function testConnection() {
    if (!mongoURI) {
      connectionStatus = "Please enter a MongoDB URI";
      return;
    }

    try {
      isConnecting = true;
      connectionStatus = "Testing connection...";

      const result = await invoke("test_mongodb_connection", { mongoUri: mongoURI });

      if (result) {
        connectionStatus = "Connection successful!";
        await setCookie("mongoURI", mongoURI, 365); // Save for 1 year

        // Envoyer l'événement puis fermer la modale après un court délai
        setTimeout(() => {
          dispatch("success", { mongoURI });
          close(); // Fermer explicitement la modale
        }, 1000);
      } else {
        connectionStatus = "Connection failed. Please check your MongoDB URI.";
      }
    } catch (error) {
      connectionStatus = `Error: ${error}`;
    } finally {
      isConnecting = false;
    }
  }

  function close() {
    dispatch('close');
  }

  function handleBackdropClick(e) {
    if (e.target === e.currentTarget) {
      close();
    }
  }

  function handleModalKeydown(e) {
    if (e.key === 'Escape') {
      close();
    }
  }
</script>

<div
  class="modal-backdrop"
  class:open={isOpen}
  onclick={handleBackdropClick}
  onkeydown={handleModalKeydown}
  role="dialog"
  aria-modal="true"
  aria-labelledby="mongodb-config-title"
  tabindex="-1"
>
  <div class="modal" role="document">
    <div class="modal-header">
      <h2 id="mongodb-config-title">MongoDB Configuration</h2>
      <button
        type="button"
        class="close-button"
        onclick={close}
        aria-label="Close"
      >&times;</button>
    </div>

    <div class="modal-body">
      {#if errorMessage}
        <div class="error-message">
          <p>Please check your MongoDB configuration.</p>
        </div>
      {/if}

      <div class="form-group">
        <label for="mongoURI">MongoDB URI:</label>
        <input
          type="text"
          id="mongoURI"
          placeholder="mongodb://username:password@host:port/database"
          bind:value={mongoURI}
        />
        <small>Example: mongodb://localhost:27017</small>
      </div>

      {#if connectionStatus}
        <div class="status-message" class:success={connectionStatus.includes("successful")} role="status">
          {connectionStatus}
        </div>
      {/if}
    </div>

    <div class="modal-footer">
      <button
        type="button"
        class="primary-button"
        onclick={testConnection}
        disabled={isConnecting}
      >
        {isConnecting ? 'Connecting...' : 'Connect'}
      </button>
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    display: none;
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.5);
    z-index: 1000;
    justify-content: center;
    align-items: center;
  }

  .modal-backdrop.open {
    display: flex;
  }

  .modal {
    background-color: var(--card-background, white);
    width: 500px;
    max-width: 90%;
    border-radius: 8px;
    box-shadow: 0 5px 15px rgba(0, 0, 0, 0.3);
    color: var(--text-color, #333);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 15px 20px;
    border-bottom: 1px solid var(--border-color, #eee);
  }

  .modal-header h2 {
    margin: 0;
    font-size: 20px;
  }

  .close-button {
    background: none;
    border: none;
    font-size: 24px;
    cursor: pointer;
    color: var(--text-color, #666);
  }

  .modal-body {
    padding: 20px;
  }

  .form-group {
    margin-bottom: 20px;
  }

  .form-group label {
    display: block;
    margin-bottom: 5px;
    font-weight: 500;
  }

  .form-group input {
    width: 100%;
    padding: 10px;
    border: 1px solid var(--border-color, #ccc);
    border-radius: 4px;
    font-size: 16px;
    background-color: var(--card-background, white);
    color: var(--text-color, #333);
  }

  .form-group small {
    display: block;
    margin-top: 5px;
    color: var(--text-color, #666);
    font-size: 12px;
  }

  .modal-footer {
    padding: 15px 20px;
    border-top: 1px solid var(--border-color, #eee);
    text-align: right;
  }

  .primary-button {
    padding: 8px 16px;
    background-color: var(--primary-color, #0099cc);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
  }

  .primary-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .error-message {
    margin-bottom: 20px;
    padding: 10px;
    background-color: rgba(255, 0, 0, 0.1);
    border: 1px solid rgba(255, 0, 0, 0.3);
    border-radius: 4px;
    color: #d32f2f;
  }

  .status-message {
    margin-top: 10px;
    padding: 10px;
    background-color: rgba(0, 0, 0, 0.05);
    border-radius: 4px;
  }

  .status-message.success {
    background-color: rgba(76, 175, 80, 0.1);
    color: #388e3c;
  }
</style>