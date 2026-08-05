class DeviceRssiChart {
    constructor() {
        this.modalEl = document.getElementById('chart-modal');
        this.canvasEl = document.getElementById('rssiHistoryChart');
        this.titleEl = document.getElementById('chart-modal-title');
        this.trendEl = document.getElementById('chart-trend-badge');
        this.distEl = document.getElementById('chart-distance-val');
        this.chart = null;

        this.init();
    }

    init() {
        const closeBtn = document.getElementById('chart-modal-close');
        if (closeBtn) {
            closeBtn.addEventListener('click', () => this.hideModal());
        }
    }

    async showDeviceHistory(mac, name) {
        this.titleEl.textContent = `Proximity Analytics: ${mac} (${name || 'BLE Device'})`;
        this.modalEl.classList.remove('hidden');

        try {
            const resp = await fetch(`/api/devices/${encodeURIComponent(mac)}/history`);
            const data = await resp.json();
            const history = data.history || [];

            if (history.length === 0) {
                this.trendEl.textContent = 'No data';
                this.distEl.textContent = '-- m';
                return;
            }

            const latestPoint = history[history.length - 1];
            this.trendEl.textContent = latestPoint.trend;
            this.distEl.textContent = `~${latestPoint.estimated_distance_m.toFixed(1)} meters`;

            this.renderChart(history);
        } catch (e) {
            console.error("Failed to load device history:", e);
        }
    }

    renderChart(history) {
        const labels = history.map(h => h.timestamp.substring(11, 19));
        const rssiData = history.map(h => h.rssi);
        const snrData = history.map(h => h.snr);
        const distanceData = history.map(h => h.estimated_distance_m);

        if (this.chart) {
            this.chart.destroy();
        }

        const ctx = this.canvasEl.getContext('2d');
        this.chart = new Chart(ctx, {
            type: 'line',
            data: {
                labels: labels,
                datasets: [
                    {
                        label: 'RSSI (dBm)',
                        data: rssiData,
                        borderColor: '#00f2fe',
                        backgroundColor: 'rgba(0, 242, 254, 0.1)',
                        borderWidth: 2,
                        fill: true,
                        tension: 0.3,
                        yAxisID: 'yRssi'
                    },
                    {
                        label: 'Estimated SNR (dB)',
                        data: snrData,
                        borderColor: '#c084fc',
                        borderWidth: 2,
                        borderDash: [5, 5],
                        fill: false,
                        tension: 0.3,
                        yAxisID: 'yRssi'
                    },
                    {
                        label: 'Approx. Distance (Meters)',
                        data: distanceData,
                        borderColor: '#ff0844',
                        backgroundColor: 'rgba(255, 8, 68, 0.1)',
                        borderWidth: 2.5,
                        fill: false,
                        tension: 0.3,
                        yAxisID: 'yDistance'
                    }
                ]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                interaction: {
                    mode: 'index',
                    intersect: false
                },
                plugins: {
                    legend: {
                        labels: {
                            color: '#94a3b8',
                            font: { family: 'Inter', size: 12 }
                        }
                    },
                    tooltip: {
                        callbacks: {
                            afterBody: (context) => {
                                const index = context[0].dataIndex;
                                return `Movement Trend: ${history[index].trend}`;
                            }
                        }
                    }
                },
                scales: {
                    x: {
                        grid: { color: 'rgba(255, 255, 255, 0.05)' },
                        ticks: { color: '#94a3b8', font: { family: 'JetBrains Mono', size: 10 } }
                    },
                    yRssi: {
                        type: 'linear',
                        display: true,
                        position: 'left',
                        title: { display: true, text: 'Signal (dBm / dB)', color: '#00f2fe' },
                        grid: { color: 'rgba(255, 255, 255, 0.05)' },
                        ticks: { color: '#94a3b8' }
                    },
                    yDistance: {
                        type: 'linear',
                        display: true,
                        position: 'right',
                        title: { display: true, text: 'Approx. Distance (Meters)', color: '#ff0844' },
                        grid: { drawOnChartArea: false },
                        ticks: { color: '#ff0844' }
                    }
                }
            }
        });
    }

    hideModal() {
        this.modalEl.classList.add('hidden');
    }
}

window.DeviceRssiChart = DeviceRssiChart;
