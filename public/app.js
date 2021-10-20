(function initApp() {
    'use strict';

    const ICON_UPLOAD = '📤';
    const ICON_DOWNLOAD = '📥';
    const STR_DROP_FILE = 'Drop file here or select one';
    const UPLOAD_SUCCESS = 0;
    const UPLOAD_ABORT = 1;
    const UPLOAD_TIMEOUT = 2;
    const UPLOAD_ERROR = 3;
    const STRMAP_FILE_STATE = {
        [UPLOAD_SUCCESS]: 'File :f uploaded successfully',
        [UPLOAD_ABORT]: 'File :f upload cancelled',
        [UPLOAD_TIMEOUT]: 'File :f upload timed out',
        [UPLOAD_ERROR]: 'File :f failed to upload, see logs'
    };
    const ICONMAP_FILE_STATE = {
        [UPLOAD_SUCCESS]: '✔',
        [UPLOAD_ABORT]: '🟥',
        [UPLOAD_TIMEOUT]: '⏰',
        [UPLOAD_ERROR]: '❌'
    };
    
    const dropIcon = document.querySelector('[data-upload-icon]');
    const dropCont = document.querySelector('[data-upload-drop]');
    const fileInput = document.querySelector('[data-file]');
    const uploadText = document.querySelector('[data-upload-text]');

    function setUploading() {
        fileInput.disabled = true;
        dropCont.setAttribute('data-uploading', '');
        dropIcon.textContent = ICON_UPLOAD;
    }

    function unlockUpload() {
        fileInput.disabled = false;
        dropCont.removeAttribute('data-uploading');
        dropIcon.textContent = ICON_DOWNLOAD;
        uploadText.textContent = STR_DROP_FILE;
    }

    function setReadyToUpload(state) {
        uploadText.textContent = STRMAP_FILE_STATE[state].replace(':f', fileInput.files[0].name);
        dropIcon.textContent = ICONMAP_FILE_STATE[state];
        document.body.style.setProperty('--upload-progress', '100%');

        setTimeout(() => {
            unlockUpload();
        }, 2000);
    }

    function setUploadProgress(percent) {
        const value = `${(percent * 100).toFixed(0)}%`;

        document.body.style.setProperty('--upload-progress', value);
        uploadText.textContent = value;
    }

    fileInput.addEventListener('input', function handleUpload() {
        const formData = new FormData();
        formData.append('file', fileInput.files[0]);

        const req = new XMLHttpRequest();

        req.upload.addEventListener('loadstart', function onUploadStart() {
            setUploading();
        });

        req.upload.addEventListener('progress', function onUploadProgress(ev) {
           setUploadProgress(ev.loaded / ev.total);
        });

        req.upload.addEventListener('load', function onUploadEnd() {
            setReadyToUpload(UPLOAD_SUCCESS);    
        });

        req.upload.addEventListener('error', function onUploadError(ev) {
            setReadyToUpload(UPLOAD_ERROR);
            console.error('failed to upload %s', fileInput.files[0].name, ev);
        });

        req.upload.addEventListener('abort', function onUploadError() {
            setReadyToUpload(UPLOAD_ABORT);
        });

        req.upload.addEventListener('timeout', function onUploadError() {
            setReadyToUpload(UPLOAD_TIMEOUT);
        });

        req.open('POST', '/upload', true);
        req.timeout = 3000;
        req.send(formData);
    });

    unlockUpload();
})();