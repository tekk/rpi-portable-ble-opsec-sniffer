class BleRadar {
    constructor(canvasId) {
        this.canvas = document.getElementById(canvasId);
        this.ctx = this.canvas.getContext('2d');
        this.angle = 0;
        this.devices = new Map(); // MAC -> { rssi, angle, lastSeen }

        this.init();
    }

    init() {
        this.animate();
    }

    updateDevice(mac, rssi, type) {
        let entry = this.devices.get(mac);
        if (!entry) {
            // Assign random polar angle on radar
            entry = {
                mac,
                rssi,
                type,
                angle: Math.random() * Math.PI * 2,
                lastSeen: Date.now()
            };
        } else {
            entry.rssi = rssi;
            entry.lastSeen = Date.now();
        }
        this.devices.set(mac, entry);

        // Update badge count
        document.getElementById('radar-count').textContent = `${this.devices.size} Active Devices`;
    }

    animate() {
        const width = this.canvas.width;
        const height = this.canvas.height;
        const centerX = width / 2;
        const centerY = height / 2;
        const maxRadius = width / 2 - 15;

        // Clear canvas
        this.ctx.fillStyle = 'rgba(10, 14, 23, 0.25)';
        this.ctx.fillRect(0, 0, width, height);

        // Draw distance rings (-30, -60, -90 dBm)
        this.ctx.strokeStyle = 'rgba(0, 242, 254, 0.15)';
        this.ctx.lineWidth = 1;

        for (let i = 1; i <= 3; i++) {
            this.ctx.beginPath();
            this.ctx.arc(centerX, centerY, (maxRadius / 3) * i, 0, Math.PI * 2);
            this.ctx.stroke();
        }

        // Draw crosshairs
        this.ctx.beginPath();
        this.ctx.moveTo(centerX, 15);
        this.ctx.lineTo(centerX, height - 15);
        this.ctx.moveTo(15, centerY);
        this.ctx.lineTo(width - 15, centerY);
        this.ctx.stroke();

        // Draw radar sweep beam
        this.angle += 0.03;
        if (this.angle > Math.PI * 2) this.angle = 0;

        const sweepX = centerX + Math.cos(this.angle) * maxRadius;
        const sweepY = centerY + Math.sin(this.angle) * maxRadius;

        const gradient = this.ctx.createLinearGradient(centerX, centerY, sweepX, sweepY);
        gradient.addColorStop(0, 'rgba(0, 242, 254, 0.4)');
        gradient.addColorStop(1, 'rgba(0, 242, 254, 0.0)');

        this.ctx.beginPath();
        this.ctx.moveTo(centerX, centerY);
        this.ctx.arc(centerX, centerY, maxRadius, this.angle - 0.2, this.angle);
        this.ctx.closePath();
        this.ctx.fillStyle = gradient;
        this.ctx.fill();

        // Plot BLE device blips
        const now = Date.now();
        this.devices.forEach((dev, mac) => {
            // Expire old devices (> 60s)
            if (now - dev.lastSeen > 60000) {
                this.devices.delete(mac);
                return;
            }

            // Map RSSI (-30 dBm to -95 dBm) to radius distance
            const normalizedRssi = Math.min(Math.max(dev.rssi, -95), -30);
            const distRatio = (normalizedRssi - (-30)) / (-95 - (-30)); // 0.0 near, 1.0 far
            const radius = Math.max(distRatio * maxRadius, 20);

            const x = centerX + Math.cos(dev.angle) * radius;
            const y = centerY + Math.sin(dev.angle) * radius;

            // Blip color based on device type
            let color = '#00f2fe'; // Default blue
            if (dev.type.includes('Tracker') || dev.type.includes('AirTag')) color = '#ff0844'; // Red alert
            if (dev.type.includes('iPhone') || dev.type.includes('Android')) color = '#c084fc'; // Purple
            if (dev.type.includes('ESP32') || dev.type.includes('nRF')) color = '#00e676'; // Green

            // Draw blip dot
            this.ctx.beginPath();
            this.ctx.arc(x, y, 4, 0, Math.PI * 2);
            this.ctx.fillStyle = color;
            this.ctx.shadowColor = color;
            this.ctx.shadowBlur = 8;
            this.ctx.fill();
            this.ctx.shadowBlur = 0;
        });

        requestAnimationFrame(() => this.animate());
    }
}

window.BleRadar = BleRadar;
