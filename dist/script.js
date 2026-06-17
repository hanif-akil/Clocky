// Theme configurations
const themes = {
    Midnight: {
        primary: '#82a0ff',
        primaryDark: '#506ec8',
        bgDark: '#0c1026',
        glassBg: 'rgba(12, 16, 38, 0.7)'
    },
    Aurora: {
        primary: '#50d2aa',
        primaryDark: '#28a082',
        bgDark: '#081a18',
        glassBg: 'rgba(8, 26, 24, 0.7)'
    },
    Sakura: {
        primary: '#ffa0b4',
        primaryDark: '#c86e8c',
        bgDark: '#1e0e14',
        glassBg: 'rgba(30, 14, 20, 0.7)'
    },
    Dusk: {
        primary: '#ffbe5a',
        primaryDark: '#c88c32',
        bgDark: '#1c1208',
        glassBg: 'rgba(28, 18, 8, 0.7)'
    },
    Slate: {
        primary: '#64d2e6',
        primaryDark: '#3ca0b4',
        bgDark: '#0e141a',
        glassBg: 'rgba(14, 20, 26, 0.7)'
    }
};

// State
let currentSettings = {
    timeFormat: '12',
    theme: 'Midnight',
    bgOpacity: 0.35,
    bgImagePath: null,
    bgImageData: null
};

let stopwatchInterval = null;
let timerInterval = null;
let alarmInterval = null;

// Initialize
document.addEventListener('DOMContentLoaded', async () => {
    await loadSettings();
    setupEventListeners();
    startClock();
    startStopwatchUpdate();
    startTimerUpdate();
    startAlarmCheck();
});

// Load settings from backend
async function loadSettings() {
    try {
        const settings = await window.__TAURI__.invoke('get_settings');
        currentSettings = settings;
        applySettings();
    } catch (error) {
        console.error('Failed to load settings:', error);
    }
}

// Apply settings to UI
function applySettings() {
    // Apply theme
    applyTheme(currentSettings.theme);

    // Apply time format
    document.querySelectorAll('.toggle-btn').forEach(btn => {
        btn.classList.toggle('active', btn.dataset.format === currentSettings.timeFormat);
    });

    // Apply wallpaper
    if (currentSettings.bgImageData) {
        applyWallpaper(currentSettings.bgImageData, currentSettings.bgOpacity);
        document.getElementById('remove-wallpaper').style.display = 'inline-block';
        if (currentSettings.bgImagePath) {
            const path = currentSettings.bgImagePath;
            const shortPath = path.length > 40 ? '...' + path.slice(-40) : path;
            document.getElementById('wallpaper-path').textContent = shortPath;
        }
    }

    // Apply opacity
    document.getElementById('wallpaper-opacity').value = currentSettings.bgOpacity * 100;
    document.getElementById('opacity-value').textContent = Math.round(currentSettings.bgOpacity * 100) + '%';

    // Apply theme selection
    document.querySelectorAll('.theme-option').forEach(opt => {
        opt.classList.toggle('active', opt.dataset.theme === currentSettings.theme);
    });
}

// Apply theme
function applyTheme(themeName) {
    const theme = themes[themeName];
    if (!theme) return;

    document.documentElement.style.setProperty('--primary', theme.primary);
    document.documentElement.style.setProperty('--primary-dark', theme.primaryDark);
    document.documentElement.style.setProperty('--bg-dark', theme.bgDark);
    document.documentElement.style.setProperty('--glass-bg', theme.glassBg);
}

// Apply wallpaper
function applyWallpaper(imageData, opacity) {
    const overlay = document.querySelector('.background-overlay');
    if (imageData) {
        // Convert RGBA data to data URL
        const canvas = document.createElement('canvas');
        // We'll need to get dimensions from backend or use a default
        canvas.width = 400;
        canvas.height = 600;
        const ctx = canvas.getContext('2d');
        const imageDataObj = new ImageData(new Uint8ClampedArray(atob(imageData).split('').map(c => c.charCodeAt(0))), 400, 600);
        ctx.putImageData(imageDataObj, 0, 0);
        overlay.style.backgroundImage = `url(${canvas.toDataURL()})`;
        overlay.style.opacity = opacity;
    } else {
        overlay.style.backgroundImage = 'none';
    }
}

