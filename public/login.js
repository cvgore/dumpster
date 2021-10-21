(function initApp() {
    'use strict';

    const STR_LOGIN = 'login';
    const STR_UNAUTHORIZED = 'invalid credentials';

    const user = document.querySelector('#user');
    const pass = document.querySelector('#pass');
    const loginForm = document.querySelector('form');
    const loginFields = document.querySelector('fieldset');
    const loginBtn = document.querySelector('[data-login]');

    function setLoginFailed() {
        return new Promise((resolve) => {
            loginBtn.value = STR_UNAUTHORIZED;

            setTimeout(() => {
                loginBtn.value = STR_LOGIN;
                resolve();
            }, 2000);
        });
    }

    async function login() {
        if (!user.value || !pass.value) {
            return;
        }

        loginFields.disabled = true;

        const formData = new FormData();

        formData.append('user', user.value);
        formData.append('pass', pass.value);

        const resp = await fetch('/ajax/login', {
            method: 'POST',
            body: formData
        });

        if (!resp.ok) {
            await setLoginFailed();
            return;
        }

        try {
            const {token} = await resp.json();
            sessionStorage.setItem('token', token);
            window.location.href = 'panel.html';
        } catch (e) {
            console.error('invalid login response');
            await setLoginFailed();
        }
    }

    loginForm.addEventListener('submit', async function handleLoginClick(ev) {
        ev.preventDefault();
        await login();
        loginFields.disabled = false;
    });

    loginBtn.textContent = STR_LOGIN;

    if (sessionStorage.getItem('token')) {
        window.location.href = 'panel.html';
    }
})();