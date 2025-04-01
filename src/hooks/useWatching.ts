import { useState, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { WatchingData } from '../types'

export const useWatching = () => {
    const [watching, setWatching] = useState<WatchingData | null>(null)
    const [status, setStatus] = useState<string>('Loading...')

    const loadWatchingData = useCallback(async () => {
        try {
            setStatus('Loading watching data...')
            const watchingData = await invoke<WatchingData | null>('get_watching_data')
            setWatching(watchingData)
            setStatus(
                watchingData ? `Watching: ${watchingData.title}` : 'Not currently watching anything'
            )
        } catch (error) {
            console.error('Failed to load watching data:', error)
            setStatus('Error loading watching data. Will retry...')
            throw error
        }
    }, [])

    useEffect(() => {
        const subscribeToWatchingUpdates = async () => {
            const unsubscribe = await listen<WatchingData | null>('watching-updated', (event) => {
                setWatching(event.payload)
                if (event.payload) {
                    setStatus(`Watching: ${event.payload.title}`)
                } else {
                    setStatus('Not currently watching anything')
                }
            })

            return unsubscribe
        }

        const unsubscribePromise = subscribeToWatchingUpdates()
        return () => {
            unsubscribePromise.then((fn) => fn())
        }
    }, [])

    return {
        watching,
        status,
        loadWatchingData,
    }
}
