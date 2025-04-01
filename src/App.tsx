import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './App.css'

import { Page } from './types'
import { useSettings } from './hooks/useSettings'
import { useWatching } from './hooks/useWatching'

import { StatusFooter } from './components/status-footer'
import { SetupPage } from './components/setup-page'
import { MainPage } from './components/main-page'
import { SettingsPage } from './components/settings-page'

function App() {
    const [userId, setUserId] = useState<string>('')
    const [page, setPage] = useState<Page>('setup')
    const { settings, loadSettings, saveSettings } = useSettings()
    const { watching, status, loadWatchingData } = useWatching()

    useEffect(() => {
        const initializeApp = async () => {
            try {
                const savedSettings = await loadSettings()
                if (savedSettings.user_id) {
                    setUserId(savedSettings.user_id)
                    setPage('main')
                    await loadWatchingData()
                }
            } catch (error) {
                console.error('Failed to initialize app:', error)
            }
        }

        initializeApp()
    }, [loadSettings, loadWatchingData])

    const handleSave = async () => {
        try {
            await invoke('save_user_id', { id: userId })
            const newSettings = { ...settings, user_id: userId }
            await saveSettings(newSettings)
            setPage('main')
        } catch (error) {
            console.error('Failed to save User ID:', error)
        }
    }

    return (
        <main className='container'>
            {page === 'setup' && (
                <SetupPage userId={userId} onUserIdChange={setUserId} onSave={handleSave} />
            )}
            {page === 'main' && (
                <MainPage watching={watching} onSettingsClick={() => setPage('settings')} />
            )}
            {page === 'settings' && (
                <SettingsPage
                    userId={userId}
                    settings={settings}
                    onUserIdChange={setUserId}
                    onSettingsChange={saveSettings}
                    onBack={() => setPage('main')}
                />
            )}
            <StatusFooter status={status} />
        </main>
    )
}

export default App
