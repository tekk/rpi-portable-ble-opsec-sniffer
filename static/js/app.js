document.addEventListener('DOMContentLoaded', () => {
    // Initialize Submodules
    window.radar = new window.BleRadar('radarCanvas');
    window.videoCorrelator = new window.VideoCorrelator();
    window.queryStudio = new window.QueryStudio();
    window.deviceChart = new window.DeviceRssiChart();
    window.wizard = new window.SetupWizard();
    window.storageManager = new window.StorageBackupManager();

    // Camera Toggle Button Setup
    setupCameraToggle();

    // Tab Navigation Handler
    const navItems = document.querySelectorAll('.nav-item');
    const tabPanes = document.querySelectorAll('.tab-pane');
    const pageTitle = document.getElementById('current-tab-title');

    const tabTitles = {
        'radar': 'Live BLE Radar & Sniffer',
        'videos': 'Motion Videos & Correlated BLE Devices',
        'devices': 'Device Inventory & Proximity Fingerprints',
        'queries': 'Query Studio (20 Predefined Queries)',
        'settings': 'Settings & System Configuration'
    };

    navItems.forEach(item => {
        item.addEventListener('click', () => {
            const tab = item.getAttribute('data-tab');
            navItems.forEach(i => i.classList.remove('active'));
            tabPanes.forEach(p => {
                p.classList.add('hidden');
                p.classList.remove('active');
            });

            item.classList.add('active');
            const targetPane = document.getElementById(`tab-${tab}`);
            if (targetPane) {
                targetPane.classList.remove('hidden');
                targetPane.classList.add('active');
            }
            pageTitle.textContent = tabTitles[tab] || 'BLE OPSEC Sniffer';

            if (tab === 'devices') loadDevices();
            if (tab === 'videos') window.videoCorrelator.loadVideos();
        });
    });

    // Device Search Input Listener
    const searchInput = document.getElementById('device-search-input');
    if (searchInput) {
        searchInput.addEventListener('input', (e) => {
            filterDevicesTable(e.target.value.toLowerCase());
        });
    }

    // OUI Update Button
    const btnUpdateOui = document.getElementById('btn-update-oui');
    const btnForceOui = document.getElementById('btn-force-oui');

    const handleOuiUpdate = async () => {
        alert("Downloading latest IEEE OUI vendor database...");
        try {
            const resp = await fetch('/api/oui/update', { method: 'POST' });
            const data = await resp.json();
            if (data.success) {
                alert(`Successfully updated OUI database with ${data.updated_vendors} vendor entries!`);
            } else {
                alert(`Update failed: ${data.error}`);
            }
        } catch (e) {
            alert(`OUI update error: ${e.message}`);
        }
    };

    if (btnUpdateOui) btnUpdateOui.addEventListener('click', handleOuiUpdate);
    if (btnForceOui) btnForceOui.addEventListener('click', handleOuiUpdate);

    // Setup WebSockets Live BLE Stream
    setupWebSocket();

    // Load initial devices list
    loadDevices();
});

// Clean Monochromatic Vector SVG Icon Generators (No Emoticons)
function getCategoryIcon(type) {
    if (!type) return getSvgIcon('generic');
    const t = type.toLowerCase();
    if (t.includes('iphone') || t.includes('ipad')) return getSvgIcon('iphone');
    if (t.includes('android')) return getSvgIcon('android');
    if (t.includes('watch') || t.includes('wearable')) return getSvgIcon('smartwatch');
    if (t.includes('tv')) return getSvgIcon('tv');
    if (t.includes('mac')) return getSvgIcon('mac');
    if (t.includes('windows') || t.includes('pc')) return getSvgIcon('windowspc');
    if (t.includes('esp32')) return getSvgIcon('esp32');
    if (t.includes('nrf')) return getSvgIcon('nrf');
    if (t.includes('airtag') || t.includes('tracker')) return getSvgIcon('tracker');
    if (t.includes('beacon')) return getSvgIcon('beacon');
    if (t.includes('smart home') || t.includes('iot')) return getSvgIcon('smarthome');
    if (t.includes('audio') || t.includes('headphone')) return getSvgIcon('audio');
    if (t.includes('hid') || t.includes('keyboard')) return getSvgIcon('hid');
    if (t.includes('medical')) return getSvgIcon('medical');
    if (t.includes('automotive') || t.includes('car')) return getSvgIcon('automotive');
    if (t.includes('game') || t.includes('console')) return getSvgIcon('gameconsole');
    if (t.includes('drone') || t.includes('uav')) return getSvgIcon('drone');
    if (t.includes('charger') || t.includes('ev')) return getSvgIcon('evcharger');
    if (t.includes('scale') || t.includes('fitness')) return getSvgIcon('smartscale');
    if (t.includes('camera') || t.includes('security')) return getSvgIcon('securitycam');
    return getSvgIcon('generic');
}

