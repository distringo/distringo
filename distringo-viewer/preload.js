const { contextBridge, ipcRenderer } = require("electron");

window.addEventListener("DOMContentLoaded", () => {
	const replaceText = (selector, text) => {
		const element = document.getElementById(selector);
		if (element) element.innerText = text;
	};

	for (const dependency of ["chrome", "node", "electron"]) {
		replaceText(`${dependency}-version`, process.versions[dependency]);
	}

	const addArgv = (selector, text) => {
		const element = document.querySelector(selector);
		if (element) {
			const li = document.createElement("li");
			li.innerText = text;
			element.appendChild(li);
		}
	};

	for (const argv of process.argv) {
		addArgv("ol#argv", argv);
	}
});

contextBridge.exposeInMainWorld("darkMode", {
	toggle: () => ipcRenderer.invoke("dark-mode:toggle"),
	system: () => ipcRenderer.invoke("dark-mode:system"),
});
