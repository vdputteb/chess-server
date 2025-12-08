// static/js/login.js
document.addEventListener('DOMContentLoaded', function() {
    const loginForm = document.getElementById('loginForm');

    // Login form handling
    if (loginForm) {
        loginForm.addEventListener('submit', async function(e) {
            e.preventDefault();
            
            const formData = new FormData(loginForm);
            const data = {
                username: formData.get('username'),
                password: formData.get('password')
            };
            
            try {
                const response = await fetch('/api/auth/login', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    credentials: 'include', // IMPORTANT: pour envoyer/recevoir les cookies
                    body: JSON.stringify(data)
                });
                
                const result = await response.json();

                if (result.success && result.data) {
                    // Stocker UNIQUEMENT les données non sensibles
                    const userData = {
                        id: result.data.id,
                        username: result.data.username
                    };
                    
                    sessionStorage.setItem('userData', JSON.stringify(userData));
                    
                    document.getElementById('errorMessage').style.display = 'none';
                    
                    // Redirection vers la page principale
                    window.location.href = '/';
                } else {
                    document.getElementById('errorMessage').textContent = result.message;
                    document.getElementById('errorMessage').style.display = 'block';
                }
            } catch (error) {
                console.error('Erreur connexion:', error);
                document.getElementById('errorMessage').textContent = 'Erreur de connexion au serveur';
                document.getElementById('errorMessage').style.display = 'block';
            }
        });
    }

    // Vérifier automatiquement l'authentification au chargement
    async function checkAuthStatus() {
        try {
            const response = await fetch('/api/auth/check', {
                method: 'GET',
                credentials: 'include' // Important pour les cookies
            });
            
            if (response.ok) {
                const result = await response.json();
                if (result.success && result.data) {
                    // Utilisateur déjà connecté, rediriger vers la page principale
                    sessionStorage.setItem('userData', JSON.stringify({
                        id: result.data.id,
                        username: result.data.username
                    }));
                    window.location.href = '/';
                }
            }
        } catch (error) {
            console.log('Non authentifié ou erreur:', error);
        }
    }
    
    checkAuthStatus();
});