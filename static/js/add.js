// static/js/add.js

class RegistrationForm {
    constructor() {
        this.form = document.getElementById('registerForm');
        this.messageDiv = document.getElementById('registerMessage');
        this.submitBtn = document.getElementById('register-submit-btn');
        this.usernameInput = document.getElementById('reg-username');
        this.emailInput = document.getElementById('reg-email');
        this.passwordInput = document.getElementById('reg-password');
        this.confirmPasswordInput = document.getElementById('reg-confirm-password');
        this.passwordMatchMsg = document.getElementById('password-match');
        this.usernameAvailabilityMsg = document.getElementById('username-availability');
        
        this.init();
    }

    init() {
        if (this.form) {
            this.setupEventListeners();
            this.setupRealTimeValidation();
        }
    }

    setupEventListeners() {
        // Soumission du formulaire
        if (this.form) {
            this.form.addEventListener('submit', (e) => this.handleSubmit(e));
        }
        
        // Validation en temps réel de l'email
        if (this.emailInput) {
            this.emailInput.addEventListener('blur', () => this.validateEmail());
        }
        
        // Vérification de la correspondance des mots de passe
        if (this.passwordInput && this.confirmPasswordInput) {
            this.passwordInput.addEventListener('input', () => this.checkPasswordMatch());
            this.confirmPasswordInput.addEventListener('input', () => this.checkPasswordMatch());
        }
    }

    setupRealTimeValidation() {
        // Vérification du username avec debounce
        if (this.usernameInput) {
            const debouncedCheck = this.debounce((username) => {
                if (username.length >= 3) {
                    this.checkUsernameAvailability(username);
                }
            }, 500);
            
            this.usernameInput.addEventListener('input', (e) => {
                const username = e.target.value.trim();
                debouncedCheck(username);
            });
        }
    }

    async handleSubmit(e) {
        e.preventDefault();
        
        if (!this.validateForm()) {
            return;
        }

        await this.submitForm();
    }

    validateForm() {
        const formData = this.getFormData();

        // Validation des champs requis
        if (!formData.username || !formData.email || !formData.password) {
            this.showMessage('Veuillez remplir tous les champs', 'error');
            return false;
        }

        // Validation du nom d'utilisateur
        if (formData.username.length < 3) {
            this.showMessage('Le nom d\'utilisateur doit contenir au moins 3 caractères', 'error');
            return false;
        }

        // Validation de l'email
        if (!this.isValidEmail(formData.email)) {
            this.showMessage('Format d\'email invalide', 'error');
            return false;
        }

        // Validation du mot de passe
        if (formData.password.length < 6) {
            this.showMessage('Le mot de passe doit contenir au moins 6 caractères', 'error');
            return false;
        }

        // Vérification de la correspondance des mots de passe
        if (formData.password !== this.confirmPasswordInput.value) {
            this.showMessage('Les mots de passe ne correspondent pas', 'error');
            return false;
        }

        return true;
    }

    getFormData() {
        return {
            username: this.usernameInput ? this.usernameInput.value.trim() : '',
            email: this.emailInput ? this.emailInput.value.trim() : '',
            password: this.passwordInput ? this.passwordInput.value : ''
        };
    }

