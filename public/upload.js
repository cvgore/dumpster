(function initApp() {
    'use strict';

    const ICON_UPLOAD = '📤';
    const ICON_DOWNLOAD = '📥';
    const STR_DROP_FILE = 'Drop file here or select one to upload';
    const UPLOAD_SUCCESS = 0;
    const UPLOAD_ABORT = 1;
    const UPLOAD_ERROR = 3;
    const STRMAP_FILE_STATE = {
        [UPLOAD_SUCCESS]: 'File :f uploaded successfully',
        [UPLOAD_ABORT]: 'File :f upload cancelled',
        [UPLOAD_ERROR]: 'File :f failed to upload. :e'
    };
    const ICONMAP_FILE_STATE = {
        [UPLOAD_SUCCESS]: '✔',
        [UPLOAD_ABORT]: '🟥',
        [UPLOAD_ERROR]: '❌'
    };
    const UNKNOWN_ERROR = 'see logs for info';

    const dropIcon = document.querySelector('[data-upload-icon]');
    const dropCont = document.querySelector('[data-upload-drop]');
    const fileInput = document.querySelector('[data-file]');
    const uploadText = document.querySelector('[data-upload-text]');

    function parseJson(str) {
        try {
            return JSON.parse(str);
        } catch (e) {
            return null;
        }
    }

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
        fileInput.value = '';
    }

    function setReadyToUpload(state, extra) {
        uploadText.textContent = STRMAP_FILE_STATE[state]
            .replace(':f', fileInput.files[0].name)
            .replace(':e', extra || '');

        dropIcon.textContent = ICONMAP_FILE_STATE[state];
        document.body.style.setProperty('--upload-progress', '100%');

        setTimeout(function invokeUnlockUpload() {
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

        req.addEventListener('readystatechange', function onUploadStateChange() {
            if (req.readyState === XMLHttpRequest.DONE) {
                if (req.status === 200) {
                    setReadyToUpload(UPLOAD_SUCCESS);
                } else {
                    const data = parseJson(req.response);

                    if (data && data.error) {
                        setReadyToUpload(UPLOAD_ERROR, data.error);
                    } else {
                        setReadyToUpload(UPLOAD_ERROR, UNKNOWN_ERROR);
                    }

                    console.error('failed to upload %s, status %d', fileInput.files[0].name, req.status, req.response);
                }
            }
        });

        req.upload.addEventListener('error', function onUploadError(ev) {
            setReadyToUpload(UPLOAD_ERROR);
            console.error('failed to upload %s', fileInput.files[0].name, ev);
        });

        req.upload.addEventListener('abort', function onUploadAbort() {
            setReadyToUpload(UPLOAD_ABORT);
        });

        req.open('POST', '/ajax/upload', true);

        req.send(formData);
    });

    unlockUpload();
})();