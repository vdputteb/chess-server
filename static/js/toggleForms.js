// static/js/toggleForms.js

document.addEventListener('DOMContentLoaded', function() {
    const card = document.getElementById('auth-card');
    const showRegisterBtn = document.getElementById('showRegisterBtn');
    const showLoginBtn = document.getElementById('showLoginBtn');
    let isAnimating = false;

    // Fonction pour basculer entre les formulaires avec animation
    function showRegisterForm() {
        if (isAnimating || !card) return;
        
        isAnimating = true;
        card.classList.add('flipped');
        
        // Réinitialiser les messages d'erreur du login
        const errorMessage = document.getElementById('errorMessage');
        if (errorMessage) {
            errorMessage.style.display = 'none';
        }
        
        setTimeout(() => {
            isAnimating = false;
        }, 800);
    }

    function showLoginForm() {
        if (isAnimating || !card) return;
        
        isAnimating = true;
        card.classList.remove('flipped');
        
        // Réinitialiser les messages d'erreur de l'inscription
        const registerMessage = document.getElementById('registerMessage');
        if (registerMessage) {
            registerMessage.style.display = 'none';
        }
        
        setTimeout(() => {
            isAnimating = false;
        }, 800);
    }

    // Événements pour basculer entre les formulaires
    if (showRegisterBtn) {
        showRegisterBtn.addEventListener('click', showRegisterForm);
    }

    if (showLoginBtn) {
        showLoginBtn.addEventListener('click', showLoginForm);
    }

    // Empêcher la soumission du formulaire pendant l'animation
    const forms = document.querySelectorAll('form');
    forms.forEach(form => {
        form.addEventListener('submit', function(e) {
            if (isAnimating) {
                e.preventDefault();
            }
        });
    });
});