    async submitForm() {
        this.setLoadingState(true);

        try {
            const formData = this.getFormData();
            console.log(JSON.stringify(formData));
            const response = await fetch('/api/auth/register', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(formData)
            });

            const data = await response.json();

            if (response.ok && data.success) {
                this.handleSuccess(data.message);
            } else {
                this.handleError(data.message);
            }

        } catch (error) {
            console.error('Erreur lors de la création du compte:', error);
            this.handleError('Erreur de connexion au serveur');
        } finally {
            this.setLoadingState(false);
        }
    }

    handleSuccess(message) {
        this.showMessage(message, 'success');
        if (this.form) {
            this.form.reset();
        }
        
        // Retour au formulaire de connexion après 2 secondes avec animation
        setTimeout(() => {
            const showLoginBtn = document.getElementById('showLoginBtn');
            if (showLoginBtn) {
                showLoginBtn.click();
            }
        }, 2000);
    }

    handleError(message) {
        this.showMessage(message, 'error');
    }

    setLoadingState(isLoading) {
        if (this.submitBtn) {
            this.submitBtn.disabled = isLoading;
            this.submitBtn.textContent = isLoading ? 'Création en cours...' : 'Créer un compte';
        }
        
        // Désactiver tous les champs pendant le chargement
        if (this.form) {
            const inputs = this.form.querySelectorAll('input');
            inputs.forEach(input => {
                input.disabled = isLoading;
            });
        }
    }

    checkPasswordMatch() {
        if (!this.passwordInput || !this.confirmPasswordInput || !this.passwordMatchMsg) {
            return;
        }

        const password = this.passwordInput.value;
        const confirmPassword = this.confirmPasswordInput.value;

        if (!password || !confirmPassword) {
            this.passwordMatchMsg.textContent = '';
            if (this.submitBtn) this.submitBtn.disabled = true;
            return;
        }

        if (password !== confirmPassword) {
            this.passwordMatchMsg.textContent = '❌ Les mots de passe ne correspondent pas';
            this.passwordMatchMsg.className = 'field-message field-error';
            if (this.submitBtn) this.submitBtn.disabled = true;
        } else {
            this.passwordMatchMsg.textContent = '✅ Les mots de passe correspondent';
            this.passwordMatchMsg.className = 'field-message field-success';
            
            // Réactiver le bouton seulement si tous les autres critères sont valides
            const formData = this.getFormData();
            if (this.submitBtn) {
                this.submitBtn.disabled = !(formData.username && formData.email && formData.password);
            }
        }
    }

    async checkUsernameAvailability(username) {
        try {
            const response = await fetch(`/api/auth/check-username?username=${encodeURIComponent(username)}`);
            const data = await response.json();

            if (response.ok) {
                if (!data.available) {
                    this.showUsernameError(data.message);
                } else {
                    this.clearUsernameError();
                }
            }
        } catch (error) {
            console.log('Erreur de vérification du username:', error);
        }
    }

    showUsernameError(message) {
        if (!this.usernameAvailabilityMsg) return;
        
        this.usernameAvailabilityMsg.textContent = '❌ ' + message;
        this.usernameAvailabilityMsg.className = 'field-message field-error';
        if (this.submitBtn) this.submitBtn.disabled = true;
    }

    clearUsernameError() {
        if (!this.usernameAvailabilityMsg) return;
        
        this.usernameAvailabilityMsg.textContent = '✅ Nom d\'utilisateur disponible';
        this.usernameAvailabilityMsg.className = 'field-message field-success';
        
        // Réactiver le bouton si les autres validations passent
        const formData = this.getFormData();
        const passwordsMatch = this.passwordInput && this.confirmPasswordInput ? 
            this.passwordInput.value === this.confirmPasswordInput.value : false;
        
        if (this.submitBtn) {
            this.submitBtn.disabled = !(formData.username && formData.email && formData.password && passwordsMatch);
        }
    }

    validateEmail() {
        const email = this.emailInput ? this.emailInput.value.trim() : '';
        if (email && !this.isValidEmail(email)) {
            this.showMessage('Format d\'email invalide', 'error');
            return false;
        }
        return true;
    }

    isValidEmail(email) {
        const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        return emailRegex.test(email);
    }

    showMessage(message, type) {
        if (!this.messageDiv) return;

        this.messageDiv.textContent = message;
        this.messageDiv.className = `error-message ${type}`;
        this.messageDiv.style.display = 'block';

        // Cacher automatiquement les messages d'erreur après 5 secondes
        if (type === 'error') {
            setTimeout(() => {
                this.messageDiv.style.display = 'none';
            }, 5000);
        }
    }

    debounce(func, wait) {
        let timeout;
        return function executedFunction(...args) {
            const later = () => {
                clearTimeout(timeout);
                func(...args);
            };
            clearTimeout(timeout);
            timeout = setTimeout(later, wait);
        };
    }
}

// Initialisation lorsque le DOM est chargé
document.addEventListener('DOMContentLoaded', function() {
    new RegistrationForm();
});

// Fonctions utilitaires globales pour une utilisation externe
window.RegistrationUtils = {
    validateEmail: function(email) {
        const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        return emailRegex.test(email);
    },
    
    validatePassword: function(password) {
        return password.length >= 6;
    },
    
    validateUsername: function(username) {
        return username.length >= 3 && username.length <= 50;
    }
};