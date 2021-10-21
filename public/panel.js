(function initApp() {
    'use strict';

    const contentEl = document.querySelector('[data-content]');
    const filesEl = document.querySelector('[data-files-list]');

    const SCOPE_USER = 'user';
    const SCOPE_COMMON = 'common';

    const STR_NEXT_FILES = 'next files';

    const CURRENT_PARAMS = new URLSearchParams(window.location.search);
    const CURRENT_SCOPE = CURRENT_PARAMS.get('scope') || SCOPE_USER;
    const CURRENT_CURSOR = CURRENT_PARAMS.get('cursor') || 0;

    function downloadFile(file) {
        const formEl = document.createElement('form');
        formEl.action = '/ajax/files/download';
        formEl.method = 'POST';
        formEl.hidden = true;

        const data = new FormData(formEl);

        data.set('filename', file.name);
        data.set('scope', CURRENT_SCOPE);

        document.body.appendChild(formEl);
        formEl.submit();
        formEl.remove();
    }

    function createFileListing(files, cursor) {
        filesEl.childNodes.forEach((n) => n.remove());

        const listGrpEl = document.createElement('ul');

        for (const file of files) {
            const listItemEl = document.createElement('li');
            const linkEl = document.createElement('button');

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
            // sessionStorage.removeItem('token');
            // window.location.href = 'login.html';
        }

        const data = await resp.json();

        createFileListing(data.files);
    }

    loadFiles()
})();