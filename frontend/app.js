// Check if the browser supports Push Notifications and Service Workers
if ('serviceWorker' in navigator && 'PushManager' in window) {
    // Register service worker
    navigator.serviceWorker.register('/service-worker.js').then(function(registration) {
        console.log('Service Worker Registered', registration);
    }).catch(function(error) {
        console.error('Service Worker Registration Failed', error);
    });

    // Push subscription process
    const subscribeButton = document.getElementById('subscribeBtn');
    subscribeButton.addEventListener('click', function() {
        subscribeUserToPush();
    });

    function subscribeUserToPush() {
        navigator.serviceWorker.ready.then(function(registration) {
            registration.pushManager.subscribe({
                userVisibleOnly: true,
                applicationServerKey: urlB64ToUint8Array('YOUR_PUBLIC_VAPID_KEY') // You must use your own VAPID key
            }).then(function(subscription) {
                console.log('User is subscribed:', subscription);
                // Send subscription to the server (for storing and later use)
                fetch('/subscribe', {
                    method: 'POST',
                    body: JSON.stringify(subscription),
                    headers: {
                        'Content-Type': 'application/json'
                    }
                });
            }).catch(function(error) {
                console.error('Failed to subscribe the user: ', error);
            });
        });
    }

    // Convert VAPID public key from base64 to UInt8Array
    function urlB64ToUint8Array(base64String) {
        const padding = '='.repeat((4 - base64String.length % 4) % 4);
        const base64 = (base64String + padding)
            .replace(/\-/g, '+')
            .replace(/_/g, '/');
        const rawData = window.atob(base64);
        const outputArray = new Uint8Array(rawData.length);
        for (let i = 0; i < rawData.length; ++i) {
            outputArray[i] = rawData.charCodeAt(i);
        }
        return outputArray;
    
}
}