function getSvgIcon(name) {
    const svgs = {
        'iphone': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4 text-cyan-400"><rect width="14" height="20" x="5" y="2" rx="2"/><line x1="12" x2="12.01" y1="18" y2="18"/></svg>`,
        'android': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4 text-emerald-400"><rect width="14" height="20" x="5" y="2" rx="2"/><path d="M12 18h.01"/></svg>`,
        'smartwatch': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4 text-purple-400"><rect width="12" height="12" x="6" y="6" rx="2"/><path d="M12 2v4"/><path d="M12 18v4"/></svg>`,
        'tv': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4 text-rose-400"><rect width="20" height="14" x="2" y="3" rx="2"/><path d="M8 21h8"/><path d="M12 17v4"/></svg>`,
        'mac': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4 text-slate-200"><rect width="18" height="12" x="3" y="4" rx="2"/><path d="M2 20h20"/><path d="M12 16v4"/></svg>`,
        'windowspc': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4 text-sky-400"><path d="M4 4h7v7H4z"/><path d="M13 4h7v7h-7z"/><path d="M4 13h7v7H4z"/><path d="M13 13h7v7h-7z"/></svg>`,
        'esp32': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4 text-amber-400"><rect width="14" height="14" x="5" y="5" rx="2"/><path d="M13 2v3"/><path d="M11 2v3"/><path d="M13 19v3"/><path d="M11 19v3"/><path d="M2 13h3"/><path d="M2 11h3"/><path d="M19 13h3"/><path d="M19 11h3"/></svg>`,
        'nrf': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4 text-cyan-400"><path d="M5 12.55a11 11 0 0 1 14.08 0"/><path d="M1.42 9a16 16 0 0 1 21.16 0"/><path d="M8.53 16.11a6 6 0 0 1 6.95 0"/><line x1="12" y1="20" x2="12.01" y2="20"/></svg>`,
        'tracker': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4 text-rose-500"><path d="M12 2a8 8 0 0 0-8 8c0 5.25 8 12 8 12s8-6.75 8-12a8 8 0 0 0-8-8z"/><circle cx="12" cy="10" r="3"/></svg>`,
        'beacon': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4 text-orange-400"><path d="M4.9 19.1C1 15.2 1 8.8 4.9 4.9"/><path d="M7.8 16.2c-2.3-2.3-2.3-6.1 0-8.5"/><circle cx="12" cy="12" r="2"/><path d="M16.2 7.8c2.3 2.3 2.3 6.1 0 8.5"/><path d="M19.1 4.9c3.9 3.9 3.9 10.3 0 14.2"/></svg>`,
        'smarthome': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4 text-emerald-400"><path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><polyline points="9 22 9 12 15 12 15 22"/></svg>`,
        'audio': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4 text-purple-400"><path d="M3 14h3a2 2 0 0 1 2 2v3a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-7a9 9 0 0 1 18 0v7a2 2 0 0 1-2 2h-1a2 2 0 0 1-2-2v-3a2 2 0 0 1 2-2h3"/></svg>`,
        'hid': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4 text-sky-400"><rect width="20" height="12" x="2" y="6" rx="2"/><path d="M6 10h.01"/><path d="M10 10h.01"/><path d="M14 10h.01"/><path d="M18 10h.01"/><path d="M8 14h8"/></svg>`,
        'medical': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4 text-red-500"><path d="M11 2a2 2 0 0 0-2 2v5H4a2 2 0 0 0-2 2v2a2 2 0 0 0 2 2h5v5a2 2 0 0 0 2 2h2a2 2 0 0 0 2-2v-5h5a2 2 0 0 0 2-2v-2a2 2 0 0 0-2-2h-5V4a2 2 0 0 0-2-2h-2z"/></svg>`,
        'automotive': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4 text-cyan-400"><path d="M19 17h2c.6 0 1-.4 1-1v-3c0-.9-.7-1.7-1.5-1.9C18.7 10.6 16 10 16 10s-1.3-1.4-2.2-2.3c-.5-.4-1.1-.7-1.8-.7H5c-.6 0-1.1.4-1.4.9l-1.4 2.9A3.7 3.7 0 0 0 2 12v4c0 .6.4 1 1 1h2"/><circle cx="7" cy="17" r="2"/><circle cx="17" cy="17" r="2"/></svg>`,
        'gameconsole': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4 text-indigo-400"><line x1="6" y1="12" x2="10" y2="12"/><line x1="8" y1="10" x2="8" y2="14"/><circle cx="15" cy="13" r="1"/><circle cx="18" cy="11" r="1"/><rect width="20" height="12" x="2" y="6" rx="6"/></svg>`,
        'drone': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4 text-emerald-400"><path d="M12 12m-3 0a3 3 0 1 0 6 0a3 3 0 1 0 -6 0"/><path d="M4.5 4.5l5 5"/><path d="M19.5 4.5l-5 5"/><path d="M4.5 19.5l5-5"/><path d="M19.5 19.5l-5-5"/><circle cx="4.5" cy="4.5" r="2.5"/><circle cx="19.5" cy="4.5" r="2.5"/><circle cx="4.5" cy="19.5" r="2.5"/><circle cx="19.5" cy="19.5" r="2.5"/></svg>`,
        'evcharger': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4 text-amber-400"><path d="M18 2v4"/><path d="M6 2v4"/><path d="M4 6h16v3a4 4 0 0 1-4 4H8a4 4 0 0 1-4-4V6z"/><path d="M12 13v9"/></svg>`,
        'smartscale': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4 text-teal-400"><rect width="18" height="18" x="3" y="3" rx="3"/><circle cx="12" cy="9" r="2.5"/><path d="M12 11.5v3.5"/></svg>`,
        'securitycam': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4 text-cyan-400"><path d="m16 13 5.223 3.482a.5.5 0 0 0 .777-.416V7.934a.5.5 0 0 0-.777-.416L16 11"/><rect width="14" height="12" x="2" y="6" rx="2"/></svg>`,
        'generic': `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4 text-cyan-400"><path d="m7 7 10 10-5 5V2l5 5L7 17"/></svg>`
    };
    return svgs[name] || svgs['generic'];
}

