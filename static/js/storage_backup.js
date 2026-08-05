class StorageBackupManager {
    constructor() {
        this.init();
    }

    init() {
        const btnSaveNas = document.getElementById('btn-save-nas');
        const btnExportBackup = document.getElementById('btn-export-backup');
        const fileRestore = document.getElementById('file-restore-input');

        if (btnSaveNas) btnSaveNas.addEventListener('click', () => this.saveNasConfig());
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
        try {
            const resp = await fetch('/api/backup/export');
            const data = await resp.json();

            const jsonStr = JSON.stringify(data, null, 2);
            const blob = new Blob([jsonStr], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            
            const a = document.createElement('a');
            a.href = url;
            a.download = `ble_opsec_backup_${new Date().toISOString().substring(0, 10)}.json`;
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

        const reader = new FileReader();
        reader.onload = async (e) => {
            try {
                const json = JSON.parse(e.target.result);
                const resp = await fetch('/api/backup/restore', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(json)
                });
                const data = await resp.json();
                if (data.success) {
                    alert("Backup restored successfully! Database and configurations updated.");
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
