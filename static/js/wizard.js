class SetupWizard {
    constructor() {
        this.wizardEl = document.getElementById('setup-wizard-modal');
        this.step1 = document.getElementById('wizard-step-1');
        this.step2 = document.getElementById('wizard-step-2');
        this.step3 = document.getElementById('wizard-step-3');
        this.step4 = document.getElementById('wizard-step-4');

        this.currentStep = 1;
        this.selectedCamera = '/dev/video0';
        this.selectedAdapter = 'hci0';
        this.password = '';
        this.motionSensitivity = 15;
        this.preRollSecs = 10;

        this.init();
    }

    async init() {
        // Add wizard trigger button listener
        const btnRunWizard = document.getElementById('btn-run-wizard');
        if (btnRunWizard) {
            btnRunWizard.addEventListener('click', () => this.showWizard());
        }

        // Show automatically on first run
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
        if (this.currentStep < 4) {
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
        [this.step1, this.step2, this.step3, this.step4].forEach((s, idx) => {
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
            progressEl.style.width = `${(this.currentStep / 4) * 100}%`;
        }
    }

    async finishWizard() {
        const camSelect = document.getElementById('wizard-camera-select');
        const hciSelect = document.getElementById('wizard-hci-select');
        const passInput = document.getElementById('wizard-password-input');
        const sensInput = document.getElementById('wizard-sensitivity-input');

        const payload = {
            camera_path: camSelect ? camSelect.value : '/dev/video0',
            hci_adapter: hciSelect ? hciSelect.value : 'hci0',
            password_protection: passInput ? passInput.value : '',
            motion_sensitivity: sensInput ? parseInt(sensInput.value) : 15,
            completed_at: new Date().toISOString()
        };

        try {
            await fetch('/api/system/config', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload)
            });
            localStorage.setItem('wizard_completed', 'true');
            this.wizardEl.classList.add('hidden');
            alert("Configuration saved successfully! Sniffer active with updated sources.");
        } catch (e) {
            console.error("Failed to save wizard config:", e);
        }
    }

    closeWizard() {
        this.wizardEl.classList.add('hidden');
    }
}

window.SetupWizard = SetupWizard;
