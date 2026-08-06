class StorageBackupManager {
    constructor() {
        this.init();
    }

    init() {
        const btnSaveNas = document.getElementById('btn-save-nas');
        const btnTestNas = document.getElementById('btn-test-nas');
        const btnExportBackup = document.getElementById('btn-export-backup');
        const fileRestore = document.getElementById('file-restore-input');

        if (btnSaveNas) btnSaveNas.addEventListener('click', () => this.saveNasConfig());
        if (btnTestNas) btnTestNas.addEventListener('click', () => this.testNasConnection('settings'));
        if (btnExportBackup) btnExportBackup.addEventListener('click', () => this.exportBackup());
        if (fileRestore) fileRestore.addEventListener('change', (e) => this.restoreBackup(e));

        this.loadStorageInfo();
    }

    async loadStorageInfo() {
        try {
            const resp = await fetch('/api/system/storage/space');
            const data = await resp.json();

            const spaceVal = document.getElementById('val-free-space');
            if (spaceVal) {
                spaceVal.textContent = `${data.free_space_gb.toFixed(2)} GB Free`;
            }

            if (data.low_space_alert) {
                const alertEl = document.getElementById('storage-space-alert');
                if (alertEl) alertEl.classList.remove('hidden');
            }
        } catch (e) {
            console.error("Failed to load storage info:", e);
        }
    }

    async testNasConnection(context = 'settings') {
        const isWizard = context === 'wizard';
        const protocolEl = document.getElementById(isWizard ? 'wizard-nas-protocol' : 'nas-protocol-select');
        const hostEl = document.getElementById(isWizard ? 'wizard-nas-host' : 'nas-host-input');
        const portEl = document.getElementById(isWizard ? 'wizard-nas-port' : 'nas-port-input');
        const userEl = document.getElementById(isWizard ? 'wizard-nas-user' : 'nas-user-input');
        const pathEl = document.getElementById(isWizard ? 'wizard-nas-path' : 'nas-path-input');

        const resultContainer = document.getElementById(isWizard ? 'wizard-nas-test-result' : 'nas-test-result');
        const titleEl = document.getElementById(isWizard ? 'wizard-nas-test-title' : 'nas-test-title');
        const msgEl = document.getElementById(isWizard ? 'wizard-nas-test-msg' : 'nas-test-msg');

        if (!hostEl || !hostEl.value.trim()) {
            alert("Please enter a NAS Host IP or hostname first.");
            return;
        }

        const payload = {
            protocol: protocolEl ? protocolEl.value : 'SFTP',
            host: hostEl.value.trim(),
            port: parseInt(portEl ? portEl.value : '22') || 22,
            username: userEl ? userEl.value.trim() : 'nas_user',
            remote_path: pathEl ? pathEl.value.trim() : '/volume1/ble_backups'
        };

        if (resultContainer) {
            resultContainer.classList.remove('hidden');
            resultContainer.className = "p-3 rounded-xl border border-cyan-500/30 bg-cyan-500/10 text-cyan-400 text-xs font-mono space-y-1";
            if (titleEl) titleEl.innerHTML = `<span class="animate-spin inline-block">🌀</span> Testing connection to ${payload.protocol}://${payload.host}:${payload.port}...`;
            if (msgEl) msgEl.textContent = "Probing remote socket address...";
        }

        try {
            const resp = await fetch('/api/system/nas/test', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload)
            });
            const data = await resp.json();

            if (resultContainer) {
                if (data.success) {
                    resultContainer.className = "p-3 rounded-xl border border-emerald-500/40 bg-emerald-500/10 text-emerald-400 text-xs font-mono space-y-1";
                    if (titleEl) titleEl.innerHTML = `✓ Connection Successful (${data.protocol} OK)`;
                    if (msgEl) msgEl.textContent = data.message;
                } else {
                    resultContainer.className = "p-3 rounded-xl border border-rose-500/40 bg-rose-500/10 text-rose-400 text-xs font-mono space-y-1";
                    if (titleEl) titleEl.innerHTML = `✕ Connection Failed (${data.protocol})`;
                    if (msgEl) msgEl.textContent = data.error;
                }
            }
        } catch (e) {
            if (resultContainer) {
                resultContainer.className = "p-3 rounded-xl border border-rose-500/40 bg-rose-500/10 text-rose-400 text-xs font-mono space-y-1";
                if (titleEl) titleEl.innerHTML = `✕ Connection Error`;
                if (msgEl) msgEl.textContent = e.message;
            }
        }
    }

    async saveNasConfig() {
        const protocol = document.getElementById('nas-protocol-select').value;
        const host = document.getElementById('nas-host-input').value;
        const port = parseInt(document.getElementById('nas-port-input').value) || 22;
        const user = document.getElementById('nas-user-input').value;
        const remotePath = document.getElementById('nas-path-input').value;
        const autoOffload = document.getElementById('nas-auto-offload').checked;

        const payload = {
            enabled: true,
            protocol,
            host,
            port,
            username: user,
            remote_path: remotePath,
            auto_backup_low_space: autoOffload,
            low_space_threshold_mb: 2000
        };

        try {
            const resp = await fetch('/api/system/nas/config', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload)
            });
            const data = await resp.json();
            if (data.success) {
                alert(`NAS Remote Storage (${protocol}) configured successfully! Auto-offload enabled.`);
            }
        } catch (e) {
            alert(`Failed to save NAS config: ${e.message}`);
        }
    }

    async exportBackup() {
        const scopeSelect = document.getElementById('backup-scope-select');
        const scope = scopeSelect ? scopeSelect.value : 'everything';

        try {
            const resp = await fetch(`/api/backup/export?scope=${encodeURIComponent(scope)}`);
            const data = await resp.json();

            const jsonStr = JSON.stringify(data, null, 2);
            const blob = new Blob([jsonStr], { type: 'application/json' });
            const url = URL.createObjectURL(blob);

            const dateStr = new Date().toISOString().substring(0, 10);
            const a = document.createElement('a');
            a.href = url;
            a.download = `ble_opsec_backup_${scope}_${dateStr}.json`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
        } catch (e) {
            alert(`Export failed: ${e.message}`);
        }
    }

    async restoreBackup(event) {
        const file = event.target.files[0];
        if (!file) return;

        const restoreScopeSelect = document.getElementById('restore-scope-select');
        const selectedScope = restoreScopeSelect ? restoreScopeSelect.value : 'auto';

        const reader = new FileReader();
        reader.onload = async (e) => {
            try {
                const json = JSON.parse(e.target.result);

                if (selectedScope !== 'auto') {
                    json.scope = selectedScope;
                }

                const resp = await fetch('/api/backup/restore', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(json)
                });
                const data = await resp.json();
                if (data.success) {
                    alert(`Backup restored successfully!\nRestored scope: ${data.scope_restored}\nRestored components: ${data.restored_components.join(', ')}`);
                    location.reload();
                }
            } catch (err) {
                alert(`Invalid backup file: ${err.message}`);
            }
        };
        reader.readAsText(file);
    }
}

window.StorageBackupManager = StorageBackupManager;
