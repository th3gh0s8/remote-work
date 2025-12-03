// welcome.js - Welcome screen functionality

document.addEventListener('DOMContentLoaded', () => {
    const userIdInput = document.getElementById('userId');
    const continueBtn = document.getElementById('continueBtn');
    const statusMessage = document.getElementById('statusMessage');

    // Check if user ID is already set when page loads
    window.__TAURI__.invoke('is_user_id_set')
        .then(isSet => {
            if (isSet) {
                // If user ID is already set, redirect to main app
                redirectToMainApp();
            }
        })
        .catch(error => {
            console.error('Error checking user ID status:', error);
        });

    // Set up continue button event listener
    continueBtn.addEventListener('click', () => {
        const userId = userIdInput.value.trim();

        if (!userId) {
            showStatusMessage('Please enter a User ID', 'error');
            return;
        }

        // Validate user ID format (you can adjust this as needed)
        if (userId.length < 3) {
            showStatusMessage('User ID must be at least 3 characters long', 'error');
            return;
        }

        // Call the Rust function to set the user ID
        window.__TAURI__.invoke('set_user_id', { userId })
            .then(result => {
                console.log('User ID set successfully:', result);
                showStatusMessage('User ID set successfully! Redirecting...', 'success');
                
                // Redirect to main app after a short delay
                setTimeout(() => {
                    redirectToMainApp();
                }, 1000);
            })
            .catch(error => {
                console.error('Error setting user ID:', error);
                showStatusMessage(`Error: ${error}`, 'error');
            });
    });

    // Allow pressing Enter key to continue
    userIdInput.addEventListener('keypress', (event) => {
        if (event.key === 'Enter') {
            continueBtn.click();
        }
    });

    function showStatusMessage(message, type) {
        statusMessage.textContent = message;
        statusMessage.className = `status-message ${type}`;
    }

    function redirectToMainApp() {
        // Use Tauri's window management to show the main application
        window.__TAURI__.invoke('show_main_window')
            .then(() => {
                console.log('Successfully switched to main window');
            })
            .catch(error => {
                console.error('Error switching to main window:', error);
                // Fallback: reload the page
                window.location.reload();
            });
    }
});