import React, { useState, useEffect } from 'react';
import './App.css';

const API_BASE = '/api';

function App() {
  const [sensorData, setSensorData] = useState({
    temperature: null,
    velocity: null,
    position: null
  });
  const [settings, setSettings] = useState({
    temperature: { min: 15, max: 35, interval: 7 },
    velocity: { min: 1000, max: 5000, interval: 6 },
    position: { x_range: 100, y_range: 100, z_range: 50, interval: 5 }
  });
  const [lastUpdate, setLastUpdate] = useState(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const response = await fetch(`${API_BASE}/data`);
        if (response.ok) {
          const data = await response.json();
          setSensorData(data);
          setLastUpdate(new Date().toLocaleTimeString());
        }
      } catch (e) {
        console.error('Failed to fetch sensor data:', e);
      }
    };

    fetchData();
    const interval = setInterval(fetchData, 2000);
    return () => clearInterval(interval);
  }, []);

  const sendSettings = async () => {
    try {
      const response = await fetch(`${API_BASE}/settings`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(settings),
      });

      if (response.ok) {
        alert('Settings sent successfully!');
      } else {
        alert('Failed to send settings');
      }
    } catch (error) {
      console.error('Error sending settings:', error);
      alert('Error sending settings');
    }
  };

  const handleSettingChange = (sensor, field, value) => {
    setSettings(prev => ({
      ...prev,
      [sensor]: {
        ...prev[sensor],
        [field]: parseFloat(value) || 0
      }
    }));
  };

  return (
    <div className="App">
      <header className="App-header">
        <h1>IoT Sensor Dashboard</h1>
        <div className="status connected">● Live</div>
      </header>

      <main className="App-main">
        <section className="settings-section">
          <h2>Sensor Settings</h2>
          <div className="settings-grid">
            <div className="setting-card">
              <h3>Temperature Sensor</h3>
              <div className="setting-row">
                <label>Min (°C):</label>
                <input
                  type="number"
                  value={settings.temperature.min}
                  onChange={(e) => handleSettingChange('temperature', 'min', e.target.value)}
                />
              </div>
              <div className="setting-row">
                <label>Max (°C):</label>
                <input
                  type="number"
                  value={settings.temperature.max}
                  onChange={(e) => handleSettingChange('temperature', 'max', e.target.value)}
                />
              </div>
              <div className="setting-row">
                <label>Interval (s):</label>
                <input
                  type="number"
                  value={settings.temperature.interval}
                  onChange={(e) => handleSettingChange('temperature', 'interval', e.target.value)}
                />
              </div>
            </div>

            <div className="setting-card">
              <h3>Velocity Sensor</h3>
              <div className="setting-row">
                <label>Min (rpm):</label>
                <input
                  type="number"
                  value={settings.velocity.min}
                  onChange={(e) => handleSettingChange('velocity', 'min', e.target.value)}
                />
              </div>
              <div className="setting-row">
                <label>Max (rpm):</label>
                <input
                  type="number"
                  value={settings.velocity.max}
                  onChange={(e) => handleSettingChange('velocity', 'max', e.target.value)}
                />
              </div>
              <div className="setting-row">
                <label>Interval (s):</label>
                <input
                  type="number"
                  value={settings.velocity.interval}
                  onChange={(e) => handleSettingChange('velocity', 'interval', e.target.value)}
                />
              </div>
            </div>

            <div className="setting-card">
              <h3>Position Sensor</h3>
              <div className="setting-row">
                <label>X Range (m):</label>
                <input
                  type="number"
                  value={settings.position.x_range}
                  onChange={(e) => handleSettingChange('position', 'x_range', e.target.value)}
                />
              </div>
              <div className="setting-row">
                <label>Y Range (m):</label>
                <input
                  type="number"
                  value={settings.position.y_range}
                  onChange={(e) => handleSettingChange('position', 'y_range', e.target.value)}
                />
              </div>
              <div className="setting-row">
                <label>Z Range (m):</label>
                <input
                  type="number"
                  value={settings.position.z_range}
                  onChange={(e) => handleSettingChange('position', 'z_range', e.target.value)}
                />
              </div>
              <div className="setting-row">
                <label>Interval (s):</label>
                <input
                  type="number"
                  value={settings.position.interval}
                  onChange={(e) => handleSettingChange('position', 'interval', e.target.value)}
                />
              </div>
            </div>
          </div>
          <button className="send-button" onClick={sendSettings}>
            Send Settings
          </button>
        </section>

        <section className="data-section">
          <h2>Sensor Data</h2>
          {lastUpdate && <p className="data-note">Last update: {lastUpdate}</p>}
          <div className="data-grid">
            <div className="data-card">
              <h3>Temperature</h3>
              {sensorData.temperature ? (
                <>
                  <div className="current-value">
                    {sensorData.temperature.value} °C
                  </div>
                  <div className="sensor-meta">
                    Device: {sensorData.temperature.device_id}
                  </div>
                </>
              ) : (
                <div className="no-data">Waiting for data...</div>
              )}
            </div>

            <div className="data-card">
              <h3>Velocity</h3>
              {sensorData.velocity ? (
                <>
                  <div className="current-value">
                    {sensorData.velocity.value} rpm
                  </div>
                  <div className="sensor-meta">
                    Device: {sensorData.velocity.device_id}
                  </div>
                </>
              ) : (
                <div className="no-data">Waiting for data...</div>
              )}
            </div>

            <div className="data-card">
              <h3>Position</h3>
              {sensorData.position ? (
                <>
                  <div className="current-value position">
                    X: {sensorData.position.value.x.toFixed(2)}<br />
                    Y: {sensorData.position.value.y.toFixed(2)}<br />
                    Z: {sensorData.position.value.z.toFixed(2)}
                  </div>
                  <div className="sensor-meta">
                    Device: {sensorData.position.device_id}
                  </div>
                </>
              ) : (
                <div className="no-data">Waiting for data...</div>
              )}
            </div>
          </div>
        </section>
      </main>
    </div>
  );
}

export default App;
