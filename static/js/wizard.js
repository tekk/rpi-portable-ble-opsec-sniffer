class SetupWizard {
    constructor() {
        this.wizardEl = document.getElementById('setup-wizard-modal');
        this.step1 = document.getElementById('wizard-step-1');
        this.step2 = document.getElementById('wizard-step-2');
        this.step3 = document.getElementById('wizard-step-3');
        this.step4 = document.getElementById('wizard-step-4');
        this.step5 = document.getElementById('wizard-step-5');

        this.currentStep = 1;
        this.totalSteps = 5;

        this.init();
    }

    async init() {
        const btnRunWizard = document.getElementById('btn-run-wizard');
        if (btnRunWizard) {
            btnRunWizard.addEventListener('click', () => this.showWizard());
        }

        if (!localStorage.getItem('wizard_completed')) {
            setTimeout(() => this.showWizard(), 1000);
        }
    }

    async showWizard() {
        this.currentStep = 1;
        this.updateStepVisibility();
        this.wizardEl.classList.remove('hidden');

        try {
            const resp = await fetch('/api/system/sources');
            const data = await resp.json();
            this.populateSources(data);
        } catch (e) {
            console.error("Failed to load hardware sources:", e);
        }
    }

    populateSources(data) {
        const camSelect = document.getElementById('wizard-camera-select');
        const hciSelect = document.getElementById('wizard-hci-select');

        if (camSelect && data.cameras) {
            camSelect.innerHTML = data.cameras.map(c => `<option value="${c}">${c} (V4L2 Video Device)</option>`).join('');
        }
        if (hciSelect && data.hci_adapters) {
            hciSelect.innerHTML = data.hci_adapters.map(h => `<option value="${h}">${h} (Bluetooth Controller)</option>`).join('');
        }
    }

    nextStep() {
        if (this.currentStep < this.totalSteps) {
            this.currentStep++;
            this.updateStepVisibility();
        } else {
            this.finishWizard();
        }
    }

    prevStep() {
        if (this.currentStep > 1) {
            this.currentStep--;
            this.updateStepVisibility();
        }
    }

    updateStepVisibility() {
        const steps = [this.step1, this.step2, this.step3, this.step4, this.step5];
        steps.forEach((s, idx) => {
            if (s) {
                if (idx + 1 === this.currentStep) {
                    s.classList.remove('hidden');
                } else {
                    s.classList.add('hidden');
                }
            }
        });

        const progressEl = document.getElementById('wizard-progress-bar');
        if (progressEl) {
            progressEl.style.width = `${(this.currentStep / this.totalSteps) * 100}%`;
        }

        const nextBtn = document.getElementById('wizard-btn-next');
        if (nextBtn) {
            nextBtn.textContent = this.currentStep === this.totalSteps ? "Complete Setup ✓" : "Next Step →";
        }
    }

    async finishWizard() {
        const cameraEnabled = document.getElementById('wizard-camera-enable-toggle')?.checked ?? true;
        const camSelect = document.getElementById('wizard-camera-select')?.value || '/dev/video0';
        const hciSelect = document.getElementById('wizard-hci-select')?.value || 'hci0';
        const passInput = document.getElementById('wizard-password-input')?.value || '';
        const sensInput = parseInt(document.getElementById('wizard-sensitivity-input')?.value || '15');
        
        // NAS Config
        const nasProtocol = document.getElementById('wizard-nas-protocol')?.value || 'SFTP';
        const nasHost = document.getElementById('wizard-nas-host')?.value || '';
        const nasPort = parseInt(document.getElementById('wizard-nas-port')?.value || '22');
        const nasUser = document.getElementById('wizard-nas-user')?.value || '';
        const nasPath = document.getElementById('wizard-nas-path')?.value || '';
        const nasAutoOffload = document.getElementById('wizard-nas-auto-offload')?.checked ?? true;

        // MQTT Config
        const mqttEnabled = document.getElementById('wizard-mqtt-enable')?.checked ?? false;
        const mqttHost = document.getElementById('wizard-mqtt-host')?.value || 'localhost';
        const mqttPort = parseInt(document.getElementById('wizard-mqtt-port')?.value || '1883');
        const mqttTopic = document.getElementById('wizard-mqtt-topic')?.value || 'homeassistant/rpi_ble_opsec';

        const payload = {
            camera_enabled: cameraEnabled,
            camera_path: camSelect,
            hci_adapter: hciSelect,
            password_protection: passInput,
            motion_sensitivity: sensInput,
            nas_config: {
                enabled: !!nasHost,
                protocol: nasProtocol,
                host: nasHost,
                port: nasPort,
                username: nasUser,
                remote_path: nasPath,
                auto_backup_low_space: nasAutoOffload
            },
            mqtt_config: {
                enabled: mqttEnabled,
                host: mqttHost,
                port: mqttPort,
                topic: mqttTopic
            },
            completed_at: new Date().toISOString()
        };

        try {
            // Apply camera dynamic toggle state immediately
            await fetch('/api/system/camera/toggle', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ enabled: cameraEnabled })
            });

            // Save full config
            await fetch('/api/system/config', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload)
            });

            // Save NAS config if host specified
            if (nasHost) {
                await fetch('/api/system/nas/config', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(payload.nas_config)
                });
            }

            localStorage.setItem('wizard_completed', 'true');
            this.wizardEl.classList.add('hidden');
            alert("All setup options saved successfully! Application fully configured.");
            location.reload();
        } catch (e) {
            console.error("Failed to save wizard config:", e);
            alert(`Setup error: ${e.message}`);
        }
    }

    closeWizard() {
        this.wizardEl.classList.add('hidden');
    }
}

window.SetupWizard = SetupWizard;
