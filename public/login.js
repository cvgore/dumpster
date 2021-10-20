(function initApp() {
    'use strict';

    const user = document.querySelector('#user');
    const pass = document.querySelector('#pass');
    const loginBtn = document.querySelector('[data-login]');

    async function login() {
        if (!user.value || !pass.value) {
            return;
        }

        const formData = new FormData();

        formData.append('user', user.value);
        formData.append('pass', pass.value);

        const resp = await fetch({
            method: 'POST',
            url: '/ajax/login',
            body: formData
        });

        if (!resp.ok) {
            //

            return;
        }

        const data = await resp.json();
        sessionStorage.setItem('token', data.token);
        window.location.href = 'panel.html';
    }

    loginBtn.addEventListener('click', function handleLoginClick() {
       login();
    });
})();