// Setup event listeners
function setupEventListeners() {
    // Navigation tabs
    document.querySelectorAll('.nav-tab, .settings-btn').forEach(tab => {
        tab.addEventListener('click', () => {
            const tabName = tab.dataset.tab;
            switchTab(tabName);
        });
    });

    // Time format toggle
    document.querySelectorAll('.toggle-btn').forEach(btn => {
        btn.addEventListener('click', async () => {
            const format = btn.dataset.format;
            currentSettings.timeFormat = format;
            await saveSettings();

            document.querySelectorAll('.toggle-btn').forEach(b => {
                b.classList.toggle('active', b.dataset.format === format);
            });
        });
    });

    // Theme selection
    document.querySelectorAll('.theme-option').forEach(opt => {
        opt.addEventListener('click', async () => {
            const theme = opt.dataset.theme;
            currentSettings.theme = theme;
            await saveSettings();

            document.querySelectorAll('.theme-option').forEach(o => {
                o.classList.toggle('active', o.dataset.theme === theme);
            });
        });
    });

    // Wallpaper picker
    document.getElementById('pick-wallpaper').addEventListener('click', async () => {
        try {
            const path = await window.__TAURI__.invoke('pick_image');
            if (path) {
                const imageData = await window.__TAURI__.invoke('load_image_as_base64', { path });
                currentSettings.bgImagePath = path;
                currentSettings.bgImageData = imageData;
                await saveSettings();

                document.getElementById('remove-wallpaper').style.display = 'inline-block';
                const shortPath = path.length > 40 ? '...' + path.slice(-40) : path;
                document.getElementById('wallpaper-path').textContent = shortPath;
            }
        } catch (error) {
            console.error('Failed to pick image:', error);
        }
    });

    // Remove wallpaper
    document.getElementById('remove-wallpaper').addEventListener('click', async () => {
        currentSettings.bgImagePath = null;
        currentSettings.bgImageData = null;
        await saveSettings();

        document.querySelector('.background-overlay').style.backgroundImage = 'none';
        document.getElementById('remove-wallpaper').style.display = 'none';
        document.getElementById('wallpaper-path').textContent = '';
    });

    // Opacity slider
    document.getElementById('wallpaper-opacity').addEventListener('input', async (e) => {
        const value = parseInt(e.target.value);
        currentSettings.bgOpacity = value / 100;
        document.getElementById('opacity-value').textContent = value + '%';

        const overlay = document.querySelector('.background-overlay');
        if (currentSettings.bgImageData) {
            overlay.style.opacity = currentSettings.bgOpacity;
        }

        // Debounce save
        clearTimeout(window.opacitySaveTimeout);
        window.opacitySaveTimeout = setTimeout(() => saveSettings(), 500);
    });

    // Stopwatch controls
    document.getElementById('stopwatch-toggle').addEventListener('click', async () => {
        const btn = document.getElementById('stopwatch-toggle');
        const isRunning = await window.__TAURI__.invoke('get_elapsed_stopwatch') > 0 &&
        btn.textContent === 'Pause';

        if (isRunning) {
            await window.__TAURI__.invoke('pause_stopwatch');
            btn.textContent = 'Start';
            btn.classList.remove('btn-secondary');
            btn.classList.add('btn-primary');
        } else {
            await window.__TAURI__.invoke('start_stopwatch');
            btn.textContent = 'Pause';
            btn.classList.remove('btn-primary');
            btn.classList.add('btn-secondary');
        }
    });

    document.getElementById('stopwatch-reset').addEventListener('click', async () => {
        await window.__TAURI__.invoke('reset_stopwatch');
        document.getElementById('stopwatch-time').textContent = '00:00:00.00';
        const btn = document.getElementById('stopwatch-toggle');
        btn.textContent = 'Start';
        btn.classList.remove('btn-secondary');
        btn.classList.add('btn-primary');
    });

    // Timer controls
    document.getElementById('timer-toggle').addEventListener('click', async () => {
        const btn = document.getElementById('timer-toggle');
        const remaining = await window.__TAURI__.invoke('get_timer_remaining');

        if (remaining === 0 || btn.textContent === 'Start') {
            const minutes = parseInt(document.getElementById('timer-minutes').value) || 0;
            const seconds = parseInt(document.getElementById('timer-seconds').value) || 0;

            if (minutes > 0 || seconds > 0) {
                await window.__TAURI__.invoke('start_timer', { minutes, seconds });
                btn.textContent = 'Pause';
                document.getElementById('timer-inputs').style.display = 'none';
            }
        } else {
            await window.__TAURI__.invoke('pause_timer');
            btn.textContent = 'Start';
        }
    });

    document.getElementById('timer-reset').addEventListener('click', async () => {
        await window.__TAURI__.invoke('reset_timer');
        document.getElementById('timer-inputs').style.display = 'flex';
        document.getElementById('timer-toggle').textContent = 'Start';
        updateTimerDisplay(60); // Default 1 minute
    });

    // Alarm controls
    document.getElementById('alarm-toggle').addEventListener('click', async () => {
        const btn = document.getElementById('alarm-toggle');
        const hour = parseInt(document.getElementById('alarm-hour').value) || 0;
        const minute = parseInt(document.getElementById('alarm-minute').value) || 0;

        if (btn.textContent === 'Set Alarm') {
            await window.__TAURI__.invoke('set_alarm', { hour, minute });
            await window.__TAURI__.invoke('toggle_alarm', { active: true });
            btn.textContent = 'Cancel';

            const format = currentSettings.timeFormat;
            let alarmTime;
            if (format === '12') {
                const h12 = hour === 0 ? 12 : hour > 12 ? hour - 12 : hour;
                const ampm = hour < 12 ? 'AM' : 'PM';
                alarmTime = `${h12.toString().padStart(2, '0')}:${minute.toString().padStart(2, '0')} ${ampm}`;
            } else {
                alarmTime = `${hour.toString().padStart(2, '0')}:${minute.toString().padStart(2, '0')}`;
            }

            document.getElementById('alarm-info').textContent = `Alarm set for ${alarmTime}`;
            document.getElementById('alarm-status').textContent = 'On';
            document.getElementById('alarm-status').style.color = 'var(--primary)';
        } else {
            await window.__TAURI__.invoke('toggle_alarm', { active: false });
            btn.textContent = 'Set Alarm';
            document.getElementById('alarm-info').textContent = '';
            document.getElementById('alarm-status').textContent = 'Off';
            document.getElementById('alarm-status').style.color = 'var(--text-secondary)';
        }
    });
}