function getBrandBadge(vendor) {
    if (!vendor) return '<span class="px-2.5 py-0.5 rounded-full text-[10px] bg-slate-800 text-slate-400 font-semibold border border-slate-700">Unknown</span>';
    const v = vendor.toLowerCase();
    if (v.includes('apple')) return '<span class="px-2.5 py-0.5 rounded-full text-[10px] bg-slate-100/10 text-slate-200 border border-slate-400/30 font-bold uppercase tracking-wider">Apple</span>';
    if (v.includes('samsung')) return '<span class="px-2.5 py-0.5 rounded-full text-[10px] bg-blue-500/10 text-blue-400 border border-blue-500/30 font-bold uppercase tracking-wider">Samsung</span>';
    if (v.includes('google')) return '<span class="px-2.5 py-0.5 rounded-full text-[10px] bg-emerald-500/10 text-emerald-400 border border-emerald-500/30 font-bold uppercase tracking-wider">Google</span>';
    if (v.includes('microsoft')) return '<span class="px-2.5 py-0.5 rounded-full text-[10px] bg-sky-500/10 text-sky-400 border border-sky-500/30 font-bold uppercase tracking-wider">Microsoft</span>';
    if (v.includes('espressif')) return '<span class="px-2.5 py-0.5 rounded-full text-[10px] bg-amber-500/10 text-amber-400 border border-amber-500/30 font-bold uppercase tracking-wider">Espressif</span>';
    if (v.includes('nordic')) return '<span class="px-2.5 py-0.5 rounded-full text-[10px] bg-cyan-500/10 text-cyan-400 border border-cyan-500/30 font-bold uppercase tracking-wider">Nordic</span>';
    if (v.includes('lg')) return '<span class="px-2.5 py-0.5 rounded-full text-[10px] bg-rose-500/10 text-rose-400 border border-rose-500/30 font-bold uppercase tracking-wider">LG</span>';
    if (v.includes('sony')) return '<span class="px-2.5 py-0.5 rounded-full text-[10px] bg-purple-500/10 text-purple-400 border border-purple-500/30 font-bold uppercase tracking-wider">Sony</span>';
    if (v.includes('garmin') || v.includes('fitbit')) return '<span class="px-2.5 py-0.5 rounded-full text-[10px] bg-teal-500/10 text-teal-400 border border-teal-500/30 font-bold uppercase tracking-wider">Fitness</span>';
    if (v.includes('raspberry')) return '<span class="px-2.5 py-0.5 rounded-full text-[10px] bg-pink-500/10 text-pink-400 border border-pink-500/30 font-bold uppercase tracking-wider">Raspberry Pi</span>';
    return `<span class="px-2.5 py-0.5 rounded-full text-[10px] bg-slate-800 text-slate-300 border border-slate-700 font-medium">${vendor}</span>`;
}

