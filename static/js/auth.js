// static/js/auth.js
class AuthManager {
    static async checkAuth() {
        try {
            const response = await fetch('/api/auth/check', {
                method: 'GET',
                credentials: 'include'
            });
            
            if (response.ok) {
                const result = await response.json();
                if (result.success) {
                    // Stocker les données utilisateur
                    sessionStorage.setItem('userData', JSON.stringify(result.data));
                    return result.data;
                }
            }
            // Supprimer les données si non authentifié
            sessionStorage.removeItem('userData');
            return null;
        } catch (error) {
            console.error('Erreur vérification auth:', error);
            return null;
        }
    }
    
    static async login(username, password) {
        try {
            const response = await fetch('/api/auth/login', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                credentials: 'include',
                body: JSON.stringify({ username, password })
            });
            
            const result = await response.json();
            
            if (result.success && result.data) {
                // Stocker les données utilisateur
                sessionStorage.setItem('userData', JSON.stringify(result.data));
            }
            
            return {
                success: result.success,
                data: result.data,
                message: result.message
            };
        } catch (error) {
            console.error('Erreur login:', error);
            return {
                success: false,
                message: 'Erreur de connexion au serveur'
            };
        }
    }
    
    static async logout() {
        try {
            await fetch('/api/auth/logout', {
                method: 'POST',
                credentials: 'include'
            });
            
            // Supprimer les données locales
            sessionStorage.removeItem('userData');
            return true;
        } catch (error) {
            console.error('Erreur logout:', error);
            return false;
        }
    }
    
    static async getUserData() {
        try {
            const userData = JSON.parse(sessionStorage.getItem('userData'));
            if (!userData) return null;
            
            const response = await fetch('/api/users/data', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                credentials: 'include',
                body: JSON.stringify({ 
                    id: userData.id,
                    username: userData.username 
                })
            });
            
            if (response.ok) {
                const result = await response.json();
                if (result.success && result.data) {
                    // Mettre à jour le sessionStorage
                    const updatedUserData = { ...userData, ...result.data };
                    sessionStorage.setItem('userData', JSON.stringify(updatedUserData));
                    return result.data;
                }
            }
            return null;
        } catch (error) {
            console.error('Erreur récupération données:', error);
            return null;
        }
    }
    
    static isAuthenticated() {
        return !!sessionStorage.getItem('userData');
    }
    
    static getCurrentUser() {
        try {
            const userData = sessionStorage.getItem('userData');
            return userData ? JSON.parse(userData) : null;
        } catch (error) {
            console.error('Erreur parsing userData:', error);
            return null;
        }
    }
    
    static async requireAuth(redirectUrl = '/login') {
        if (!this.isAuthenticated()) {
            window.location.href = redirectUrl;
            return false;
        }
        
        // Vérifier avec le serveur que le cookie est toujours valide
        try {
            const response = await fetch('/api/auth/check', {
                method: 'GET',
                credentials: 'include'
            });
            
            if (!response.ok) {
                sessionStorage.removeItem('userData');
                window.location.href = redirectUrl;
                return false;
            }
            
            return true;
        } catch (error) {
            console.error('Erreur vérification auth:', error);
            sessionStorage.removeItem('userData');
            window.location.href = redirectUrl;
            return false;
        }
    }
}

// Exposer globalement
window.AuthManager = AuthManager;