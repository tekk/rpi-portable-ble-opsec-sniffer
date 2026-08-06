class QueryStudio {
    constructor() {
        this.listEl = document.getElementById('query-selector-list');
        this.editorEl = document.getElementById('sql-editor');
        this.btnRun = document.getElementById('btn-run-query');
        this.theadEl = document.getElementById('thead-query');
        this.tbodyEl = document.getElementById('tbody-query');
        this.countEl = document.getElementById('query-result-count');

        this.queries = [];
        this.init();
    }

    async init() {
        this.btnRun.addEventListener('click', () => this.runCustomQuery());
        await this.loadQueries();
    }

    async loadQueries() {
        try {
            const resp = await fetch('/api/queries');
            const data = await resp.json();
            this.queries = data.queries || [];
            this.renderQueryList();
            if (this.queries.length > 0) {
                this.selectQuery(this.queries[0]);
            }
        } catch (e) {
            console.error("Failed to load predefined queries:", e);
        }
    }

    renderQueryList() {
        this.listEl.innerHTML = this.queries.map(q => `
            <div class="query-item p-3 bg-slate-950/60 border border-slate-800/80 rounded-xl cursor-pointer hover:border-cyan-500/50 hover:bg-slate-900/60 transition-all space-y-1" id="qitem-${q.id}" onclick="window.queryStudio.selectQueryById(${q.id})">
                <h4 class="font-bold text-xs text-slate-100">${q.title}</h4>
                <p class="text-[11px] text-slate-400 leading-tight">${q.description}</p>
            </div>
        `).join('');
    }

    selectQueryById(id) {
        const query = this.queries.find(q => q.id === id);
        if (query) {
            this.selectQuery(query);
        }
    }

    selectQuery(query) {
        document.querySelectorAll('.query-item').forEach(el => el.classList.remove('selected'));
        const selectedEl = document.getElementById(`qitem-${query.id}`);
        if (selectedEl) selectedEl.classList.add('selected');

        document.getElementById('query-title').textContent = query.title;
        this.editorEl.value = query.sql.trim();
        this.runCustomQuery();
    }

    async runCustomQuery() {
        const sql = this.editorEl.value.trim();
        if (!sql) return;

        this.tbodyEl.innerHTML = `<tr><td class="p-6 text-center text-slate-500">Executing query...</td></tr>`;

        try {
            const resp = await fetch('/api/queries/run', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ sql })
            });

            const data = await resp.json();

            if (data.error) {
                this.countEl.textContent = 'Error';
                this.theadEl.innerHTML = '';
                this.tbodyEl.innerHTML = `<tr><td class="p-6 text-rose-400 font-mono">${data.error}</td></tr>`;
                return;
            }

            const results = data.results || [];
            this.countEl.textContent = `${results.length} rows returned`;
            this.renderResultsTable(results);
        } catch (e) {
            this.tbodyEl.innerHTML = `<tr><td class="p-6 text-rose-400 font-mono">Query execution failed: ${e.message}</td></tr>`;
        }
    }

    renderResultsTable(rows) {
        if (rows.length === 0) {
            this.theadEl.innerHTML = '';
            this.tbodyEl.innerHTML = `<tr><td class="p-6 text-center text-slate-500">Query executed successfully. 0 rows returned.</td></tr>`;
            return;
        }

        const columns = Object.keys(rows[0]);
        this.theadEl.innerHTML = `<tr>${columns.map(c => `<th class="p-3.5">${c}</th>`).join('')}</tr>`;

        this.tbodyEl.innerHTML = rows.map(r => `
            <tr class="hover:bg-slate-800/40 transition-colors">
                ${columns.map(c => {
                    const val = r[c];
                    if (val === null || val === undefined) {
                        return `<td class="p-3.5 text-slate-200"><span class="text-slate-600">NULL</span></td>`;
                    }
                    const lowerCol = c.toLowerCase();
                    if (lowerCol.includes('rssi')) {
                        const bargraph = window.createRssiBargraphHtml ? window.createRssiBargraphHtml(val) : '';
                        return `<td class="p-3.5 text-cyan-400 font-bold flex items-center">${bargraph} <span>${val} dBm</span></td>`;
                    }
                    if (lowerCol.includes('seen') || lowerCol.includes('time') || lowerCol.includes('timestamp') || lowerCol.includes('start') || lowerCol.includes('end')) {
                        const formattedTime = window.formatLocalTimestamp ? window.formatLocalTimestamp(val) : val;
                        return `<td class="p-3.5 text-slate-300 font-mono text-[11px]">${formattedTime}</td>`;
                    }
                    return `<td class="p-3.5 text-slate-200">${val}</td>`;
                }).join('')}
            </tr>
        `).join('');
    }
}

window.QueryStudio = QueryStudio;
