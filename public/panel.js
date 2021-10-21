(function initApp() {
    'use strict';

    const contentEl = document.querySelector('[data-content]');
    const filesEl = document.querySelector('[data-files-list]');

    const SCOPE_USER = 'user';
    const SCOPE_COMMON = 'common';

    const STR_NEXT_FILES = 'next files';

    const CURRENT_PARAMS = new URLSearchParams(window.location.search);
    const CURRENT_SCOPE = CURRENT_PARAMS.get('scope') || SCOPE_COMMON;
    const CURRENT_CURSOR = CURRENT_PARAMS.get('cursor') || 0;

    async function downloadFile(file, ev) {
        ev.preventDefault();

        const token = sessionStorage.getItem('token');
        const data = new FormData();

        data.set('filename', file.name);
        data.set('scope', CURRENT_SCOPE);

        const resp = await fetch('/ajax/files/download', {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${token}`
            },
            body: data
        });

        if (!resp.ok) {
            return;
        }

        const blob = await resp.blob();

        const url = window.URL.createObjectURL(blob);
        const tmpAnchorEl = document.createElement('a');
        tmpAnchorEl.href = url;
        tmpAnchorEl.download = file.name;

        document.body.appendChild(tmpAnchorEl);
        tmpAnchorEl.click();
        tmpAnchorEl.remove();
    }

    function createFileListing(files, cursor) {
        filesEl.childNodes.forEach((n) => n.remove());

        const listGrpEl = document.createElement('ul');

        for (const file of files) {
            const listItemEl = document.createElement('li');
            const linkEl = document.createElement('a');

            linkEl.href = "#";
            linkEl.textContent = file.name;

            linkEl.addEventListener('click', downloadFile.bind(null, file));

            listItemEl.append(linkEl);
            listGrpEl.append(listItemEl);
        }

        filesEl.append(listGrpEl);

        if (cursor) {
            const url = new URL(window.location.href);
            url.searchParams.set('scope', CURRENT_SCOPE);
            url.searchParams.set('cursor', cursor);

            const moreFilesBtn = document.createElement('a');
            moreFilesBtn.className = 'button';
            moreFilesBtn.textContent = STR_NEXT_FILES;
            moreFilesBtn.href = window.location.href;

            filesEl.append(moreFilesBtn);
        }
    }

    async function loadFiles() {
        const token = sessionStorage.getItem('token');

        const resp = await fetch('/ajax/files', {
            method: 'GET',
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });

        if (!resp.ok) {
            sessionStorage.removeItem('token');
            window.location.href = 'login.html';
        }

        const data = await resp.json();

        createFileListing(data.files);
    }

    loadFiles();
})();