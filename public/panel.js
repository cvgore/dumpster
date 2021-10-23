(function initApp() {
    'use strict';

    const contentEl = document.querySelector('[data-content]');
    const filesEl = document.querySelector('[data-files-list]');
    const toggleScopeUserBtn = document.querySelector('[data-scope-user]');
    const toggleScopeCommonBtn = document.querySelector('[data-scope-common]');
    const logoutBtn = document.querySelector('[data-logout]');

    const SCOPE_USER = 'user';
    const SCOPE_COMMON = 'common';

    const STR_NEXT_FILES = 'next files >';
    const STR_PREV_FILES = '< prev files';

    const CURRENT_PARAMS = new URL(window.location.href).searchParams;
    const CURRENT_SCOPE = CURRENT_PARAMS.get('scope') || SCOPE_COMMON;
    const CURRENT_CURSOR = parseInt(CURRENT_PARAMS.get('cursor') || 0);

    async function downloadFile(file, ev) {
        ev.preventDefault();

        const token = sessionStorage.getItem('token');
        const formData = new FormData();

        formData.set('filename', file.name);
        formData.set('scope', CURRENT_SCOPE);

        const req = new XMLHttpRequest();

        const target = ev.target;
        const progressEl = document.createElement('b');
        progressEl.className = 'download-progress';

        target.append(progressEl);

        req.addEventListener('loadstart', function onDownloadStart() {
            progressEl.textContent = '...';
        });

        req.addEventListener('progress', function onDownloadProgress(ev) {
            const percent = ev.loaded / ev.total;

            progressEl.textContent = `${(percent * 100).toFixed(0)}%`;
        });

        req.addEventListener('readystatechange', function onDownloadStateChange() {
            if (req.readyState === XMLHttpRequest.DONE) {
                if (req.status === 200) {
                    const url = URL.createObjectURL(req.response);
                    const tmpAnchorEl = document.createElement('a');
                    tmpAnchorEl.href = url;
                    tmpAnchorEl.download = file.name;

                    document.body.appendChild(tmpAnchorEl);
                    tmpAnchorEl.click();
                    tmpAnchorEl.remove();
                    progressEl.remove();
                } else if (req.status === 401) {
                    sessionStorage.removeItem('token');
                    window.location.href = 'panel.html';
                } else {
                    console.error('failed to download %s, status %d', file, req.status, req.response);
                }
            }
        });

        req.addEventListener('error', function onDownloadError(ev) {
            progressEl.remove();

            console.error('failed to download %s', file, ev);
        });

        req.addEventListener('abort', function onDownloadAbort() {
            progressEl.remove();
        });

        req.open('POST', '/ajax/files/download', true);
        req.setRequestHeader('Authorization', `Bearer ${token}`);
        req.responseType = 'blob';
        req.send(formData);
    }

    function createFileListing({files, prevCursor, nextCursor}) {
        filesEl.childNodes.forEach((n) => n.remove());

        const listGrpEl = document.createElement('ul');

        for (const file of files) {
            const listItemEl = document.createElement('li');
            const linkEl = document.createElement('a');

            linkEl.href = 'javascript: void 0';
            linkEl.textContent = file.name;

            linkEl.addEventListener('click', downloadFile.bind(null, file));

            listItemEl.append(linkEl);
            listGrpEl.append(listItemEl);
        }

        filesEl.append(listGrpEl);

        if (nextCursor !== null) {
            const url = new URL(window.location.href);
            url.searchParams.set('scope', CURRENT_SCOPE);
            url.searchParams.set('cursor', nextCursor);

            const nextFilesBtn = document.createElement('a');
            nextFilesBtn.className = 'button';
            nextFilesBtn.textContent = STR_NEXT_FILES;
            nextFilesBtn.href = url;

            filesEl.append(nextFilesBtn);
        }

        if (prevCursor !== null) {
            const url = new URL(window.location.href);
            url.searchParams.set('scope', CURRENT_SCOPE);
            url.searchParams.set('cursor', prevCursor);

            const prevFilesBtn = document.createElement('a');
            prevFilesBtn.className = 'button';
            prevFilesBtn.textContent = STR_PREV_FILES;
            prevFilesBtn.href = url;

            filesEl.prepend(prevFilesBtn);
        }
    }

    async function logout() {
        const token = sessionStorage.getItem('token');

        await fetch('/ajax/logout', {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${token}`
            },
        });

        sessionStorage.removeItem('token');
        window.location.href = 'login.html';
    }

    function switchScope(scope) {
        const url = new URL(window.location.href);
        url.searchParams.set('scope', scope);
        url.searchParams.set('cursor', 0);

        window.location.href = url;
    }

    async function loadFiles() {
        const token = sessionStorage.getItem('token');

        const url = new URL(`${window.location.origin}/ajax/files`);
        url.searchParams.set('scope', CURRENT_SCOPE);
        url.searchParams.set('cursor', CURRENT_CURSOR);

        const resp = await fetch(url.toString(), {
            method: 'GET',
            headers: {
                'Authorization': `Bearer ${token}`
            },

        });

        if (!resp.ok) {
            sessionStorage.removeItem('token');
            window.location.href = 'login.html';
        }

        const data = await resp.json();

        createFileListing(data);
    }

    toggleScopeUserBtn.addEventListener('click', switchScope.bind(null, SCOPE_USER));
    toggleScopeCommonBtn.addEventListener('click', switchScope.bind(null, SCOPE_COMMON));
    logoutBtn.addEventListener('click', logout);

    if (CURRENT_SCOPE === SCOPE_COMMON) {
        toggleScopeCommonBtn.className += ' button-outline';
    } else if (CURRENT_SCOPE === SCOPE_USER) {
        toggleScopeUserBtn.className += ' button-outline';
    }

    loadFiles();
})();