// Switch tab
function switchTab(tabName) {
    document.querySelectorAll('.panel').forEach(panel => {
        panel.classList.remove('active');
    });

    document.querySelectorAll('.nav-tab').forEach(tab => {
        tab.classList.remove('active');
    });

    const targetPanel = document.getElementById(`${tabName}-panel`);
    if (targetPanel) {
        targetPanel.classList.add('active');
    }

    const targetTab = document.querySelector(`.nav-tab[data-tab="${tabName}"]`);
    if (targetTab) {
        targetTab.classList.add('active');
    }

    const settingsBtn = document.querySelector('.settings-btn');
    if (tabName === 'settings') {
        settingsBtn.style.color = 'var(--primary)';
    } else {
        settingsBtn.style.color = '';
    }
}

// Start clock
async function startClock() {
    await updateClock();
    setInterval(updateClock, 1000);
}

async function updateClock() {
    try {
        const time = await window.__TAURI__.invoke('get_current_time', {
            format: currentSettings.timeFormat
        });
        const date = await window.__TAURI__.invoke('get_current_date');

        document.getElementById('clock-time').textContent = time;
        document.getElementById('clock-date').textContent = date;
    } catch (error) {
        console.error('Failed to update clock:', error);
    }
}

// Start stopwatch update
async function startStopwatchUpdate() {
    setInterval(async () => {
        try {
            const elapsed = await window.__TAURI__.invoke('get_elapsed_stopwatch');
            const totalSeconds = Math.floor(elapsed / 1000);
            const milliseconds = Math.floor((elapsed % 1000) / 10);
            const hours = Math.floor(totalSeconds / 3600);
            const minutes = Math.floor((totalSeconds % 3600) / 60);
            const seconds = totalSeconds % 60;

            const display = `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}.${milliseconds.toString().padStart(2, '0')}`;
            document.getElementById('stopwatch-time').textContent = display;
        } catch (error) {
            console.error('Failed to update stopwatch:', error);
        }
    }, 10);
}

// Start timer update
async function startTimerUpdate() {
    setInterval(async () => {
        try {
            const remaining = await window.__TAURI__.invoke('get_timer_remaining');
            if (remaining !== null && remaining !== undefined) {
                updateTimerDisplay(remaining);

                if (remaining === 0) {
                    const btn = document.getElementById('timer-toggle');
                    if (btn.textContent === 'Pause') {
                        await window.__TAURI__.invoke('play_alarm_sound');
                        btn.textContent = 'Start';
                        document.getElementById('timer-inputs').style.display = 'flex';
                    }
                }
            }
        } catch (error) {
            console.error('Failed to update timer:', error);
        }
    }, 100);
}

function updateTimerDisplay(seconds) {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    document.getElementById('timer-time').textContent =
    `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;

    // Update ring progress
    const progressCircle = document.getElementById('timer-progress');
    const radius = 85;
    const circumference = 2 * Math.PI * radius;
    const totalDuration = parseInt(document.getElementById('timer-minutes').value) * 60 +
    parseInt(document.getElementById('timer-seconds').value) || 60;
    const progress = seconds / totalDuration;
    const offset = circumference - (progress * circumference);
    progressCircle.style.strokeDashoffset = offset;
}

// Start alarm check
async function startAlarmCheck() {
    setInterval(async () => {
        try {
            const alarmTriggered = await window.__TAURI__.invoke('check_alarm');
            if (alarmTriggered) {
                await window.__TAURI__.invoke('play_alarm_sound');
                document.getElementById('alarm-status').textContent = 'RINGING!';
                document.getElementById('alarm-status').style.color = '#ff6e6e';

                // Auto dismiss after a few seconds
                setTimeout(async () => {
                    await window.__TAURI__.invoke('toggle_alarm', { active: false });
                    document.getElementById('alarm-toggle').textContent = 'Set Alarm';
                    document.getElementById('alarm-info').textContent = '';
                    document.getElementById('alarm-status').textContent = 'Off';
                    document.getElementById('alarm-status').style.color = 'var(--text-secondary)';
                }, 5000);
            }
        } catch (error) {
            console.error('Failed to check alarm:', error);
        }
    }, 1000);
}

// Save settings
async function saveSettings() {
    try {
        await window.__TAURI__.invoke('update_settings', { settings: currentSettings });
    } catch (error) {
        console.error('Failed to save settings:', error);
    }
}
