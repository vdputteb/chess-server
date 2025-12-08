// static/js/base.js
document.addEventListener('DOMContentLoaded', function() {
    const pseudo = document.getElementById("userPseudo");
    
    // Vérifier l'authentification au chargement de chaque page
    async function checkAuthAndUpdateDisplay() {
        try {
            const response = await fetch('/api/auth/check', {
                method: 'GET',
                credentials: 'include'
            });
            
            if (response.ok) {
                const result = await response.json();
                if (result.success && result.data && pseudo) {
                    // Afficher le pseudo de l'utilisateur
                    pseudo.textContent = result.data.username;
                    
                    // Stocker les données minimales
                    sessionStorage.setItem('userData', JSON.stringify({
                        id: result.data.id,
                        username: result.data.username
                    }));
                    
                    // Afficher les éléments pour utilisateur connecté
                    showAuthenticatedUI();
                } else {
                    showUnauthenticatedUI();
                }
            } else {
                showUnauthenticatedUI();
            }
        } catch (error) {
            console.log('Erreur vérification auth:', error);
            showUnauthenticatedUI();
        }
    }
    
    function showAuthenticatedUI() {
        // Cacher le lien de login, afficher les autres
        const loginLink = document.getElementById('login-link');
        const logoutLink = document.getElementById('logout-link');
        const accountLink = document.getElementById('account-link');
        
        if (loginLink) loginLink.style.display = 'none';
        if (logoutLink) logoutLink.style.display = 'block';
        if (accountLink) accountLink.style.display = 'block';
        
        // Écouter le clic sur déconnexion
        if (logoutLink) {
            logoutLink.addEventListener('click', async function(e) {
                e.preventDefault();
                try {
                    await fetch('/api/auth/logout', {
                        method: 'POST',
                        credentials: 'include'
                    });
                    
                    // Supprimer les données locales
                    sessionStorage.removeItem('userData');
                    
                    // Recharger la page
                    window.location.href = '/';
                } catch (error) {
                    console.error('Erreur déconnexion:', error);
                }
            });
        }
    }
    
    function showUnauthenticatedUI() {
        // Afficher le lien de login, cacher les autres
        const loginLink = document.getElementById('login-link');
        const logoutLink = document.getElementById('logout-link');
        const accountLink = document.getElementById('account-link');
        
        if (loginLink) loginLink.style.display = 'block';
        if (logoutLink) logoutLink.style.display = 'none';
        if (accountLink) accountLink.style.display = 'none';
        
        // Si on est sur une page protégée (sauf login), rediriger
        const protectedPages = ['/account', '/param', '/online', '/solo']; // Modifié
        const currentPath = window.location.pathname;
        
        // Empêcher l'accès aux pages protégées si non connecté
        if (protectedPages.includes(currentPath) && currentPath !== '/login') {
            window.location.href = '/login';
        }
    }
    
    // Exécuter la vérification
    checkAuthAndUpdateDisplay();
    
    // Vérifier périodiquement l'authentification (toutes les 5 minutes)
    setInterval(checkAuthAndUpdateDisplay, 5 * 60 * 1000);
});