window.getCategoryIcon = getCategoryIcon;
window.getBrandBadge = getBrandBadge;

let totalPacketsCount = 0;
const knownMacs = new Set();
let trackerCount = 0;
let esp32Count = 0;

function setupWebSocket() {
    const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${location.host}/ws`;
    const ws = new WebSocket(wsUrl);

    const tbodyPackets = document.getElementById('tbody-packets');

    ws.onmessage = (event) => {
        try {
            const packet = JSON.parse(event.data);
            totalPacketsCount++;
            knownMacs.add(packet.mac_address);

            if (packet.inferred_device_type.includes('Tracker') || packet.inferred_device_type.includes('AirTag')) {
                trackerCount++;
            }
            if (packet.inferred_device_type.includes('ESP32') || packet.inferred_device_type.includes('nRF')) {
                esp32Count++;
            }

            // Update Metrics Counters
            document.getElementById('metric-total-pkts').textContent = totalPacketsCount;
            document.getElementById('metric-total-devs').textContent = knownMacs.size;
            document.getElementById('metric-trackers').textContent = trackerCount;
            document.getElementById('metric-esp32').textContent = esp32Count;

            // Plot blip on Radar Canvas
            if (window.radar) {
                window.radar.updateDevice(packet.mac_address, packet.rssi, packet.inferred_device_type);
            }

            const catIcon = getCategoryIcon(packet.inferred_device_type);
            const brandBadge = getBrandBadge(packet.vendor_name);

            // Append row to Live Packet Feed table (keep max 20 rows)
            const rowHtml = `
                <tr class="hover:bg-slate-800/40 transition-colors cursor-pointer" onclick="window.deviceChart.showDeviceHistory('${packet.mac_address}', '${packet.local_name || packet.vendor_name}')">
                    <td class="p-3.5 text-slate-300">${packet.timestamp.substring(11, 19)}</td>
                    <td class="p-3.5"><span class="px-2 py-0.5 rounded bg-slate-800 text-slate-300 font-bold">${packet.adapter_name}</span></td>
                    <td class="p-3.5 font-bold text-cyan-400 hover:underline">${packet.mac_address}</td>
                    <td class="p-3.5"><span class="px-2 py-0.5 rounded bg-indigo-500/10 text-indigo-400 border border-indigo-500/30">${packet.address_type}</span></td>
                    <td class="p-3.5"><span class="px-2.5 py-1 rounded-lg bg-cyan-500/10 text-cyan-400 border border-cyan-500/30 font-bold flex items-center gap-2 w-fit">${catIcon} <span>${packet.inferred_device_type}</span></span></td>
                    <td class="p-3.5">${brandBadge}</td>
                    <td class="p-3.5 font-bold text-cyan-400">${packet.rssi} dBm</td>
                    <td class="p-3.5 text-slate-400 font-mono text-[11px]">${packet.local_name || 'N/A'}</td>
                </tr>
            `;

            if (tbodyPackets.querySelector('.text-slate-500')) {
                tbodyPackets.innerHTML = '';
            }

            tbodyPackets.insertAdjacentHTML('afterbegin', rowHtml);

            if (tbodyPackets.children.length > 20) {
                tbodyPackets.removeChild(tbodyPackets.lastChild);
            }
        } catch (e) {
            console.error("WS Packet parse error:", e);
        }
    };

    ws.onclose = () => {
        setTimeout(setupWebSocket, 3000);
    };
}

let cachedDevices = [];

async function loadDevices() {
    const tbody = document.getElementById('tbody-devices');
    try {
        const resp = await fetch('/api/devices');
        const data = await resp.json();
        cachedDevices = data.devices || [];

        renderDevicesTable(cachedDevices);
    } catch (e) {
        console.error("Failed to load device list:", e);
    }
}

function renderDevicesTable(devices) {
    const tbody = document.getElementById('tbody-devices');
    if (devices.length === 0) {
        tbody.innerHTML = `<tr><td colspan="9" class="p-6 text-center text-slate-500">No devices logged yet.</td></tr>`;
        return;
    }

    tbody.innerHTML = devices.map(d => {
        const catIcon = getCategoryIcon(d.inferred_type);
        const brandBadge = getBrandBadge(d.vendor_name);

        return `
            <tr class="hover:bg-slate-800/40 transition-colors">
                <td class="p-3.5 font-bold text-slate-100">${d.mac_address}</td>
                <td class="p-3.5"><span class="px-2 py-0.5 rounded bg-slate-800 text-slate-300">${d.address_type}</span></td>
                <td class="p-3.5"><span class="px-2.5 py-1 rounded-lg bg-cyan-500/10 text-cyan-400 border border-cyan-500/30 font-bold flex items-center gap-2 w-fit">${catIcon} <span>${d.inferred_type}</span></span></td>
                <td class="p-3.5">${brandBadge}</td>
                <td class="p-3.5 text-slate-400">${d.last_adv_name || 'N/A'}</td>
                <td class="p-3.5 font-bold text-cyan-400">${d.max_rssi} dBm</td>
                <td class="p-3.5 text-slate-300 font-bold">${d.total_packets}</td>
                <td class="p-3.5 text-slate-400">${d.last_seen.substring(11, 19)}</td>
                <td class="p-3.5">
                    <button class="px-3 py-1.5 rounded-lg bg-cyan-500/10 hover:bg-cyan-500/20 text-cyan-400 border border-cyan-500/30 font-bold transition-all text-xs flex items-center gap-1.5" onclick="window.deviceChart.showDeviceHistory('${d.mac_address}', '${d.last_adv_name || d.vendor_name}')">
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-3.5 h-3.5"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>
                        <span>Graph</span>
                    </button>
                </td>
            </tr>
        `;
    }).join('');
}

function filterDevicesTable(query) {
    if (!query) {
        renderDevicesTable(cachedDevices);
        return;
    }
    const filtered = cachedDevices.filter(d => 
        d.mac_address.toLowerCase().includes(query) ||
        d.vendor_name.toLowerCase().includes(query) ||
        d.inferred_type.toLowerCase().includes(query)
    );
    renderDevicesTable(filtered);
}

let isCameraActive = true;

async function setupCameraToggle() {
    const btnToggle = document.getElementById('btn-toggle-camera');
    const indicator = document.getElementById('camera-status-indicator');
    const text = document.getElementById('camera-status-text');

    if (!btnToggle) return;

    const updateCameraUI = (enabled) => {
        isCameraActive = enabled;
        if (enabled) {
            indicator.className = "w-2.5 h-2.5 rounded-full bg-emerald-400 animate-pulse";
            text.textContent = "📷 Camera Active";
            btnToggle.className = "px-3.5 py-2 text-xs font-bold rounded-xl bg-slate-800 text-slate-200 border border-slate-700 hover:bg-slate-700 transition-all flex items-center gap-2";
        } else {
            indicator.className = "w-2.5 h-2.5 rounded-full bg-rose-500";
            text.textContent = "📷 Camera Disabled";
            btnToggle.className = "px-3.5 py-2 text-xs font-bold rounded-xl bg-rose-500/10 text-rose-400 border border-rose-500/30 hover:bg-rose-500/20 transition-all flex items-center gap-2";
        }
    };

    try {
        const resp = await fetch('/api/system/camera/status');
        const data = await resp.json();
        updateCameraUI(data.camera_enabled);
    } catch (e) {
        console.error("Failed to fetch camera status:", e);
    }

    btnToggle.addEventListener('click', async () => {
        const newState = !isCameraActive;
        try {
            const resp = await fetch('/api/system/camera/toggle', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ enabled: newState })
            });
            const data = await resp.json();
            updateCameraUI(data.camera_enabled);
        } catch (e) {
            alert(`Failed to toggle camera: ${e.message}`);
        }
    });
}

