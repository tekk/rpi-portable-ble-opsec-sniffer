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

        // Setup Sensitivity Slider Hint Listener
        const sensInput = document.getElementById('wizard-sensitivity-input');
        if (sensInput) {
            sensInput.addEventListener('input', (e) => this.updateSensitivityHint(parseInt(e.target.value)));
            this.updateSensitivityHint(parseInt(sensInput.value));
        }

        // Setup Camera Video Output Test Button
        this.setupCameraTest();

        if (!localStorage.getItem('wizard_completed')) {
            setTimeout(() => this.showWizard(), 1000);
        }
    }

    setupCameraTest() {
        const btnTest = document.getElementById('btn-test-camera');
        const camSelect = document.getElementById('wizard-camera-select');
        const resultPanel = document.getElementById('camera-test-result');
        const badge = document.getElementById('camera-test-badge');
        const specs = document.getElementById('camera-test-specs');
        const preview = document.getElementById('camera-test-preview');

        if (!btnTest || !camSelect) return;

        btnTest.addEventListener('click', async () => {
            const dev = camSelect.value || '/dev/video0';
            btnTest.disabled = true;
            btnTest.innerHTML = `<span class="animate-spin inline-block">🌀</span> Testing...`;

            if (resultPanel) resultPanel.classList.remove('hidden');
            if (badge) badge.className = "font-bold text-cyan-400";
            if (badge) badge.textContent = `Testing video output for ${dev}...`;
            if (specs) specs.textContent = "Probing V4L2 signal...";

            try {
                const resp = await fetch('/api/system/camera/test', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ device: dev })
                });
                const data = await resp.json();

                if (data.success) {
                    if (badge) {
                        badge.className = "font-bold text-emerald-400";
                        badge.textContent = `✓ ${data.status}`;
                    }
                    if (specs) specs.textContent = `${data.resolution} @ ${data.fps} FPS`;
                    if (preview && data.preview_url) {
                        preview.src = data.preview_url;
                    }
                } else {
                    if (badge) {
                        badge.className = "font-bold text-rose-400";
                        badge.textContent = `❌ Video Output Test Failed`;
                    }
                    if (specs) specs.textContent = data.error || "Device not outputting video";
                    if (preview) preview.src = "";
                }
            } catch (e) {
                if (badge) {
                    badge.className = "font-bold text-rose-400";
                    badge.textContent = `❌ Test Error`;
                }
                if (specs) specs.textContent = e.message;
            } finally {
                btnTest.disabled = false;
                btnTest.innerHTML = `<span>📹 Test</span>`;
            }
        });
    }

    updateSensitivityHint(val) {
        const label = document.getElementById('wizard-sensitivity-label');
        const hint = document.getElementById('wizard-sensitivity-hint');
        let text = "Medium";

        if (val <= 9) text = "Very high";
        else if (val <= 14) text = "High";
        else if (val <= 19) text = "Medium";
        else if (val <= 24) text = "Low";
        else text = "Very low";

        if (label) label.textContent = text;
        if (hint) hint.textContent = `Sensitivity: ${text}`;
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

    async nextStep() {
        // If on Step 2 (BLE & OUI), execute actual IEEE OUI Database Download with progress bar
        if (this.currentStep === 2) {
            await this.downloadOuiDatabaseInWizard();
        }

        if (this.currentStep < this.totalSteps) {
            this.currentStep++;
            this.updateStepVisibility();
        } else {
            this.finishWizard();
        }
    }

    async downloadOuiDatabaseInWizard() {
        const container = document.getElementById('oui-download-container');
        const progressBar = document.getElementById('oui-download-progress-bar');
        const percentText = document.getElementById('oui-download-percent');
        const statusText = document.getElementById('oui-download-status');
        const nextBtn = document.getElementById('wizard-btn-next');

        if (!container || !progressBar) return;

        container.classList.remove('hidden');
        if (nextBtn) nextBtn.disabled = true;

        let progress = 10;
        const interval = setInterval(() => {
            if (progress < 90) {
                progress += Math.floor(Math.random() * 15) + 5;
                if (progress > 90) progress = 90;
                progressBar.style.width = `${progress}%`;
                percentText.textContent = `${progress}%`;
            }
        }, 150);

        try {
            statusText.innerHTML = `<span class="animate-spin text-cyan-400 inline-block">🌀</span> Downloading official IEEE OUI database mirror...`;
            const resp = await fetch('/api/oui/update', { method: 'POST' });
            const data = await resp.json();

            clearInterval(interval);
            progressBar.style.width = `100%`;
            percentText.textContent = `100%`;

            const vendorCount = (data.updated_vendors !== undefined) ? data.updated_vendors : (data.updated_count || 0);
            statusText.innerHTML = `<span class="text-emerald-400 font-bold">✓ Successfully loaded ${vendorCount.toLocaleString()} IEEE OUI vendor records!</span>`;
            
            await new Promise(res => setTimeout(res, 800));
        } catch (e) {
            clearInterval(interval);
            console.error("OUI download wizard error:", e);
            statusText.innerHTML = `<span class="text-amber-400 font-bold">⚠️ Using 50+ built-in offline vendor OUI rules.</span>`;
            await new Promise(res => setTimeout(res, 800));
        } finally {
            if (nextBtn) nextBtn.disabled = false;
        }
    }

    prevStep() {
        if (this.currentStep > 1) {
            this.currentStep--;
            this.updateStepVisibility();
        }
    }

    updateStepVisibility() {
        for (let i = 1; i <= this.totalSteps; i++) {
            const stepEl = document.getElementById(`wizard-step-${i}`);
            if (stepEl) {
                if (i === this.currentStep) {
                    stepEl.classList.remove('hidden');
                } else {
                    stepEl.classList.add('hidden');
                }
            }
        }

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
