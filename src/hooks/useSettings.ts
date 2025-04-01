import { useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Settings } from '../types'

export const useSettings = () => {
    const [settings, setSettings] = useState<Settings>({
        user_id: null,
        show_anime_title: true,
        show_episode_title: true,
        show_episode_number: true,
        show_progress: true,
        show_timestamp: true,
        enabled: true,
    })

    const loadSettings = useCallback(async () => {
        try {
            const savedSettings = await invoke<Settings>('get_settings')
            setSettings(savedSettings)
            return savedSettings
        } catch (error) {
            console.error('Failed to load settings:', error)
            throw error
        }
    }, [])

    const saveSettings = useCallback(async (newSettings: Settings) => {
        try {
            await invoke('save_settings_command', { settings: newSettings })
            setSettings(newSettings)
        } catch (error) {
            console.error('Failed to save settings:', error)
            throw error
        }
    }, [])

    return {
        settings,
        loadSettings,
        saveSettings,
    }
}
