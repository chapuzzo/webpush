if ("serviceWorker" in navigator && "PushManager" in window) {
  navigator.serviceWorker
    .register("/service-worker.js")
    .then(function (registration) {
      console.log("Service Worker Registered", registration);
    })
    .catch(function (error) {
      console.error("Service Worker Registration Failed", error);
    });

  const subscribeButton = document.getElementById("subscribeBtn");
  subscribeButton.addEventListener("click", function () {
    subscribeUserToPush();
  });

  async function subscribeUserToPush() {
    const pubkey = await (await fetch("/pubkey")).text();
    navigator.serviceWorker.ready.then(function (registration) {
      registration.pushManager
        .subscribe({
          userVisibleOnly: true,
          applicationServerKey: urlB64ToUint8Array(pubkey),
        })
        .then(function (subscription) {
          console.log("User is subscribed:", subscription);
          fetch("/subscribe", {
            method: "POST",
            body: JSON.stringify(subscription),
            headers: {
              "Content-Type": "application/json",
            },
          });
        })
        .catch(function (error) {
          console.error("Failed to subscribe the user: ", error);
        });
    });
  }

  function urlB64ToUint8Array(base64String) {
    const padding = "=".repeat((4 - (base64String.length % 4)) % 4);
    const base64 = (base64String + padding)
      .replace(/\-/g, "+")
      .replace(/_/g, "/");
    const rawData = window.atob(base64);
    const outputArray = new Uint8Array(rawData.length);
    for (let i = 0; i < rawData.length; ++i) {
      outputArray[i] = rawData.charCodeAt(i);
    }
    return outputArray;
  }
}
