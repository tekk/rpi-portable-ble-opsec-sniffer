class VideoCorrelator {
    constructor() {
        this.tbody = document.getElementById('tbody-videos');
        this.player = document.getElementById('motionVideoPlayer');
        this.corrList = document.getElementById('correlated-devices-list');
        this.timelineList = document.getElementById('video-live-timeline');
        
        this.currentVideoId = null;
        this.timelineData = [];
        this.videoStartTimeEpoch = 0;
        this.playbackSpeed = 1.0;

        this.init();
    }

    init() {
        this.cachedVideos = [];
        this.videoSorter = window.TableSorter ? new window.TableSorter('table-videos', () => this.renderVideoTable(this.cachedVideos)) : null;
        this.loadVideos();
        this.setupControls();

        if (this.player) {
            this.player.addEventListener('timeupdate', () => this.syncBleTimeline());
        }
    }

    setupControls() {
        const btnPlay = document.getElementById('vctrl-play');
        const btnPause = document.getElementById('vctrl-pause');
        const btnRewind = document.getElementById('vctrl-rewind');
        const btnForward = document.getElementById('vctrl-forward');
        const btnPrevFrame = document.getElementById('vctrl-prev-frame');
        const btnNextFrame = document.getElementById('vctrl-next-frame');
        const speedSelect = document.getElementById('vctrl-speed');

        if (btnPlay) btnPlay.addEventListener('click', () => this.player.play());
        if (btnPause) btnPause.addEventListener('click', () => this.player.pause());
        if (btnRewind) btnRewind.addEventListener('click', () => { this.player.currentTime = Math.max(0, this.player.currentTime - 5); });
        if (btnForward) btnForward.addEventListener('click', () => { this.player.currentTime = Math.min(this.player.duration, this.player.currentTime + 5); });
        
        // Frame by Frame (assuming ~25fps = 0.04s per frame)
        if (btnPrevFrame) btnPrevFrame.addEventListener('click', () => {
            this.player.pause();
            this.player.currentTime = Math.max(0, this.player.currentTime - 0.04);
        });
        if (btnNextFrame) btnNextFrame.addEventListener('click', () => {
            this.player.pause();
            this.player.currentTime = Math.min(this.player.duration, this.player.currentTime + 0.04);
        });

        // Speedup Controls (1x, 2x, 3x, 4x, 8x)
        if (speedSelect) {
            speedSelect.addEventListener('change', (e) => {
                const speed = parseFloat(e.target.value);
                this.player.playbackRate = speed;
                this.playbackSpeed = speed;
            });
        }
    }

    async loadVideos() {
        try {
            const resp = await fetch('/api/videos');
            const data = await resp.json();
            this.cachedVideos = data.videos || [];
            this.renderVideoTable(this.cachedVideos);
        } catch (e) {
            console.error("Failed to load motion videos:", e);
        }
    }

    renderVideoTable(videos) {
        if (!this.tbody) return;
        const list = videos || this.cachedVideos || [];
        const sorted = this.videoSorter ? this.videoSorter.sortData(list) : list;

        if (sorted.length === 0) {
            this.tbody.innerHTML = `<tr><td colspan="6" class="p-6 text-center text-slate-500">No recorded motion events yet.</td></tr>`;
            return;
        }

        this.tbody.innerHTML = sorted.map(v => `
            <tr class="hover:bg-slate-800/40 transition-colors">
                <td class="p-3.5 font-bold text-slate-100">${v.filename}</td>
                <td class="p-3.5 text-slate-400 font-mono text-[11px]">${window.formatLocalTimestamp ? window.formatLocalTimestamp(v.start_time) : v.start_time}</td>
                <td class="p-3.5 text-slate-300 font-bold">${v.duration_secs}s</td>
                <td class="p-3.5"><span class="px-2.5 py-1 rounded-full bg-cyan-500/10 text-cyan-400 font-bold border border-cyan-500/30">${v.trigger_score.toFixed(1)}</span></td>
                <td class="p-3.5"><span class="px-2.5 py-1 rounded-full bg-purple-500/10 text-purple-400 font-bold border border-purple-500/30">${v.correlated_device_count} Devices</span></td>
                <td class="p-3.5">
                    <button class="px-3 py-1.5 rounded-lg bg-cyan-500/10 hover:bg-cyan-500/20 text-cyan-400 font-bold border border-cyan-500/30 transition-all flex items-center gap-1.5 text-xs" onclick="window.videoCorrelator.playVideo('${v.filename}', '${v.start_time}', ${v.duration_secs}, ${v.correlated_device_count}, ${v.video_id})">
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-3.5 h-3.5"><polygon points="5 3 19 12 5 21 5 3"/></svg>
                        <span>Play Synchronized</span>
                    </button>
                </td>
            </tr>
        `).join('');
    }

    async playVideo(filename, startTime, duration, devCount, videoId) {
        this.currentVideoId = videoId;
        this.videoStartTimeEpoch = new Date(startTime).getTime() / 1000;

        document.getElementById('video-player-title').textContent = `Motion Video: ${filename}`;
        const formattedStart = window.formatLocalTimestamp ? window.formatLocalTimestamp(startTime) : startTime;
        document.getElementById('video-meta-time').textContent = `Start: ${formattedStart}`;
        document.getElementById('video-meta-duration').textContent = `Duration: ${duration}s`;
        document.getElementById('video-meta-devices').textContent = `Devices: ${devCount}`;

        this.player.src = `/videos/${filename}`;
        this.player.play();

        // Load second-by-second BLE timeline packet data
        try {
            const resp = await fetch(`/api/videos/${videoId}/timeline`);
            const data = await resp.json();
            this.timelineData = data.timeline || [];
            this.renderCorrelatedList(this.timelineData);
        } catch (e) {
            console.error("Failed to load video timeline data:", e);
        }
    }

    syncBleTimeline() {
        if (!this.player || !this.timelineData.length) return;

        const currentVideoSec = Math.floor(this.player.currentTime);
        const currentTargetEpoch = Math.floor(this.videoStartTimeEpoch + currentVideoSec);

        // Filter packets captured at this exact second (+/- 1s window)
        const activeNow = this.timelineData.filter(p => Math.abs(p.timestamp_epoch - currentTargetEpoch) <= 1);

        const currentSecBadge = document.getElementById('video-current-sec-badge');
        if (currentSecBadge) {
            currentSecBadge.textContent = `Time: +${currentVideoSec}s (${activeNow.length} Active BLE Packets)`;
        }

        // Highlight active devices in correlated list
        const items = this.corrList.querySelectorAll('.corr-device-card');
        items.forEach(card => {
            const mac = card.getAttribute('data-mac');
            const isMatch = activeNow.some(p => p.mac_address === mac);
            if (isMatch) {
                card.classList.add('border-cyan-400', 'bg-cyan-500/10', 'ring-2', 'ring-cyan-500/50');
                card.classList.remove('border-slate-800/80', 'bg-slate-950/60');
            } else {
                card.classList.remove('border-cyan-400', 'bg-cyan-500/10', 'ring-2', 'ring-cyan-500/50');
                card.classList.add('border-slate-800/80', 'bg-slate-950/60');
            }
        });
    }

    renderCorrelatedList(items) {
        if (items.length === 0) {
            this.corrList.innerHTML = `<div class="text-slate-500 text-center py-12 text-xs">No BLE packets detected during this motion timeframe.</div>`;
            return;
        }

        // Deduplicate devices by MAC
        const uniqueDevs = {};
        items.forEach(p => {
            if (!uniqueDevs[p.mac_address]) {
                uniqueDevs[p.mac_address] = p;
            }
        });

        this.corrList.innerHTML = Object.values(uniqueDevs).map(item => {
            const catIcon = window.getCategoryIcon ? window.getCategoryIcon(item.inferred_type) : '';
            const brandBadge = window.getBrandBadge ? window.getBrandBadge(item.vendor_name) : item.vendor_name;
            const rssiBargraph = window.createRssiBargraphHtml ? window.createRssiBargraphHtml(item.rssi) : '';

            return `
                <div class="corr-device-card p-3 bg-slate-950/60 border border-slate-800/80 rounded-xl space-y-1.5 transition-all duration-200" data-mac="${item.mac_address}">
                    <div class="flex justify-between items-center">
                        <span class="font-mono font-bold text-xs text-slate-100 flex items-center gap-2">
                            <span>${catIcon}</span> <span>${item.mac_address}</span>
                        </span>
                        <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-cyan-500/10 text-cyan-400 border border-cyan-500/30 flex items-center">${rssiBargraph} <span>${item.rssi} dBm</span></span>
                    </div>
                    <div class="text-[11px] text-slate-400 flex justify-between items-center">
                        <div>${brandBadge}</div>
                        <span class="text-cyan-400 font-bold flex items-center gap-1.5">${catIcon} <span>${item.inferred_type}</span></span>
                    </div>
                </div>
            `;
        }).join('');
    }
}

window.VideoCorrelator = VideoCorrelator;
