// static/js/account.js
document.addEventListener("DOMContentLoaded", function () {
    // Éléments DOM
    const profilePicture = document.getElementById("profile-picture");
    const pictureUpload = document.getElementById("picture-upload");
    const changePictureBtn = document.getElementById("change-picture-btn");
    const editForm = document.getElementById("edit-form");
    const formMessage = document.getElementById("form-message");
    const logoutBtn = document.getElementById("logout-btn");

    let currentUser = null;

    // Vérifier immédiatement si l'utilisateur est connecté
    checkAuthentication();

    async function checkAuthentication() {
        try {
            const response = await fetch('/api/auth/check', {
                method: 'GET',
                credentials: 'include'
            });
            
            if (!response.ok) {
                // Rediriger vers la page de login si non connecté
                window.location.href = '/login';
                return;
            }
            
            const result = await response.json();
            if (result.success && result.data) {
                currentUser = {
                    id: result.data.id,
                    username: result.data.username
                };
                
                sessionStorage.setItem('userData', JSON.stringify(currentUser));
                loadUserData(); // Charger les données complètes
                loadProfilePicture(currentUser.id);
                loadGameHistory();
                
                // Afficher le pseudo de l'utilisateur
                const pseudoElement = document.getElementById("userPseudo");
                if (pseudoElement) {
                    pseudoElement.textContent = currentUser.username;
                }
            } else {
                window.location.href = '/login';
            }
        } catch (error) {
            console.error('Erreur vérification authentification:', error);
            window.location.href = '/login';
        }
    }

    // Charger les données utilisateur depuis le serveur
    async function loadUserData() {
        try {
            const userData = JSON.parse(sessionStorage.getItem('userData'));
            
            if (!userData) {
                console.error("Données utilisateur manquantes");
                return;
            }

            // Utiliser le cookie pour l'authentification
            const response = await fetch('/api/users/data', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                credentials: 'include', // IMPORTANT: pour les cookies
                body: JSON.stringify({
                    id: userData.id,
                    username: userData.username // Le serveur vérifie que l'ID correspond au cookie
                })
            });
            
            if (response.ok) {
                const result = await response.json();
                if (result.success && result.data) {
                    // Mettre à jour les données utilisateur
                    currentUser = {
                        ...currentUser,
                        ...result.data
                    };
                    
                    // Mettre à jour l'affichage
                    updateProfileDisplay(result.data);
                } else {
                    console.error("Erreur dans la réponse:", result.message);
                }
            } else if (response.status === 401 || response.status === 403) {
                // Session expirée ou non autorisée
                showFormMessage("Session expirée, veuillez vous reconnecter", "error");
                setTimeout(() => {
                    window.location.href = '/login';
                }, 2000);
            } else {
                console.error("Erreur HTTP:", response.status);
            }
            
        } catch (error) {
            console.error("Erreur chargement données utilisateur:", error);
        }
    }

    async function loadProfilePicture(userId) {
        try {
            // Correction: Utiliser le bon endpoint
            const response = await fetch(`/api/users/profile-picture/${userId}`, {
                method: 'GET',
                credentials: 'include'
            });
            
            if (response.ok) {
                const result = await response.json();
                if (result.success && result.img) {
                    // Convertir les données binaires en URL
                    const blob = new Blob([new Uint8Array(result.img)], { 
                        type: response.headers.get('content-type') || 'image/jpeg' 
                    });
                    const url = URL.createObjectURL(blob);
                    profilePicture.src = url;
                    return;
                }
            }
            // Fallback à l'image par défaut
            profilePicture.src = '/static/img/people.jpg';
        } catch (error) {
            console.error('Erreur chargement photo:', error);
            profilePicture.src = '/static/img/people.jpg';
        }
    }

    function updateProfileDisplay(userData) {
        if (document.getElementById("username-display")) {
            document.getElementById("username-display").textContent = userData.username;
        }
        if (document.getElementById("email-display")) {
            document.getElementById("email-display").textContent = userData.email || "Non défini";
        }
        
        // Formater la date de création
        let creationDate = "Non défini";
        if (userData.created_at) {
            creationDate = new Date(userData.created_at).toLocaleDateString('fr-FR');
        } else if (userData.creation_date) {
            creationDate = userData.creation_date;
        }
        
        if (document.getElementById("creation-date-display")) {
            document.getElementById("creation-date-display").textContent = creationDate;
        }
        
        if (document.getElementById("username")) {
            document.getElementById("username").value = userData.username;
        }
        if (document.getElementById("email")) {
            document.getElementById("email").value = userData.email || "";
        }
    }

    // Gestion de la déconnexion
    if (logoutBtn) {
        logoutBtn.addEventListener('click', async function(e) {
            e.preventDefault();
            try {
                const response = await fetch('/api/auth/logout', {
                    method: 'POST',
                    credentials: 'include'
                });
                
                if (response.ok) {
                    // Supprimer les données locales
                    sessionStorage.removeItem('userData');
                    // Rediriger vers la page de login
                    window.location.href = '/login';
                }
            } catch (error) {
                console.error('Erreur déconnexion:', error);
            }
        });
    }

    if (changePictureBtn && pictureUpload) {
        changePictureBtn.addEventListener("click", function () {
            pictureUpload.click();
        });

        pictureUpload.addEventListener("change", async function (e) {
            const file = e.target.files[0];
            if (file) {
                if (file.size > 5 * 1024 * 1024) {
                    showFormMessage("La photo ne doit pas dépasser 5MB", "error");
                    return;
                }

                if (!file.type.startsWith("image/")) {
                    showFormMessage("Veuillez sélectionner une image valide", "error");
                    return;
                }

                // Afficher un aperçu immédiat
                const reader = new FileReader();
                reader.onload = function (e) {
                    profilePicture.src = e.target.result;
                };
                reader.readAsDataURL(file);

                // Upload vers le serveur
                await uploadProfilePicture(file);
            }
        });
    }

    async function uploadProfilePicture(file) {
        if (!currentUser || !currentUser.username) {
            showFormMessage("Utilisateur non identifié", "error");
            return;
        }

        const formData = new FormData();
        formData.append("image", file);
        formData.append("username", currentUser.username);

        try {
            const response = await fetch("/api/users/profile-update", {
                method: "POST",
                body: formData,
                credentials: 'include' // IMPORTANT: pour les cookies
            });

            const result = await response.json();

            if (response.ok && result.success) {
                showFormMessage("Photo de profil mise à jour avec succès!", "success");

                // Recharger la photo depuis le serveur
                if (currentUser.id) {
                    setTimeout(() => {
                        loadProfilePicture(currentUser.id);
                    }, 1000);
                }
            } else {
                showFormMessage("Erreur lors de l'upload: " + result.message, "error");
                // Recharger l'ancienne photo en cas d'erreur
                loadProfilePicture(currentUser.id);
            }
        } catch (error) {
            console.error("Erreur upload:", error);
            showFormMessage("Erreur lors de l'upload de la photo", "error");
            // Recharger l'ancienne photo en cas d'erreur
            loadProfilePicture(currentUser.id);
        }
    }

    if (editForm) {
        editForm.addEventListener("submit", async function (e) {
            e.preventDefault();

            // Vérifier à nouveau l'authentification
            try {
                const authCheck = await fetch('/api/auth/check', {
                    method: 'GET',
                    credentials: 'include'
                });
                
                if (!authCheck.ok) {
                    showFormMessage("Session expirée, veuillez vous reconnecter", "error");
                    window.location.href = '/login';
                    return;
                }
            } catch (error) {
                showFormMessage("Erreur de connexion", "error");
                return;
            }

            const formData = {
                id: currentUser.id,
                username: document.getElementById("username").value,
                email: document.getElementById("email").value,
                old_password: document.getElementById("current-password").value,
                new_password: document.getElementById("new-password").value
            };

            // Validation côté client
            const confirmPassword = document.getElementById("confirm-password").value;
            if (formData.new_password && formData.new_password !== confirmPassword) {
                showFormMessage("Les mots de passe ne correspondent pas", "error");
                return;
            }

            if (formData.new_password && formData.new_password.length < 6) {
                showFormMessage("Le mot de passe doit contenir au moins 6 caractères", "error");
                return;
            }

            // Envoi au serveur
            try {
                const response = await fetch("/api/users/profile-pp", {
                    method: "PUT",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    credentials: 'include', // IMPORTANT: pour les cookies
                    body: JSON.stringify(formData),
                });

                const result = await response.json();

                if (response.ok && result.success) {
                    showFormMessage("Modifications enregistrées avec succès!", "success");

                    // Mettre à jour l'affichage et le sessionStorage
                    currentUser.username = formData.username || currentUser.username;
                    currentUser.email = formData.email || currentUser.email;
                    updateProfileDisplay(currentUser);

                    // Mettre à jour le sessionStorage
                    const updatedUserData = {
                        ...currentUser,
                        username: formData.username || currentUser.username,
                        email: formData.email || currentUser.email
                    };
                    sessionStorage.setItem('userData', JSON.stringify(updatedUserData));

                    // Vider les champs de mot de passe
                    document.getElementById("current-password").value = "";
                    document.getElementById("new-password").value = "";
                    document.getElementById("confirm-password").value = "";
                } else {
                    showFormMessage(result.message || "Erreur lors de la modification", "error");
                }
            } catch (error) {
                console.error("Erreur:", error);
                showFormMessage("Erreur de connexion au serveur", "error");
            }
        });
    }

    function showFormMessage(message, type) {
        if (!formMessage) return;
        
        formMessage.textContent = message;
        formMessage.className = `form-message ${type}`;
        formMessage.style.display = "block";

        setTimeout(() => {
            formMessage.style.display = "none";
        }, 5000);
    }

    async function loadGameHistory() {
        try {
            if (!currentUser || !currentUser.id) return;

            // Récupérer les parties depuis l'API
            const response = await fetch(`/api/users/games?user_id=${currentUser.id}`, {
                method: 'GET',
                credentials: 'include' // IMPORTANT: pour les cookies
            });
            
            if (response.ok) {
                const result = await response.json();
                if (result.success && result.games) {
                    displayGames(result.games);
                } else {
                    console.error("Erreur dans la réponse:", result.message);
                    displayEmptyGames();
                }
            } else if (response.status === 401 || response.status === 403) {
                // Session expirée
                displayEmptyGames();
                showFormMessage("Session expirée pour charger les parties", "error");
            } else {
                console.error("Erreur HTTP:", response.status);
                displayEmptyGames();
            }
            
        } catch (error) {
            console.error("Erreur chargement des parties:", error);
            displayEmptyGames();
        }
    }

    function displayGames(games) {
        const tbody = document.getElementById("games-tbody");
        if (!tbody) return;
        
        let wins = 0, losses = 0, draws = 0;

        // Vider le tableau existant
        while (tbody.firstChild) {
            tbody.removeChild(tbody.firstChild);
        }

        // Trier les parties par date (les plus récentes en premier)
        games.sort((a, b) => new Date(b.created_at) - new Date(a.created_at));

        games.forEach((game) => {
            const row = document.createElement("tr");
            
            // Date
            const dateCell = document.createElement("td");
            const gameDate = new Date(game.created_at);
            dateCell.textContent = gameDate.toLocaleDateString('fr-FR');
            
            // Adversaire
            const opponentCell = document.createElement("td");
            opponentCell.textContent = game.opponent_username;
            
            // Résultat
            const resultCell = document.createElement("td");
            let resultText = "En cours";
            let resultClass = "";
            
            if (game.result) {
                const userIsWhite = game.user_color === "white";
                
                if (game.result === "1-0") {
                    // Les blancs gagnent
                    if (userIsWhite) {
                        resultText = "Victoire";
                        resultClass = "win";
                        wins++;
                    } else {
                        resultText = "Défaite";
                        resultClass = "loss";
                        losses++;
                    }
                } else if (game.result === "0-1") {
                    // Les noirs gagnent
                    if (!userIsWhite) {
                        resultText = "Victoire";
                        resultClass = "win";
                        wins++;
                    } else {
                        resultText = "Défaite";
                        resultClass = "loss";
                        losses++;
                    }
                } else if (game.result === "1/2-1/2") {
                    resultText = "Nulle";
                    resultClass = "draw";
                    draws++;
                }
            } else {
                resultClass = "pending";
            }
            
            resultCell.textContent = resultText;
            resultCell.className = resultClass;
            
            // Durée
            const durationCell = document.createElement("td");
            durationCell.textContent = calculateGameDuration(game);
            
            // Assembler la ligne
            row.appendChild(dateCell);
            row.appendChild(opponentCell);
            row.appendChild(resultCell);
            row.appendChild(durationCell);
            
            tbody.appendChild(row);
        });

        updateStatistics(wins, losses, draws);
    }

    function calculateGameDuration(game) {
        if (game.pgn) {
            const moveCount = (game.pgn.match(/\d+\./g) || []).length;
            const estimatedMinutes = Math.max(5, Math.min(60, moveCount * 2));
            return `${estimatedMinutes} min`;
        }
        return "15 min";
    }

    function updateStatistics(wins, losses, draws) {
        const totalGames = wins + losses + draws;
        const winRate = totalGames > 0 ? Math.round((wins / totalGames) * 100) : 0;

        const totalGamesElement = document.getElementById("total-games");
        const winsElement = document.getElementById("wins");
        const lossesElement = document.getElementById("losses");
        const drawsElement = document.getElementById("draws");
        const winRateElement = document.getElementById("win-rate");

        if (totalGamesElement) totalGamesElement.textContent = totalGames;
        if (winsElement) winsElement.textContent = wins;
        if (lossesElement) lossesElement.textContent = losses;
        if (drawsElement) drawsElement.textContent = draws;
        if (winRateElement) winRateElement.textContent = `${winRate}%`;
    }

    function displayEmptyGames() {
        const tbody = document.getElementById("games-tbody");
        if (!tbody) return;
        
        // Vider le tableau
        while (tbody.firstChild) {
            tbody.removeChild(tbody.firstChild);
        }
        
        // Afficher un message "Aucune partie"
        const emptyRow = document.createElement("tr");
        const emptyCell = document.createElement("td");
        emptyCell.colSpan = 4;
        emptyCell.textContent = "Aucune partie jouée pour le moment";
        emptyCell.style.textAlign = "center";
        emptyCell.style.padding = "20px";
        emptyCell.style.color = "#666";
        
        emptyRow.appendChild(emptyCell);
        tbody.appendChild(emptyRow);
        
        // Mettre à jour les statistiques à zéro
        updateStatistics(0, 0, 0);
    }
});