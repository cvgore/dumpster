'use strict';

(function initApp() {
    const ICON_UPLOAD = '📤';
    const ICON_DOWNLOAD = '📥';

    const dropIcon = document.querySelector('[data-upload-icon]');
    const fileInput = document.querySelector('[data-file]');

    fileInput.addEventListener('input', async function handleUpload() {
        fileInput.disabled = true;

        const formData = new FormData();

        formData.append('file', fileInput.files[0]);

        const req = new XMLHttpRequest();

        req.addEventListener('loadstart', () => {
            dropIcon.setAttribute('data-uploading', '');
            dropIcon.textContent = ICON_UPLOAD;
        })

        req.open('POST', '/upload', true);
        req.send(formData);
    });
})();